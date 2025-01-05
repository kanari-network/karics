
use std::io;

use karics::{HttpServer, Request, Response, Router};

// API Handlers
fn get_users(_req: &Request, rsp: &mut Response) -> io::Result<()> {
    rsp.body("Get users");
    Ok(())
}

fn create_user(_req: &Request, rsp: &mut Response) -> io::Result<()> {
    rsp.body("Create user");
    Ok(())
}

fn health_check(_req: &Request, rsp: &mut Response) -> io::Result<()> {
    rsp.body("OK");
    Ok(())
}

fn main() -> io::Result<()> {
    let mut router = Router::new();
    
    // API Routes
    router.add_route("/api/users", get_users);
    router.add_route("/api/users/create", create_user);
    router.add_route("/health", health_check);

    // Start server

    println!("Server starting on http://127.0.0.1:8080");
    let server = HttpServer(router).start("127.0.0.1:8080")?;
    server.wait();
    
    Ok(())
}