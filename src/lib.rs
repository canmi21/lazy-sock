/* src/lib.rs */

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
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

/// 回调函数类型定义
pub type HandlerFn = Arc<dyn Fn(Request) -> Response + Send + Sync>;

/// 日志回调函数类型
pub type LogCallbackFn = Arc<dyn Fn(&str) + Send + Sync>;

/// 提示回调函数类型
pub type PromptCallbackFn = Arc<dyn Fn(&str) + Send + Sync>;

/// LazySock 服务器主结构
pub struct LazySock {
    socket_path: PathBuf,
    router: Arc<RwLock<Router>>,
    log_callback: Option<LogCallbackFn>,
    prompt_callback: Option<PromptCallbackFn>,
    cleanup_on_exit: bool,
}

impl LazySock {
    /// 创建新的 LazySock 实例
    pub fn new<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            router: Arc::new(RwLock::new(Router::new())),
            log_callback: None,
            prompt_callback: None,
            cleanup_on_exit: true,
        }
    }

    /// 设置日志回调函数
    pub fn with_log_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.log_callback = Some(Arc::new(callback));
        self
    }

    /// 设置提示回调函数
    pub fn with_prompt_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.prompt_callback = Some(Arc::new(callback));
        self
    }

    /// 设置是否在退出时清理socket文件
    pub fn with_cleanup_on_exit(mut self, cleanup: bool) -> Self {
        self.cleanup_on_exit = cleanup;
        self
    }

    /// 注册路由处理函数
    pub async fn route<F>(&self, method: Method, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let mut router = self.router.write().await;
        router.add_route(method, path, Arc::new(handler));
    }

    /// 启动服务器
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        // 检查socket文件是否存在
        if let Err(e) = self.check_and_handle_existing_socket().await {
            return Err(e);
        }

        // 创建Unix socket监听器
        let listener = UnixListener::bind(&self.socket_path)?;
        self.log(&format!("Server started on socket: {:?}", self.socket_path));

        // 设置信号处理器用于优雅关闭
        let socket_path_for_cleanup = self.socket_path.clone();
        let cleanup_on_exit = self.cleanup_on_exit;
        let mut cleanup_task = tokio::spawn(async move {
            if let Ok(()) = signal::ctrl_c().await {
                if cleanup_on_exit {
                    let _ = fs::remove_file(&socket_path_for_cleanup).await;
                }
            }
        });

        // 主服务循环
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
                _ = &mut cleanup_task => {
                    self.log("Server shutting down...");
                    break;
                }
            }
        }

        Ok(())
    }

    /// 检查并处理已存在的socket文件
    async fn check_and_handle_existing_socket(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.socket_path.exists() {
            self.prompt(
                "Socket file already exists. Will override in 3 seconds... (Ctrl+C to abort now)",
            );

            // 等待3秒，期间可以被Ctrl+C中断
            tokio::select! {
                _ = sleep(Duration::from_secs(3)) => {
                    fs::remove_file(&self.socket_path).await?;
                    self.log("Removed existing socket file");
                }
                _ = signal::ctrl_c() => {
                    self.prompt("Aborted by user");
                    return Err("User aborted".into());
                }
            }
        }

        Ok(())
    }

    /// 记录日志
    fn log(&self, message: &str) {
        if let Some(callback) = &self.log_callback {
            callback(message);
        }
    }

    /// 显示提示信息
    fn prompt(&self, message: &str) {
        if let Some(callback) = &self.prompt_callback {
            callback(message);
        }
    }
}

/// 处理单个连接
async fn handle_connection(
    mut stream: UnixStream,
    router: Arc<RwLock<Router>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut reader = BufReader::new(&mut stream);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    // 解析HTTP请求行
    let parts: Vec<&str> = request_line.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid request line".into());
    }

    let method = match parts[0] {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        _ => return Err("Unsupported method".into()),
    };

    let path = parts[1].to_string();

    // 读取剩余的头部（简单实现，跳过）
    let headers = HashMap::new();
    let mut line = String::new();
    while reader.read_line(&mut line).await? > 0 {
        if line.trim().is_empty() {
            break;
        }
        // 这里可以解析头部，简单起见暂时跳过
        line.clear();
    }

    // 创建请求对象
    let request = Request::new(method.clone(), path.clone(), headers, Vec::new());

    // 路由处理
    let router_guard = router.read().await;
    let response = if let Some(handler) = router_guard.find_handler(&method, &path) {
        handler(request)
    } else {
        Response::not_found("Route not found")
    };

    // 发送响应
    let response_data = response.to_http_response();
    stream.write_all(response_data.as_bytes()).await?;
    stream.flush().await?;

    Ok(())
}

/// 便捷宏用于快速创建服务器
#[macro_export]
macro_rules! lazy_sock {
    ($path:expr) => {
        $crate::LazySock::new($path)
            .with_log_callback(|msg| println!("{}", msg))
            .with_prompt_callback(|msg| println!("{}", msg))
    };
}
