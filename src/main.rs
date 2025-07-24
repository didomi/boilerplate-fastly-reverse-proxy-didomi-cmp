use fastly::http::{header, Method, StatusCode};
use fastly::{Backend, Error, Request, Response};
use log_fastly::Logger;

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    init_logger();
    
    log::info!("Processing request: {} {}", req.get_method(), req.get_path());
    
    route_request(req)
}

/// Routes incoming requests to appropriate Didomi backends.
///
/// Implements two routing strategies:
/// - /api/* routes -> didomi_api backend
/// - /sdk/* routes -> didomi_sdk backend  
/// - All other routes -> 404 Not Found
fn route_request(req: Request) -> Result<Response, Error> {
    let path = req.get_path();
    
    match req.get_method() {
        &Method::GET | &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH => {
            if path.starts_with("/api/") {
                // Route API requests to Didomi API backend
                log::info!("Routing API request to didomi_api backend: {}", path);
                proxy_to_backend(req, "didomi_api")
            } else if path.starts_with("/sdk/") {
                // Route SDK requests to Didomi SDK backend
                log::info!("Routing SDK request to didomi_sdk backend: {}", path);
                proxy_to_backend(req, "didomi_sdk")
            } else {
                // Return 404 for unmatched routes
                log::info!("No matching route found for: {}", path);
                Ok(not_found_response())
            }
        }
        _ => {
            // Return 405 Method Not Allowed for unsupported methods
            log::info!("Method not allowed: {}", req.get_method());
            Ok(method_not_allowed_response())
        }
    }
}

/// Proxies a request to the specified backend.
fn proxy_to_backend(mut req: Request, backend_name: &str) -> Result<Response, Error> {
    // Set up backend
    let backend = Backend::from_name(backend_name)?;
    
    // Add CORS headers for preflight requests
    if req.get_method() == &Method::OPTIONS {
        return Ok(cors_preflight_response());
    }
    
    // Forward important headers
    let user_agent = req.get_header_str("User-Agent").unwrap_or("Fastly-Proxy/1.0").to_string();
    req.set_header("Host", get_backend_host(backend_name));
    req.set_header("User-Agent", &user_agent);
    
    // Send request to backend
    let mut resp = req.send(backend)?;
    
    // Add CORS headers to response
    resp.set_header("Access-Control-Allow-Origin", "*");
    resp.set_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS");
    resp.set_header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With");
    resp.set_header("Access-Control-Max-Age", "86400");
    
    // Set appropriate cache headers
    match backend_name {
        "didomi_sdk" => {
            // SDK resources can be cached for 1 hour
            resp.set_header("Cache-Control", "public, max-age=3600");
        }
        "didomi_api" => {
            // API responses should not be cached
            resp.set_header("Cache-Control", "no-cache, no-store, must-revalidate");
            resp.set_header("Pragma", "no-cache");
            resp.set_header("Expires", "0");
        }
        _ => {}
    }
    
    log::info!("Successfully proxied request to {} backend", backend_name);
    Ok(resp)
}

/// Returns the appropriate host header for the backend.
fn get_backend_host(backend_name: &str) -> &'static str {
    match backend_name {
        "didomi_sdk" => "sdk.privacy-center.org",
        "didomi_api" => "api.privacy-center.org",
        _ => "unknown"
    }
}

/// Creates a CORS preflight response.
fn cors_preflight_response() -> Response {
    Response::from_status(StatusCode::OK)
        .with_header("Access-Control-Allow-Origin", "*")
        .with_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, PATCH, OPTIONS")
        .with_header("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Requested-With")
        .with_header("Access-Control-Max-Age", "86400")
        .with_header("Content-Length", "0")
}

/// Creates a standard 404 Not Found response.
fn not_found_response() -> Response {
    Response::from_status(StatusCode::NOT_FOUND)
        .with_body("Not Found - Only /api/* and /sdk/* routes are supported")
        .with_header(header::CONTENT_TYPE, "text/plain")
        .with_header("Access-Control-Allow-Origin", "*")
}

/// Creates a 405 Method Not Allowed response.
fn method_not_allowed_response() -> Response {
    Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
        .with_body("Method Not Allowed")
        .with_header(header::CONTENT_TYPE, "text/plain")
        .with_header("Allow", "GET, POST, PUT, DELETE, PATCH, OPTIONS")
        .with_header("Access-Control-Allow-Origin", "*")
}

/// Initializes the logger for debugging and monitoring.
fn init_logger() {
    let logger = Logger::builder()
        .default_endpoint("cmp_proxy_log")
        .echo_stdout(true)
        .max_level(log::LevelFilter::Info)
        .build()
        .expect("Failed to build Logger");

    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                message
            ))
        })
        .chain(Box::new(logger) as Box<dyn log::Log>)
        .apply()
        .expect("Failed to initialize logger");
