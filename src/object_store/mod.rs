use querystring::querify;

pub fn opts_from_query_string(s: &str) -> Vec<(String, String)> {
    querify(s)
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

pub fn opts_from_env() -> Vec<(String, String)> {
    std::env::vars()
        .map(|(k, v)| (k.to_lowercase(), v))
        .collect()
}
