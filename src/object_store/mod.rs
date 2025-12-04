use querystring::querify;
use url::Url;

pub fn opts_from_url(u: &Url) -> Vec<(String, String)> {
    querify(u.fragment().unwrap_or_default())
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub fn opts_from_env() -> Vec<(String, String)> {
    std::env::vars()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect()
}
