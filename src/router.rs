use hyper::{Method, Response, StatusCode, header};
use regex::Regex;
use std::any::Any;
use std::io::{self, Error, ErrorKind};
use std::{collections::HashMap, sync::Arc};
use crate::{Request, Response as KaricsResponse}; // Import both Response types
use crate::HttpService;

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

pub struct ApiService {
    router: Arc<Router<Vec<u8>>>,
    context: Arc<dyn Any + Send + Sync>  // Use Any to allow type retrieval
}

impl ApiService {
    pub fn new(router: Arc<Router<Vec<u8>>>) -> Self {
        ApiService {
            router: router,
            context: Arc::new(()) as Arc<dyn Any + Send + Sync>,
        }
    }

    pub fn with_context<T: 'static + Send + Sync>(router: Arc<Router<Vec<u8>>>, context: Arc<T>) -> Self {
        ApiService {
            router,
            context: context as Arc<dyn Any + Send + Sync>,
        }
    }
    
    // Method to get the context as a specific type
    pub fn get_context<T: 'static + Send + Sync>(&self) -> Option<Arc<T>> {
        self.context.clone().downcast::<T>().ok()
    }
}


impl<ResponseBody: From<Vec<u8>>> Router<ResponseBody> {
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
        let regex_pattern = match match_type {
            MatchType::Exact => format!("^{}$", pattern),
            MatchType::Prefix => format!("^{}.*", pattern),
            MatchType::Regex => pattern.to_string(),
        };

        let regex = Regex::new(&regex_pattern)
            .map_err(|_| RouterError::InvalidPattern(pattern.to_string()))?;

        let route = Route {
            pattern: regex,
            _match_type: match_type,
            handler: Box::new(handler),
        };

        self.routes
            .entry(method)
            .or_insert_with(Vec::new)
            .push(route);

