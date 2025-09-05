/* src/lib.rs */

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal;
use tokio::sync::RwLock;
use tokio::time::{Duration, sleep};

mod request;
mod response;
mod router;

pub use request::Request;
pub use response::Response;
pub use router::{Method, Router};

/// Type alias for the handler function.
pub type HandlerFn = Arc<dyn Fn(Request) -> Response + Send + Sync>;

/// Type alias for the log callback function.
pub type LogCallbackFn = Arc<dyn Fn(&str) + Send + Sync>;

/// Type alias for the prompt callback function.
pub type PromptCallbackFn = Arc<dyn Fn(&str) + Send + Sync>;

/// The main LazySock server struct.
pub struct LazySock {
    socket_path: PathBuf,
    router: Arc<RwLock<Router>>,
    log_callback: Option<LogCallbackFn>,
    prompt_callback: Option<PromptCallbackFn>,
    cleanup_on_exit: bool,
}

impl LazySock {
    /// Creates a new LazySock server instance.
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            router: Arc::new(RwLock::new(Router::new())),
            log_callback: None,
            prompt_callback: None,
            cleanup_on_exit: true,
        }
    }

    /// Sets a custom log callback function.
    pub fn with_log_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.log_callback = Some(Arc::new(callback));
        self
    }

    /// Sets a custom prompt callback function.
    pub fn with_prompt_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.prompt_callback = Some(Arc::new(callback));
        self
    }

    /// Configures whether to clean up the socket file on exit.
    pub fn with_cleanup_on_exit(mut self, cleanup: bool) -> Self {
        self.cleanup_on_exit = cleanup;
        self
    }

    /// Registers a handler for a specific method and path.
    pub async fn route<F>(&self, method: Method, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut router = self.router.write().await;
        router.add_route(method, path, Arc::new(handler));
    }

    /// Starts the server and listens for incoming connections.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        if let Err(e) = self.check_and_handle_existing_socket().await {
            return Err(e);
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        self.log(&format!("Server started on socket: {:?}", self.socket_path));

        let socket_path_for_cleanup = self.socket_path.clone();
        let cleanup_on_exit = self.cleanup_on_exit;

        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, _)) => {
                            let router = Arc::clone(&self.router);
                            let log_callback = self.log_callback.clone();
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, router).await {
                                    if let Some(logger) = log_callback {
                                        logger(&format!("Error handling connection: {}", e));
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            self.log(&format!("Error accepting connection: {}", e));
                        }
                    }
                }
                _ = signal::ctrl_c() => {
                    self.log("Server shutting down...");
                    if cleanup_on_exit {
                        let _ = fs::remove_file(&socket_path_for_cleanup).await;
                        self.log(&format!("Cleaned up socket file: {:?}", socket_path_for_cleanup));
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    /// Checks for an existing socket file and handles it.
    async fn check_and_handle_existing_socket(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.socket_path.exists() {
            self.prompt(
                "Socket file already exists. Will override in 3 seconds... (Ctrl+C to abort now)",
            );

            tokio::select! {
                _ = sleep(Duration::from_secs(3)) => {
                    fs::remove_file(&self.socket_path).await?;
                    self.log("Removed existing socket file.");
                }
                _ = signal::ctrl_c() => {
                    self.prompt("Aborted by user.");
                    return Err("User aborted".into());
                }
            }
        }
        Ok(())
    }

    /// Logs a message using the configured callback.
    fn log(&self, message: &str) {
        if let Some(callback) = &self.log_callback {
            callback(message);
        }
    }

    /// Shows a prompt message using the configured callback.
    fn prompt(&self, message: &str) {
        if let Some(callback) = &self.prompt_callback {
            callback(message);
        }
    }
}

/// Handles a single incoming client connection.
async fn handle_connection(
    mut stream: UnixStream,
    router: Arc<RwLock<Router>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid request line".into());
    }

    let method = Method::from_str(parts[0]).ok_or("Unsupported HTTP method")?;
    let path = parts[1].to_string();

    let mut headers = HashMap::new();
    let mut line = String::new();
    loop {
        reader.read_line(&mut line).await?;
        if line.trim().is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_string(), value.trim().to_string());
        }
        line.clear();
    }

    let mut body = Vec::new();
    if let Some(content_length_str) = headers.get("Content-Length") {
        if let Ok(content_length) = content_length_str.parse::<usize>() {
            if content_length > 0 {
                body.resize(content_length, 0);
                reader.read_exact(&mut body).await?;
            }
        }
    }

    let request = Request::new(method.clone(), path, headers, body);
    let router_guard = router.read().await;

    let response =
        if let Some(handler) = router_guard.find_handler(&method, request.path_without_query()) {
            handler(request)
        } else {
            Response::not_found("Route not found")
        };

    let response_data = response.to_http_response();
    stream.write_all(response_data.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}

/// A convenient macro to quickly create a server instance using `fancy-log`.
#[macro_export]
macro_rules! lazy_sock {
    ($path:expr) => {
        $crate::LazySock::new($path)
            .with_log_callback(|msg| fancy_log::log(fancy_log::LogLevel::Info, msg))
            .with_prompt_callback(|msg| fancy_log::log(fancy_log::LogLevel::Info, msg))
    };
}
