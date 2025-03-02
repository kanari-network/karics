// cargo run --example user


use hyper::{Method, Response, StatusCode};
use karics::{HttpService, HttpServiceFactory, Request, Router};
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::Arc;

// User data structure
#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u64,
    name: String,
    email: String,
}

// API Service structure
struct ApiService {
    router: Arc<Router<Vec<u8>>>,
}

// Implementation of HttpService for ApiService
impl HttpService for ApiService {
    fn call(&mut self, req: Request, rsp: &mut karics::Response) -> io::Result<()> {
        let method = Method::from_bytes(req.method().as_bytes()).unwrap();
        let path = req.path();
        
        match self.router.handle(&method, path) {
            Ok(response) => {
                rsp.status_code(response.status().as_u16() as usize, "OK")
                    .header("Content-Type: application/json");
                // Fix: directly use the body since it's already Vec<u8>
                rsp.body_vec(response.body().to_vec());
                Ok(())
            }
            Err(_) => {
                rsp.status_code(404, "Not Found")
                    .header("Content-Type: application/json")
                    .body(r#"{"error": "Not Found"}"#);
                Ok(())
            }
        }
    }
}

// Factory for creating API services
struct ApiServiceFactory {
    router: Arc<Router<Vec<u8>>>,
}

impl HttpServiceFactory for ApiServiceFactory {
    type Service = ApiService;

    fn new_service(&self, _id: usize) -> Self::Service {
        ApiService {
            router: Arc::clone(&self.router),
        }
    }
}


fn get_all_users() -> Response<Vec<u8>> {
    let users = vec![
        User {
            id: 1,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
        },
    ];
    
    Response::builder()
        .status(StatusCode::OK)
        .body(serde_json::to_vec(&users).unwrap())
        .unwrap()
}

// GET /users/{id}
fn get_user_by_id(params: Vec<String>) -> Response<Vec<u8>> {
    let user = User {
        id: params[0].parse().unwrap(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .body(serde_json::to_vec(&user).unwrap())
        .unwrap()
}

fn main() -> io::Result<()> {
    // Create router
    let mut root = Router::new();

    // GET /users
    root.get("/users", |_| get_all_users()).unwrap()
        .get("/users/(\\d+)", |params| get_user_by_id(params)).unwrap();

    // Create service factory
    let factory = ApiServiceFactory {
        router: Arc::new(root),
    };

    // Start server
    let handle = factory.start("127.0.0.1:3000")?;
    println!("Server running on http://127.0.0.1:3000");
    
    // Wait for server
    handle.join().unwrap();
    Ok(())
}