use crate::ApiError;
use hyper::Request;

/// Provides handy functions for parsing the query parameters of a URL.

#[derive(Clone, Copy)]
pub struct UrlQuery<'a>(url::form_urlencoded::Parse<'a>);

impl<'a> UrlQuery<'a> {
    /// Instantiate from an existing `Request`.
    ///
    /// Returns `Err` if `req` does not contain any query parameters.
    pub fn from_request<T>(req: &'a Request<T>) -> Result<Self, ApiError> {
        let query_str = req.uri().query().ok_or_else(|| {
            ApiError::BadRequest(
                "URL query must be valid and contain at least one key.".to_string(),
            )
        })?;

        Ok(UrlQuery(url::form_urlencoded::parse(query_str.as_bytes())))
    }

    /// Returns the first `(key, value)` pair found where the `key` is in `keys`.
    ///
    /// If no match is found, an `InvalidQueryParams` error is returned.
    pub fn first_of(mut self, keys: &[&str]) -> Result<(String, String), ApiError> {
        self.0
            .find(|(key, _value)| keys.contains(&&**key))
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .ok_or_else(|| {
                ApiError::BadRequest(format!(
                    "URL query must contain at least one of the following keys: {:?}",
                    keys
                ))
            })
    }

    /// Returns the value for `key`, if and only if `key` is the only key present in the query
    /// parameters.
    pub fn only_one(self, key: &str) -> Result<String, ApiError> {
        let queries: Vec<_> = self
            .0
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        if queries.len() == 1 {
            let (first_key, first_value) = &queries[0]; // Must have 0 index if len is 1.
            if first_key == key {
                Ok(first_value.to_string())
            } else {
                Err(ApiError::BadRequest(format!(
                    "Only the {} query parameter is supported",
                    key
                )))
            }
        } else {
            Err(ApiError::BadRequest(format!(
                "Only one query parameter is allowed, {} supplied",
                queries.len()
            )))
        }
    }

    /// Returns a vector of all values present where `key` is in `keys
    ///
    /// If no match is found, an `InvalidQueryParams` error is returned.
    pub fn all_of(self, key: &str) -> Result<Vec<String>, ApiError> {
        let queries: Vec<_> = self
            .0
            .filter_map(|(k, v)| {
                if k.eq(key) {
                    Some(v.into_owned())
                } else {
                    None
                }
            })
            .collect();
        Ok(queries)
    }
}
