/* src/request.rs */

use crate::router::Method;
use std::collections::HashMap;
use url::form_urlencoded;

/// Represents an incoming HTTP-like request.
#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    /// Creates a new Request.
    pub fn new(
        method: Method,
        path: String,
        headers: HashMap<String, String>,
        body: Vec<u8>,
    ) -> Self {
        Self {
            method,
            path,
            headers,
            body,
        }
    }

    /// Gets the request method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Gets the full request path, including query string.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Gets all request headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Gets a specific header value by name.
    pub fn header(&self, name: &str) -> Option<&String> {
        self.headers.get(name)
    }

    /// Gets the raw request body as bytes.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Gets the request body as a string.
    pub fn body_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Parses the query parameters from the path using the `url` crate.
    pub fn query_params(&self) -> HashMap<String, String> {
        self.path
            .split_once('?')
            .map(|(_, query_string)| {
                form_urlencoded::parse(query_string.as_bytes())
                    .into_owned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets the path part of the URL, without the query string.
    pub fn path_without_query(&self) -> &str {
        self.path.split('?').next().unwrap_or(&self.path)
    }
}
