# LazySock

**LazySock** is a lightweight Rust library for building Unix Domain Socket services with minimal boilerplate. It provides a simple macro to create `.sock` files (defaulting to `/tmp`, no root privileges required, with automatic cleanup on system reboot). Initialize a socket path, register routes, bind handlers, and you're ready to send `curl` requests for JSON responses.

## Features
- **Zero Boilerplate**: Use the `lazy_sock!` macro to quickly set up a Unix Domain Socket server.
- **Simple Routing**: Register routes for HTTP-like methods (`GET`, `POST`, `PUT`, `DELETE`) with ease.
- **Flexible Responses**: Support for JSON, HTML, plain text, and binary responses.
- **Asynchronous**: Built on top of Tokio for high-performance async I/O.
- **Lightweight**: Minimal dependencies and straightforward API.
- **Customizable**: Configure logging and cleanup behavior as needed.

## Installation

Add LazySock to your project by including it in your `Cargo.toml`:

```toml
[dependencies]
lazy-sock = "1"
```

Ensure you have the following dependencies in your `Cargo.toml` as well:

```toml
tokio = { version = "1", features = ["full"] }
url = "2"
```

## Usage

### Basic Example

The following example demonstrates how to create a simple Unix Domain Socket server with multiple routes:

<xaiArtifactInner>

```rust
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
```

</xaiArtifactInner>

### Running the Example

1. Save the above code in `examples/demo.rs`.
2. Run the example using:
   ```bash
   cargo run --example demo
   ```
3. Test the server with `curl` commands, for example:
   ```bash
   curl --unix-socket /tmp/lazy-sock-demo.sock http://localhost/
   curl --unix-socket /tmp/lazy-sock-demo.sock http://localhost/health
   curl --unix-socket /tmp/lazy-sock-demo.sock -X POST http://localhost/echo -d 'Hello from curl!'
   ```

### API Overview

- **LazySock::new(socket_path)**: Creates a new LazySock instance with the specified socket path.
- **lazy_sock!(socket_path)**: A macro to create a LazySock instance with default logging and prompt callbacks.
- **route(method, path, handler)**: Registers a route for a specific HTTP method and path with a handler function.
- **run()**: Starts the server and listens for incoming connections.
- **Response**: Supports `json`, `html`, `text`, and `binary` response types with automatic `Content-Type` and `Content-Length` headers.
- **Request**: Provides methods to access the request method, path, headers, body, and query parameters.

### Customization

- **Logging**: Use `with_log_callback` to set a custom logging function.
- **Prompts**: Use `with_prompt_callback` to handle prompt messages (e.g., for existing socket file warnings).
- **Socket Cleanup**: Use `with_cleanup_on_exit` to control whether the socket file is removed on server shutdown (default: `true`).

## Project Structure

```
lazy-sock/
├── examples/
│   └── demo.rs         # Example server implementation
├── src/
│   ├── lib.rs         # Main library code
│   ├── request.rs     # Request struct and methods
│   ├── response.rs    # Response struct and methods
│   └── router.rs      # Router and HTTP method definitions
├── Cargo.toml         # Project configuration
└── README.md          # This file
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please submit issues or pull requests to the [GitHub repository](https://github.com/canmi21/lazy-sock).

## Contact

For questions or feedback, please open an issue on the [GitHub repository](https://github.com/canmi21/lazy-sock).