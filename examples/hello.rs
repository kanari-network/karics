use hyper::{Response, StatusCode, header};
use karics::router::ApiService;
use karics::HttpServiceFactory;
use serde::{Deserialize, Serialize};
use std::io;
use std::sync::{Arc, Mutex};

// Define User struct
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: usize,
    name: String,
    email: String,
}

// Factory for creating API services
struct ApiServiceFactory {
    router: Arc<karics::router::Router<Vec<u8>>>,
    users: Arc<Mutex<Vec<User>>>,
}

impl HttpServiceFactory for ApiServiceFactory {
    type Service = ApiService;

    fn new_service(&self, _id: usize) -> Self::Service {
        ApiService::with_context(self.router.clone(), self.users.clone())
    }
}

// GET /users
fn get_all_users(
    users: Arc<Mutex<Vec<User>>>,
) -> impl Fn(Vec<String>) -> Response<Vec<u8>> + Clone {
    move |_params| {
        let users_guard = users.lock().unwrap();
        let users_json = serde_json::to_vec(&*users_guard).unwrap_or_else(|_| b"[]".to_vec());

        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "application/json")
            .body(users_json)
            .unwrap()
    }
}

// GET /users/:id
fn get_user_by_id(
    users: Arc<Mutex<Vec<User>>>,
) -> impl Fn(Vec<String>) -> Response<Vec<u8>> + Clone {
    move |params| {
        let user_id = params
            .get(1)
            .and_then(|id| id.parse::<usize>().ok())
            .unwrap_or(0);

        let users_guard = users.lock().unwrap();

        match users_guard.iter().find(|user| user.id == user_id) {
            Some(user) => {
                let user_json = serde_json::to_vec(user).unwrap();
                Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(user_json)
                    .unwrap()
            }
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "application/json")
                .body(r#"{"error":"User not found"}"#.as_bytes().to_vec())
                .unwrap(),
        }
    }
}

// POST /users
fn create_user(users: Arc<Mutex<Vec<User>>>) -> impl Fn(Vec<String>) -> Response<Vec<u8>> + Clone {
    move |_params| {
        // In a real app, parse the request body here
        let mut users_guard = users.lock().unwrap();

        // Generate a new ID
        let new_id = users_guard.len() + 1;

        // Create a new user
        let new_user = User {
            id: new_id,
            name: format!("User {}", new_id),
            email: format!("user{}@example.com", new_id),
        };

        // Add the user to the store
        users_guard.push(new_user.clone());

        // Return the created user
        let user_json = serde_json::to_vec(&new_user).unwrap();

        Response::builder()
            .status(StatusCode::CREATED)
            .header(header::CONTENT_TYPE, "application/json")
            .body(user_json)
            .unwrap()
    }
}

fn main() -> io::Result<()> {
    // Create a shared user store
    let users = Arc::new(Mutex::new(Vec::<User>::new()));

    // Create router
    let mut router = karics::router::Router::new();

    // Register routes
    router
        .get(r"^/users$", get_all_users(users.clone()))
        .unwrap();
    router
        .get(r"^/users/(\d+)$", get_user_by_id(users.clone()))
        .unwrap();
    router
        .post(r"^/users$", create_user(users.clone()))
        .unwrap();

    // Create service factory
    let factory = ApiServiceFactory {
        router: Arc::new(router),
        users,
    };

    // Start server
    println!("Server running on http://127.0.0.1:8080");
    let handle = factory.start("127.0.0.1:8080")?;

    // Wait for server
    handle.join().unwrap();
    Ok(())
}
