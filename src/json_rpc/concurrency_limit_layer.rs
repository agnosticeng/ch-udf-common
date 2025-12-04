// Near-copy of : https://docs.rs/tower/latest/src/tower/limit/concurrency/service.rs.html

use alloy::transports::{TransportError, TransportFut};
use alloy_json_rpc::{RequestPacket, ResponsePacket};
use futures_core::ready;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio_util::sync::PollSemaphore;
use tower::{Layer, Service};

#[derive(Debug)]
pub struct ConcurrencyLimitService<S> {
    inner: S,
    sem: PollSemaphore,
    permit: Option<OwnedSemaphorePermit>,
}

impl<S> ConcurrencyLimitService<S> {
    pub fn new(inner: S, limit: usize) -> Self {
        ConcurrencyLimitService {
            inner,
            sem: PollSemaphore::new(Arc::new(Semaphore::new(limit))),
            permit: None,
        }
    }
}

impl<T: Clone> Clone for ConcurrencyLimitService<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            sem: self.sem.clone(),
            permit: None,
        }
    }
}

impl<S> Service<RequestPacket> for ConcurrencyLimitService<S>
where
    S: Service<RequestPacket, Future = TransportFut<'static>, Error = TransportError>
        + Send
        + 'static
        + Clone,
{
    type Response = ResponsePacket;
    type Error = TransportError;
    type Future = TransportFut<'static>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.permit.is_none() {
            self.permit = ready!(self.sem.poll_acquire(cx));
        }

        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: RequestPacket) -> Self::Future {
        let _ = self.permit.take();
        self.inner.call(request)
    }
}

#[derive(Debug, Clone)]
pub struct ConcurrencyLimitLayer {
    limit: usize,
}

impl ConcurrencyLimitLayer {
    pub fn new(limit: usize) -> Self {
        ConcurrencyLimitLayer { limit }
    }
}

impl<S> Layer<S> for ConcurrencyLimitLayer {
    type Service = ConcurrencyLimitService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ConcurrencyLimitService::new(inner, self.limit)
    }
}
