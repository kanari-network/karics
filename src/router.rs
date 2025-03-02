use hyper::{Method, Response};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
pub enum RouterError {
    InvalidPath,
    MethodNotAllowed(Method),
    NotFound(String),
    InvalidPattern(String),
}

#[derive(Debug, PartialEq)]
pub enum MatchType {
    Exact,
    Regex,
    Prefix,
}

pub struct Route<ResponseBody> {
    pattern: Regex,
    _match_type: MatchType,
    handler: Box<dyn Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync>,
}

pub struct Router<ResponseBody> {
    routes: HashMap<Method, Vec<Route<ResponseBody>>>,
}

impl<ResponseBody> Router<ResponseBody> {
    pub fn new() -> Self {
        Router {
            routes: HashMap::with_capacity(32), // Pre-allocate space
        }
    }

    // Advanced route registration with method chaining
    pub fn route<F>(
        &mut self,
        method: Method,
        pattern: &str,
        match_type: MatchType,
        handler: F,
    ) -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        let pattern = match match_type {
            MatchType::Exact => format!("^{}$", regex::escape(pattern)),
            MatchType::Regex => format!("^{}$", pattern),
            MatchType::Prefix => format!("^{}.*$", regex::escape(pattern)),
        };

        let regex = Regex::new(&pattern).map_err(|e| RouterError::InvalidPattern(e.to_string()))?;

        let route = Route {
            pattern: regex,
            _match_type: match_type,
            handler: Box::new(handler),
        };

        self.routes
            .entry(method)
            .or_insert_with(|| Vec::with_capacity(16))
            .push(route);

        Ok(self)
    }

    // Optimized request handling with early returns
    pub fn handle(
        &self,
        method: &Method,
        path: &str,
    ) -> Result<Response<ResponseBody>, RouterError> {
        let routes = self
            .routes
            .get(method)
            .ok_or_else(|| RouterError::MethodNotAllowed(method.clone()))?;

        for route in routes {
            if let Some(captures) = route.pattern.captures(path) {
                let path_params: Vec<String> = captures
                    .iter()
                    .skip(1)
                    .filter_map(|c| c.map(|m| m.as_str().to_string()))
                    .collect();

                return Ok((route.handler)(path_params));
            }
        }

        Err(RouterError::NotFound(path.to_string()))
    }

    // Convenience methods for common HTTP methods

    // GET method registration
    pub fn get<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        self.route(Method::GET, pattern, MatchType::Regex, handler)
    }

    // POST method registration
    pub fn post<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        self.route(Method::POST, pattern, MatchType::Regex, handler)
    }

    // PUT method registration
    pub fn put<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        self.route(Method::PUT, pattern, MatchType::Regex, handler)
    }
    
    // DELETE method registration
    pub fn delete<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        self.route(Method::DELETE, pattern, MatchType::Regex, handler)
    }
    
    // PATCH method registration
    pub fn patch<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, RouterError>
    where   
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        self.route(Method::PATCH, pattern, MatchType::Regex, handler)
    }
    
    // HEAD method registration
    pub fn head<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        self.route(Method::HEAD, pattern, MatchType::Regex, handler)
    }

    // OPTIONS method registration
    pub fn options<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
    {
        self.route(Method::OPTIONS, pattern, MatchType::Regex, handler)
    }
    
}


