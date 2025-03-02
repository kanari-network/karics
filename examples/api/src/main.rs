use std::io;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use karics::{HttpServer, Request, Response, Router};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    id: u32,
    name: String,
}

type Users = Arc<Mutex<Vec<User>>>;

fn main() -> io::Result<()> {
    let users = Arc::new(Mutex::new(Vec::new()));
    let mut router = Router::new();
    let users_clone = users.clone();

    // GET /users
    router.get("/users", move |_params| {
        let users = users_clone.lock().unwrap();
        let json = serde_json::to_vec(&*users).unwrap();
        let mut response = Response::new(BytesMut::new());
        response.header("Content-Type: application/json");
        response.body_vec(json);
        response
    });

    // GET /users/{id}
    router.get("/users/(\\d+)", move |params| {
        let id = params[0].parse::<u32>().unwrap();
        let users = users.lock().unwrap();
        
        if let Some(user) = users.iter().find(|user| user.id == id) {
            let json = serde_json::to_vec(user).unwrap();
            let mut response = Response::new(BytesMut::new());
            response.header("Content-Type: application/json");
            response.body_vec(json);
            response
        } else {
            let mut response = Response::new(BytesMut::new());
            response.status_code(404, "Not Found");
            response.body("User not found");
            response
        }
    });

    // POST /users
    let users_clone = users.clone();
    router.post("/users", move |params| {
        let body = std::str::from_utf8(params[0].as_bytes()).unwrap();
        let user: User = serde_json::from_str(body).unwrap();
        
        let mut users = users_clone.lock().unwrap();
        users.push(user);
        
        let mut response = Response::new(BytesMut::new());
        response.status_code(201, "Created");
        response.body("User created");
        response
    });

    let server = HttpServer::create(router, "127.0.0.1:3000")?;
    println!("Server running on http://127.0.0.1:3000");
    server.run()
}