        Ok(self)
    }


    // Add handle method
    pub fn handle(&self, method: &Method, path: &str) -> Result<Response<ResponseBody>, RouterError> {
        match self.match_route(method, path) {
            Ok((handler, params)) => Ok(handler(params)),
            Err(e) => match e {
                RouterError::NotFound(_) => {
                    Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(Vec::from(r#"{"error": "Not Found"}"#).into())
                        .unwrap())
                },
                RouterError::MethodNotAllowed(_) => {
                    Ok(Response::builder()
                        .status(StatusCode::METHOD_NOT_ALLOWED)
                        .body(Vec::from(r#"{"error": "Method Not Allowed"}"#).into())
                        .unwrap())
                },
                _ => {
                    Ok(Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Vec::from(r#"{"error": "Internal Server Error"}"#).into())
                        .unwrap())
                }
            }
        }
    }


        // Add match_route method
        pub fn match_route(&self, method: &Method, path: &str) 
        -> Result<(&Box<dyn Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync>, Vec<String>), RouterError> {
        
        let routes = self.routes.get(method)
            .ok_or_else(|| RouterError::MethodNotAllowed(method.clone()))?;

        for route in routes {
            if let Some(captures) = route.pattern.captures(path) {
                let mut params = Vec::new();
                for i in 0..captures.len() {
                    params.push(captures.get(i)
                        .map_or("".to_string(), |m| m.as_str().to_string()));
                }
                return Ok((&route.handler, params));
            }
        }

        Err(RouterError::NotFound(path.to_string()))
    }

    // Add convenience method for GET with specific status code
    pub fn get_with_status<F>(&mut self, pattern: &str, status: StatusCode, handler: F) 
        -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static,
        ResponseBody: From<Vec<u8>>, // Add this bound
    {
        self.route(Method::GET, pattern, MatchType::Regex, move |params| {
            Response::builder()
                .status(status.clone()) // Use StatusCode directly
                .body(handler(params).into_body())
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Vec::from("Internal Server Error").into()) // Convert to Vec<u8> first
                        .unwrap()
                })
        })
    }

    // Add method to register multiple methods for same path
    pub fn any<F>(&mut self, methods: &[Method], pattern: &str, handler: F) 
        -> Result<&mut Self, RouterError>
    where
        F: Fn(Vec<String>) -> Response<ResponseBody> + Send + Sync + 'static + Clone,
    {
        for method in methods {
            self.route(method.clone(), pattern, MatchType::Regex, handler.clone())?;
        }
        Ok(self)
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


impl HttpService for ApiService {
    fn call(&mut self, req: Request, rsp: &mut KaricsResponse) -> io::Result<()> {
        // Parse method safely
        let method = Method::from_bytes(req.method().as_bytes())
            .map_err(|_| Error::new(ErrorKind::InvalidInput, "Invalid method"))?;

        // Route the request
        match self.router.handle(&method, req.path()) {
            Ok(response) => {
                // Set status code
                let status = response.status().as_u16() as usize;
                rsp.status_code(status, status_code_to_message(status));

                // Add standard headers
                rsp.header("Server: Karics")
                   .header("X-Content-Type-Options: nosniff")
                   .header("X-Frame-Options: DENY");

                // Add Content-Type if present
                if let Some(ct) = response.headers().get(header::CONTENT_TYPE) {
                    if let Ok(ct_str) = ct.to_str() {
                        match ct_str {
                            "application/json" => rsp.header("Content-Type: application/json"),
                            "text/plain" => rsp.header("Content-Type: text/plain"),
                            "text/html" => rsp.header("Content-Type: text/html"),
                            // Add other common content types as needed
                            _ => rsp.header("Content-Type: application/octet-stream")
                        };
                    }
                }

                // Set response body
                rsp.body_vec(response.into_body());
                Ok(())
            }

            Err(e) => {
                // Map router errors to responses
                let (status, msg) = match e {
                    RouterError::NotFound(_) => (404, "Not Found"),
                    RouterError::MethodNotAllowed(_) => (405, "Method Not Allowed"), 
                    _ => (500, "Internal Server Error")
                };
            
                // Use static strings for error messages instead of format!
                rsp.status_code(status, msg)
                   .header("Content-Type: application/json");
                
                match status {
                    404 => rsp.body(r#"{"error": "Not Found"}"#),
                    405 => rsp.body(r#"{"error": "Method Not Allowed"}"#),
                    _ => rsp.body(r#"{"error": "Internal Server Error"}"#)
                }
            
                Ok(())
            }
        }
    }
}


// Helper function for status code messages
fn status_code_to_message(code: usize) -> &'static str {
    match code {
        100 => "Continue",
        101 => "Switching Protocols",
        102 => "Processing",
        103 => "Early Hints",
        
        200 => "OK",
        201 => "Created",
        202 => "Accepted", 
        203 => "Non-Authoritative Information",
        204 => "No Content",
        205 => "Reset Content",
        206 => "Partial Content",
        207 => "Multi-Status",
        208 => "Already Reported",
        226 => "IM Used",
        
        300 => "Multiple Choices",
        301 => "Moved Permanently",
        302 => "Found",
        303 => "See Other",
        304 => "Not Modified",
        305 => "Use Proxy",
        307 => "Temporary Redirect",
        308 => "Permanent Redirect",
        
        400 => "Bad Request",
        401 => "Unauthorized",
        402 => "Payment Required",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        406 => "Not Acceptable",
        407 => "Proxy Authentication Required",
        408 => "Request Timeout",
        409 => "Conflict",
        410 => "Gone",
        411 => "Length Required",
        412 => "Precondition Failed",
        413 => "Payload Too Large",
        414 => "URI Too Long",
        415 => "Unsupported Media Type",
        416 => "Range Not Satisfiable",
        417 => "Expectation Failed",
        418 => "I'm a teapot",
        421 => "Misdirected Request",
        422 => "Unprocessable Content",
        423 => "Locked",
        424 => "Failed Dependency",
        425 => "Too Early",
        426 => "Upgrade Required",
        428 => "Precondition Required",
        429 => "Too Many Requests",
        431 => "Request Header Fields Too Large",
        451 => "Unavailable For Legal Reasons",
        
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        505 => "HTTP Version Not Supported",
        506 => "Variant Also Negotiates",
        507 => "Insufficient Storage",
        508 => "Loop Detected",
        510 => "Not Extended",
        511 => "Network Authentication Required",
        
        _ => "Unknown Status Code"
    }
}