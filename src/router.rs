/* src/router.rs */

use crate::HandlerFn;
use std::collections::HashMap;

/// Represents an HTTP method.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
}

impl Method {
    /// Tries to convert a string slice to a Method.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "GET" => Some(Method::Get),
            "POST" => Some(Method::Post),
            "PUT" => Some(Method::Put),
            "DELETE" => Some(Method::Delete),
            _ => None,
        }
    }
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Get => write!(f, "GET"),
            Method::Post => write!(f, "POST"),
            Method::Put => write!(f, "PUT"),
            Method::Delete => write!(f, "DELETE"),
        }
    }
}

/// A key for the routes map, composed of a method and a path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RouteKey {
    method: Method,
    path: String,
}

/// The router, responsible for managing routes and their handlers.
pub struct Router {
    routes: HashMap<RouteKey, HandlerFn>,
}

impl Router {
    /// Creates a new, empty router.
    pub fn new() -> Self {
        Self {
            routes: HashMap::new(),
        }
    }

    /// Adds a new route to the router.
    pub fn add_route(&mut self, method: Method, path: &str, handler: HandlerFn) {
        let key = RouteKey {
            method,
            path: path.to_string(),
        };
        self.routes.insert(key, handler);
    }

    /// Finds a handler that matches the given method and path.
    pub fn find_handler(&self, method: &Method, path: &str) -> Option<&HandlerFn> {
        let key = RouteKey {
            method: method.clone(),
            path: path.to_string(),
        };
        self.routes.get(&key)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}
