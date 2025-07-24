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
