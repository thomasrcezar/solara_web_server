use rocket::{get, routes, Config, Ignite, Rocket}; 
use rocket::http::Status;
use rocket::response::status as rocket_status;
use std::net::IpAddr;
use std::str::FromStr;
use tokio::sync::oneshot;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

// Track active connections for monitoring
static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Debug, PartialEq)]
pub enum ServerStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error(String),
}

impl fmt::Display for ServerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerStatus::Stopped => write!(f, "Stopped"),
            ServerStatus::Starting => write!(f, "Starting..."),
            ServerStatus::Running => write!(f, "Running"),
            ServerStatus::Stopping => write!(f, "Stopping..."),
            ServerStatus::Error(e) => write!(f, "Error: {}", e),
        }
    }
}

// Configuration struct to pass to the server thread
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub max_connections: usize,
    // pub protocol: String, // Add later if needed for HTTPS config
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8000,
            max_connections: 100,
        }
    }
}

#[get("/")]
fn index() -> &'static str {
    ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
    let result = "Hello, world from Solara Server!";
    ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
    result
}

#[get("/hello/<name>")]
fn hello(name: &str) -> String {
    ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
    let result = format!("Hello, {}!", name);
    ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
    result
}

// Added error response route
#[get("/error")]
fn error() -> rocket_status::Custom<String> {
    rocket_status::Custom(Status::InternalServerError, "Server error simulation".to_string())
}

// Added monitoring route
#[get("/server-status")]
fn server_status() -> String {
    let connections = ACTIVE_CONNECTIONS.load(Ordering::SeqCst);
    format!("{{\"status\": \"online\", \"active_connections\": {}}}", connections)
}

// Start server function with improved error handling
pub async fn start_server(
    config: ServerConfig,
) -> Result<Rocket<Ignite>, rocket::Error> {
    println!("Configuring server for {}:{} with max {} connections", 
        config.host, config.port, config.max_connections);

    // Attempt to parse the IP address with improved error handling
    let ip_addr = match IpAddr::from_str(&config.host) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Failed to parse host '{}': {}. Defaulting to 127.0.0.1", config.host, e);
            IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
        }
    };

    let rocket_config = Config {
        address: ip_addr,
        port: config.port,
        workers: config.max_connections.min(16), // Cap at 16 workers
        ..Config::default()
    };

    // Build and ignite the server with additional routes
    rocket::custom(rocket_config)
        .mount("/", routes![index, hello, error, server_status])
        .ignite()
        .await
}

// Improved launch function with better error handling
pub async fn launch_ignited_server(
    rocket: Rocket<Ignite>,
    shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), rocket::Error> {
    println!("Launching server at {}:{}", rocket.config().address, rocket.config().port);
    
    // Get the shutdown handle from the ignited instance
    let shutdown_handle = rocket.shutdown();
    
    // Spawn a task to listen for the signal with improved error handling
    tokio::spawn(async move {
        match shutdown_rx.await {
            Ok(_) => {
                println!("Shutdown signal received, stopping server gracefully...");
                shutdown_handle.notify();
            },
            Err(e) => {
                println!("Shutdown channel closed unexpectedly: {}", e);
                // Still attempt to shut down gracefully
                shutdown_handle.notify();
            }
        }
    });
    
    // Launch the server and map the result
    match rocket.launch().await {
        Ok(_) => {
            println!("Server shutdown completed successfully");
            Ok(())
        },
        Err(e) => {
            eprintln!("Server error during operation: {}", e);
            Err(e)
        }
    }
}

// New function to get current server stats
pub fn get_server_stats() -> ServerStats {
    ServerStats {
        active_connections: ACTIVE_CONNECTIONS.load(Ordering::SeqCst),
        uptime_seconds: 0, // You could track this with SystemTime
    }
}

// Stats structure for monitoring
#[derive(Clone, Debug)]
pub struct ServerStats {
    pub active_connections: usize,
    pub uptime_seconds: u64,
}
