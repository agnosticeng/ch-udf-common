use std::task::{Context,Poll};
use std::time::Duration;
use backoff::backoff::Backoff;
use backoff::exponential::{ExponentialBackoff, ExponentialBackoffBuilder};
use backoff::SystemClock;
use serde::Deserialize;
use alloy::transports::{TransportError,TransportErrorKind,TransportFut};
use alloy_json_rpc::{RequestPacket,ResponsePacket,ErrorPayload};
use tower::{Layer,Service};
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub retryable_status_codes: Vec<u16>,
    pub initial_interval: Duration,
    pub randomization_factor: f64,
    pub multiplier: f64,
    pub max_interval: Duration,
    pub max_elapsed_time: Duration,
    pub max_tries: u32,
}

#[derive(Debug, Clone)]
pub struct RetryService<S> {
    inner: S,
    conf: RetryConfig
}

impl<S> Service<RequestPacket> for RetryService<S>
where
    S: Service<RequestPacket, Future = TransportFut<'static>, Error = TransportError>
        + Send
        + 'static
        + Clone
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: RequestPacket) -> Self::Future {
        let inner = self.inner.clone();
        let this = self.clone();
        let mut inner = std::mem::replace(&mut self.inner, inner);

        Box::pin(async move {
            let mut tries: u32 = 0;
            let mut bkoff: ExponentialBackoff<SystemClock> = ExponentialBackoffBuilder::new()
                .with_initial_interval(this.conf.initial_interval)
                .with_max_elapsed_time(Some(this.conf.max_elapsed_time))
                .with_max_interval(this.conf.max_interval)
                .with_multiplier(this.conf.multiplier)
                .with_randomization_factor(this.conf.randomization_factor)
                .build();

            loop {
                tries += 1;
                let res = inner.call(request.clone()).await;

                if res.is_ok() {
                    return res
                }

                let err = res.unwrap_err();

                if !this.should_retry(&err) {
                    return Err(err);
                }
                
                let delay = this
                    .backoff_hint(&err)
                    .or_else(|| bkoff.next_backoff());

                if tries > this.conf.max_tries || delay.is_none() {
                    return Err(TransportErrorKind::custom_str(&format!(
                        "max retries exceeded {err}"
                    )));
                }

                sleep(delay.unwrap()).await;
            }
        })
    }
}

impl<S> RetryService<S> {
    fn should_retry(&self, err: &TransportError) -> bool {
        match err {
            TransportError::Transport(TransportErrorKind::HttpError(e)) => self.conf.retryable_status_codes.contains(&e.status),
            TransportError::Transport(err) => err.is_retry_err(),
            TransportError::DeserError { text, .. } => {
                if let Ok(resp) = serde_json::from_str::<ErrorPayload>(text) {
                    return resp.is_retry_err();
                }

                #[derive(Deserialize)]
                struct Resp {
                    error: ErrorPayload,
                }

                if let Ok(resp) = serde_json::from_str::<Resp>(text) {
                    return resp.error.is_retry_err();
                }

                false
            }
            TransportError::ErrorResp(err) => err.is_retry_err(),
            _ => false,
        }
    }

    fn backoff_hint(&self, err: &TransportError) -> Option<std::time::Duration> {
        // we must have access to HTTP response header to get value from Retry-After header
        // if let TransportError::Transport(TransportErrorKind::HttpError(e)) = err {
        //     if e.status == 429 {
                
        //     }
        // }

        if let TransportError::ErrorResp(resp) = err {
            let data = resp.try_data_as::<serde_json::Value>();

            if let Some(Ok(data)) = data {
                // if daily rate limit exceeded, infura returns the requested backoff in the error
                // response
                let backoff_seconds = &data["rate"]["backoff_seconds"];
                // infura rate limit error
                if let Some(seconds) = backoff_seconds.as_u64() {
                    return Some(std::time::Duration::from_secs(seconds));
                }
                if let Some(seconds) = backoff_seconds.as_f64() {
                    return Some(std::time::Duration::from_secs(seconds as u64 + 1));
                }
            }
        }
        None
    }
    
}

#[derive(Debug, Clone)]
pub struct RetryLayer {
    conf: RetryConfig,
}

impl RetryLayer {
    pub fn new(conf: RetryConfig) -> Self {
        RetryLayer { conf }
    }
}

impl<S> Layer<S> for RetryLayer {
    type Service = RetryService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RetryService {
            inner,
            conf: self.conf.clone()
        }
    }
}
