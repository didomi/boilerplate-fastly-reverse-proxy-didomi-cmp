# Didomi reverse proxy - Fastly setup guide

This guide explains how to configure Fastly Compute@Edge to create a reverse proxy that serves the Didomi Consent notice from your own domain. Two implementation options are available based on your path requirements:

## Choose Your Implementation

### Option A: Direct path routing

- **Customer Usage**: `/api/*` and `/sdk/*` paths directly
- **Architecture**: Fastly with minimal WASM processing
- **Implementation**: Simple backend routing with lightweight Rust code

### Option B: Consent path routing

- **Customer Usage**: `/consent/*` prefix for all CMP requests
- **Architecture**: Fastly Compute@Edge with full URL transformation
- **Implementation**: URL transformation and advanced processing

### Domain vs Subdomain Trade-offs

<aside>
⚠️

Only *Consent path routing* option works with either setup. *Direct path routing* option, which occupies the main path, works only with a subdomain dedicated to the Didomi CMP.

</aside>

## Architecture overview

## Common prerequisites (Both options)

- Fastly account with Compute@Edge enabled
- Rust toolchain with WebAssembly support
- `fastly` CLI tool installed ([installing and configuring Fastly CLI](https://www.fastly.com/documentation/reference/tools/cli/))
- Domain configured for Fastly service
- Access to your domain's DNS configuration

## Shared setup steps (both options)

### Domain and DNS configuration

### 1. Domain setup in Fastly

### Add domains

1. **Log into Fastly Dashboard**
2. **Navigate to**: Configure → Domains
3. **Add Domains**: Enter both `YOUR_DOMAIN_NAME` and `www.YOUR_DOMAIN_NAME`

### 2. DNS configuration

### For root domain (A record)

```bash
# Configure A records pointing to Fastly IP addresses
# Get current Fastly IP addresses from: https://docs.fastly.com/en/guides/accessing-fastlys-ip-ranges

# Example A records (verify current IPs with Fastly):
YOUR_DOMAIN_NAME.     300    IN    A    151.101.1.140
YOUR_DOMAIN_NAME.     300    IN    A    151.101.65.140
YOUR_DOMAIN_NAME.     300    IN    A    151.101.129.140
YOUR_DOMAIN_NAME.     300    IN    A    151.101.193.140
```

### For subdomain (CNAME Record)

```bash
# Configure CNAME record for www subdomain
www.YOUR_DOMAIN_NAME.     300    IN    CNAME    YOUR_DOMAIN_NAME.
```

### 3. TLS Certificate configuration


### Certificate subscription setup

1. **Navigate to**: TLS Configuration → Certificates
2. **Create new subscription**
3. **Configure subscription**:
    - **Domains**: Enter `YOUR_DOMAIN_NAME, www.YOUR_DOMAIN_NAME` (comma-separated)
    - **Common Name**: `YOUR_DOMAIN_NAME`
    - **Certification Authority**: Let's Encrypt
    - **TLS Configuration**: `HTTP/3 & TLS v1.3 + 0RTT (t.sni)`

### ACME challenge configuration

After submitting the certificate request, Fastly will provide an ACME challenge:

1. **Create DNS CNAME record**:
    
    ```bash
    # Example provided by Fastly (replace with your actual values)
    _acme-challenge.YOUR_DOMAIN_NAME    CNAME    YOUR_CHALLENGE_TOKEN.fastly-validations.com
    ```
    
2. **Verify DNS propagation**:
    
    ```bash
    # Verify the ACME challenge CNAME is propagated
    dig _acme-challenge.YOUR_DOMAIN_NAME CNAME +short
    # Should return: YOUR_CHALLENGE_TOKEN.fastly-validations.com
    ```
    
3. **Certificate validation**: Fastly will automatically validate domain ownership and issue the certificate

### Fastly service configuration


### 1. Create Fastly service

Create a new Compute@Edge service in Fastly dashboard or via CLI:

```bash
fastly compute init --from=https://github.com/fastly/compute-starter-kit-rust-default
```

### 2. Configure backends (Both options use same backends)

In the Fastly dashboard, configure two backends:

**Backend 1: Didomi SDK**

- **Name**: `didomi_sdk`
- **Address**: `sdk.privacy-center.org`
- **Port**: `443` (HTTPS)
- **Host Header**: `sdk.privacy-center.org`
- **Override Host**: Yes
- **Use SSL**: Yes
- **SSL SNI Hostname**: `sdk.privacy-center.org`
- **SSL Certificate Hostname**: `sdk.privacy-center.org`

**Backend 2: Didomi API**

- **Name**: `didomi_api`
- **Address**: `api.privacy-center.org`
- **Port**: `443` (HTTPS)
- **Host Header**: `api.privacy-center.org`
- **Override Host**: Yes
- **Use SSL**: Yes
- **SSL SNI Hostname**: `api.privacy-center.org`
- **SSL Certificate Hostname**: `api.privacy-center.org`

---

## Option A: Direct path routing implementation

This option uses simple direct routing with minimal WASM code.

### Step 1: Create [fastly.toml](https://github.com/didomi/boilerplate-fastly-reverse-proxy-didomi-cmp/blob/main/fastly.toml)

```toml
# This file describes a Fastly Compute package. To learn more visit:
# https://www.fastly.com/documentation/reference/compute/fastly-toml

authors = ["didomi-team"]
description = "Boilerplate Fastly Reverse Proxy for Didomi CMP"
language = "rust"
manifest_version = 3
name = "boilerplate-fastly-reverse-proxy-didomi-cmp"
service_id = "{{YOUR_SERVICE_ID}}"

[local_server]

  [local_server.backends]

    [local_server.backends.didomi_api]
      url = "https://api.privacy-center.org"

    [local_server.backends.didomi_sdk]
      url = "https://sdk.privacy-center.org"

[scripts]
  build = "    cargo build --bin boilerplate-fastly-reverse-proxy-didomi-cmp --release --target wasm32-wasip1 --color always\n"

```

### Step 2: Create [Cargo.toml](https://github.com/didomi/boilerplate-fastly-reverse-proxy-didomi-cmp/blob/main/Cargo.toml)

```toml
[package]
name = "boilerplate-fastly-reverse-proxy-didomi-cmp"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = "0.4.41"
fastly = "0.11.5"
fern = "0.7.1"
log = "0.4"
log-fastly = "0.11.5"

```

### Step 3: Create simple implementation ([src/main_simplified.rs](https://github.com/didomi/boilerplate-fastly-reverse-proxy-didomi-cmp/blob/main/src/main_simplified.rs))

```rust
use fastly::http::{header, Method, StatusCode};
use fastly::{Backend, Error, Request, Response};

mod template;

#[fastly::main]
fn main(req: Request) -> Result<Response, Error> {
    let path = req.get_path();
    
    match req.get_method() {
        &Method::GET | &Method::POST => {
            if path.starts_with("/api/") {
                proxy_to_backend(req, "didomi_api")
            } else if path.starts_with("/sdk/") {
                proxy_to_backend(req, "didomi_sdk")
            } else if path == "/" {
                // Serve UI template for root path
                Ok(Response::from_status(StatusCode::OK)
                    .with_body(template::HTML_TEMPLATE)
                    .with_header(header::CONTENT_TYPE, "text/html"))
            } else {
                Ok(Response::from_status(StatusCode::NOT_FOUND)
                    .with_body("Not Found"))
            }
        }
        _ => Ok(Response::from_status(StatusCode::METHOD_NOT_ALLOWED)
            .with_body("Method Not allowed"))
    }
}

fn proxy_to_backend(mut req: Request, backend_name: &str) -> Result<Response, Error> {
    let backend = Backend::from_name(backend_name)?;
    let mut resp = req.send(backend)?;
    
    // Add CORS headers
    resp.set_header("Access-Control-Allow-Origin", "*");
    
    Ok(resp)
}

```

### Step 4: Deploy option A

```bash
# Build and deploy
cargo build --release --target wasm32-wasip1
fastly compute publish
```

### Step 5: Test option A

```bash
# Test SDK route
curl https://YOUR_OWN_DOMAIN/sdk/YOUR_API_KEY/loader.js

# Test API route
curl https://YOUR_OWN_DOMAIN/api/events
```

---

## Option B: Consent path routing implementation

This option includes URL transformation to handle `/consent/*` prefixes.

**Prerequisites:** Use the same backends and DNS setup as Option A above.

### Step 1: Create [fastly.toml](https://github.com/didomi/boilerplate-fastly-reverse-proxy-didomi-cmp/blob/main/fastly.toml) for Option B

```toml
# This file describes a Fastly Compute package. To learn more visit:
# https://docs.fastly.com/en/guides/compute-configuration

authors = ["YOUR_EMAIL@YOUR_DOMAIN_NAME"]
description = "Didomi CMP Consent Path Routing"
language = "rust"
manifest_version = 3
name = "boilerplate-fastly-reverse-proxy-didomi-cmp"

[build]
rust_target = "wasm32-wasip1"

[local_server]

  [local_server.backends]

    [local_server.backends.didomi_sdk]
    url = "https://sdk.privacy-center.org"

    [local_server.backends.didomi_api]
    url = "https://api.privacy-center.org"
```

### Step 2: Create [Cargo.toml](https://github.com/didomi/boilerplate-fastly-reverse-proxy-didomi-cmp/blob/main/Cargo.toml) for Option B

```toml
[package]
name = "boilerplate-fastly-reverse-proxy-didomi-cmp"
version = "0.1.0"
edition = "2021"

[dependencies]
chrono = "0.4.41"
fastly = "0.11.5"
fern = "0.7.1"
log = "0.4"
log-fastly = "0.11.5"
```

### 3. Main implementation ([src/main.rs](https://github.com/didomi/boilerplate-fastly-reverse-proxy-didomi-cmp/blob/main/src/main.rs))

```rust
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
}
```

## Deployment steps

### 1. Build the application

```bash
# Build for WebAssembly target
cargo build --target wasm32-wasip1 --release
```

### 2. Test locally

```bash
# Start local development server
fastly compute serve

# Test CMP endpoints (replace YOUR_API_KEY and NOTICE_ID with actual values, e.g., YOUR_API_KEY=24cd3901-9da4-4643-96a3-9b1c573b5264, NOTICE_ID=J3nR2TTU)
curl http://127.0.0.1:7676/consent/YOUR_API_KEY/loader.js?target_type=notice&target=NOTICE_ID
```

### 3. Deploy to Fastly

```bash
# Deploy to production
fastly compute publish

# Verify deployment
curl https://YOUR_DOMAIN_NAME/consent/YOUR_API_KEY/loader.js?target_type=notice&target=NOTICE_ID
```

## Configuration requirements

### DNS configuration

Point your domain/subdomain to Fastly:

- Create a CNAME record pointing to your Fastly service domain
- Or configure A records to Fastly IP addresses

### SSL/TLS setup

1. **Upload SSL Certificate** to Fastly (if using custom domain)
2. **Enable TLS** for both backends
3. **Configure SNI** for proper SSL handshake

### Headers and caching

### For SDK resources (`/consent/*`):

- **Cache TTL**: 3600 seconds (1 hour)
- **Vary Header**: Accept-Encoding, Accept-Language
- **CORS**: Enabled for cross-origin requests

### For API endpoints (`/consent/api/*`):

- **Cache TTL**: 0 (no caching)
- **Cache-Control**: no-cache, no-store, must-revalidate
- **CORS**: Enabled with appropriate headers

## Testing verification

### 1. CMP functionality test

```bash
# Test CMP loader (replace YOUR_API_KEY and NOTICE_ID with actual values, e.g., YOUR_API_KEY=24cd3901-9da4-4643-96a3-9b1c573b5264, NOTICE_ID=J3nR2TTU)
curl "https://YOUR_DOMAIN_NAME/consent/YOUR_API_KEY/loader.js?target_type=notice&target=NOTICE_ID"
```

This setup provides a production-ready CMP reverse proxy.

# Production Setup vs. Test Configuration

In a realistic production environment, the UI would be hosted on a separate CDN like CloudFront/S3 or similar static hosting service or even in Fastly in a separate project. Fastly would be configured with a backend pointing to your UI hosting service (e.g., ui_backend → [https://your-ui.cloudfront.net](https://your-ui.cloudfront.net/)), and the root path / would route to this backend instead of serving embedded HTML.

For testing purposes, this [boilerplate](https://github.com/didomi/boilerplate-fastly-reverse-proxy-didomi-cmp) includes an embedded HTML template with the Didomi consent notice integration. This allows you to quickly test the consent management functionality without setting up separate UI infrastructure. The template includes the necessary Didomi SDK snippets and demonstrates how the consent notice integrates with your routing logic.

To switch between configurations, simply swap the routing logic in [main.rs](http://main.rs/) between serving the embedded template or proxying to a UI backend.