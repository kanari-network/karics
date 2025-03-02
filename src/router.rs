use hyper::{Method, Response, StatusCode};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug)]
pub enum RouterError {
    InvalidPath,
    MethodNotAllowed(Method),
    NotFound(String),
}

pub struct Route<T> {
    pattern: Regex,
    handler: Box<dyn Fn(Vec<String>) -> Response<T> + Send + Sync>,
}

pub struct Router<T> {
    routes: HashMap<Method, Vec<Route<T>>>,
}

impl<T> Router<T> {
    pub fn new() -> Self {
        Router {
            routes: HashMap::with_capacity(32), // Pre-allocate space
        }
    }

    // Advanced route registration with method chaining
    pub fn route<F>(&mut self, method: Method, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Vec<String>) -> Response<T> + Send + Sync + 'static,
    {
        let route = Route {
            pattern: Regex::new(&format!("^{}$", pattern)).unwrap(),
            handler: Box::new(handler),
        };
        self.routes
            .entry(method)
            .or_insert_with(|| Vec::with_capacity(16))
            .push(route);
        self
    }

    // Optimized request handling with early returns
    pub fn handle(&self, method: &Method, path: &str) -> Result<Response<T>, RouterError> {
        // Validate path
        if path.is_empty() {
            return Err(RouterError::InvalidPath);
        }

        // Get routes for method or return early
        let routes = self
            .routes
            .get(method)
            .ok_or(RouterError::MethodNotAllowed(method.clone()))?;

        // Find matching route
        for route in routes {
            if let Some(captures) = route.pattern.captures(path) {
                let params: Vec<String> = captures
                    .iter()
                    .skip(1)
                    .filter_map(|c| c.map(|m| m.as_str().to_string()))
                    .collect();
                return Ok((route.handler)(params));
            }
        }

        Err(RouterError::NotFound(path.to_string()))
    }

    // Convert RouterError to Response
    pub fn error_to_response(error: RouterError) -> Response<T>
    where
        T: Default,
    {
        match error {
            RouterError::NotFound(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(T::default())
                .unwrap(),
            RouterError::MethodNotAllowed(_) => Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(T::default())
                .unwrap(),
            RouterError::InvalidPath => Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(T::default())
                .unwrap(),
        }
    }

    // Common HTTP method handlers
    pub fn get<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Vec<String>) -> Response<T> + Send + Sync + 'static,
    {
        self.route(Method::GET, pattern, handler)
    }

    pub fn post<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Vec<String>) -> Response<T> + Send + Sync + 'static,
    {
        self.route(Method::POST, pattern, handler)
    }

    pub fn put<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Vec<String>) -> Response<T> + Send + Sync + 'static,
    {
        self.route(Method::PUT, pattern, handler)
    }

    pub fn delete<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(Vec<String>) -> Response<T> + Send + Sync + 'static,
    {
        self.route(Method::DELETE, pattern, handler)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Method, StatusCode};

    #[test]
    fn test_router_basic_routes() {
        let mut router = Router::<String>::new();
        
        router.get("/test", |_| {
            Response::builder()
                .status(StatusCode::OK)
                .body("GET".to_string())
                .unwrap()
        });

        router.post("/test", |_| {
            Response::builder()
                .status(StatusCode::OK) 
                .body("POST".to_string())
                .unwrap()
        });

        let get_response = router.handle(&Method::GET, "/test")
            .expect("GET request should succeed");
        assert_eq!(get_response.status(), StatusCode::OK);
        
        let post_response = router.handle(&Method::POST, "/test")
            .expect("POST request should succeed");
        assert_eq!(post_response.status(), StatusCode::OK);

        let not_found = router.handle(&Method::GET, "/notfound")
            .expect("Not found should return response");
        assert_eq!(not_found.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_route_params() {
        let mut router = Router::<String>::new();

        router.get("/users/(\\d+)", |params| {
            Response::builder()
                .status(StatusCode::OK)
                .body(params[0].clone())
                .unwrap()
        });

        let response = router.handle(&Method::GET, "/users/123")
            .expect("Parameter route should match");
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn test_invalid_requests() {
        let router = Router::<String>::new();

        let empty_path = router.handle(&Method::GET, "")
            .expect("Empty path should return response");
        assert_eq!(empty_path.status(), StatusCode::BAD_REQUEST);

        let bad_method = router.handle(&Method::PUT, "/test")
            .expect("Invalid method should return response");
        assert_eq!(bad_method.status(), StatusCode::METHOD_NOT_ALLOWED);
    }
}
