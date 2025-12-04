use std::result::Result;
use std::str;
use thiserror::Error;
use url::Url;

#[derive(Error, Debug)]
pub enum UrlExtError {
    #[error("parse error")]
    Parse(#[from] url::ParseError),
    #[error("cannot be a base")]
    CannotBeABase,
}

pub trait UrlExt {
    fn append_path_segments<I>(&self, segments: I) -> Result<Url, UrlExtError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>;
}

impl UrlExt for Url {
    fn append_path_segments<I>(&self, segments: I) -> Result<Url, UrlExtError>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut u = Url::parse(self.as_str())?;
        u.path_segments_mut()
            .map_err(|_| UrlExtError::CannotBeABase)?
            .extend(segments);
        Ok(u)
    }
}
