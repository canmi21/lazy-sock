/* examples/demo.rs */

use lazy_sock::{Method, Response, lazy_sock};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = lazy_sock!("/tmp/lazy-sock-demo.sock");

    // Route for GET /
    server
        .route(Method::Get, "/", |_req| {
            Response::json(r#"{"message": "Hello, World!", "status": "success"}"#)
        })
        .await;

    // Route for GET /health
    server
        .route(Method::Get, "/health", |_req| {
            Response::json(r#"{"status": "healthy"}"#)
        })
        .await;

    // Route for POST /echo
    server
        .route(Method::Post, "/echo", |req| match req.body_string() {
            Ok(body) if !body.is_empty() => Response::json(&format!(r#"{{"echo": "{}"}}"#, body)),
            Ok(_) => Response::new(400).with_text("Request body is empty"),
            Err(_) => Response::new(400).with_text("Invalid UTF-8 in request body"),
        })
        .await;

    // Route for GET /html
    server
        .route(Method::Get, "/html", |_req| {
            Response::html(
                r#"
            <!DOCTYPE html>
            <html>
            <head><title>Lazy Sock Demo</title></head>
            <body><h1>Hello from Lazy Sock!</h1></body>
            </html>
        "#,
            )
        })
        .await;

    println!("Lazy Sock Demo Server Starting...");
    println!("Socket: /tmp/lazy-sock-demo.sock");
    println!("Try these commands:");
    println!("   curl --unix-socket /tmp/lazy-sock-demo.sock http://localhost/");
    println!("   curl --unix-socket /tmp/lazy-sock-demo.sock http://localhost/health");
    println!("   curl --unix-socket /tmp/lazy-sock-demo.sock http://localhost/html");
    println!(
        "   curl --unix-socket /tmp/lazy-sock-demo.sock -X POST http://localhost/echo -d 'Hello from curl!'"
    );
    println!("Press Ctrl+C to stop");
    println!();

    // Run the server
    server.run().await?;

    println!("Server stopped gracefully.");
    Ok(())
}
