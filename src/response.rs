/* src/response.rs */

use std::collections::HashMap;

/// Represents an HTTP-like response.
#[derive(Debug, Clone)]
pub struct Response {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    /// Creates a new response with a given status code.
    pub fn new(status_code: u16) -> Self {
        Self {
            status_code,
            headers: HashMap::new(),
            body: Vec::new(),
        }
    }

    /// Creates a 200 OK response.
    pub fn ok() -> Self {
        Self::new(200)
    }

    /// Creates a 404 Not Found response.
    pub fn not_found(message: &str) -> Self {
        Self::new(404).with_text(message)
    }

    /// Creates a 500 Internal Server Error response.
    pub fn internal_error(message: &str) -> Self {
        Self::new(500).with_text(message)
    }

    /// Adds a header to the response.
    pub fn with_header(mut self, name: &str, value: &str) -> Self {
        self.headers.insert(name.to_string(), value.to_string());
        self
    }

    /// Sets a plain text body for the response.
    pub fn with_text(mut self, text: &str) -> Self {
        self.body = text.as_bytes().to_vec();
        self.headers.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        );
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    /// Sets a JSON body for the response.
    pub fn with_json(mut self, json_str: &str) -> Self {
        self.body = json_str.as_bytes().to_vec();
        self.headers
            .insert("Content-Type".to_string(), "application/json".to_string());
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    /// Sets an HTML body for the response.
    pub fn with_html(mut self, html: &str) -> Self {
        self.body = html.as_bytes().to_vec();
        self.headers.insert(
            "Content-Type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    /// Sets a binary body for the response.
    pub fn with_binary(mut self, data: Vec<u8>, content_type: &str) -> Self {
        self.body = data;
        self.headers
            .insert("Content-Type".to_string(), content_type.to_string());
        self.headers
            .insert("Content-Length".to_string(), self.body.len().to_string());
        self
    }

    /// Gets the status code.
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    /// Gets the response headers.
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Gets the response body.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Converts the Response struct into a raw HTTP response string.
    pub fn to_http_response(&self) -> String {
        let status_line = format!("HTTP/1.1 {} {}", self.status_code, self.status_text());
        let mut headers_string = String::new();
        for (key, value) in &self.headers {
            headers_string.push_str(&format!("{}: {}\r\n", key, value));
        }
        let body_string = String::from_utf8_lossy(&self.body);

        format!("{}\r\n{}\r\n{}", status_line, headers_string, body_string)
    }

    /// Returns the standard reason phrase for a status code.
    fn status_text(&self) -> &'static str {
        match self.status_code {
            200 => "OK",
            201 => "Created",
            204 => "No Content",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            500 => "Internal Server Error",
            _ => "Unknown",
        }
    }
}

// Convenience functions for creating common response types.
impl Response {
    /// Creates a 200 OK response with a JSON body.
    pub fn json(data: &str) -> Self {
        Self::ok().with_json(data)
    }

    /// Creates a 200 OK response with a plain text body.
    pub fn text(text: &str) -> Self {
        Self::ok().with_text(text)
    }

    /// Creates a 200 OK response with an HTML body.
    pub fn html(html: &str) -> Self {
        Self::ok().with_html(html)
    }
}
