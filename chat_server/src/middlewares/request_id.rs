use axum::http::HeaderValue;
use axum::response::Response;
use axum::{extract::Request, middleware::Next};
use tracing::warn;
use uuid::{ContextV7, Timestamp, Uuid};

const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // if the x-request-id header is not set, generate a new one
    let req_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => v.as_bytes().to_vec(),
        None => {
            // generate a new x-request-id header
            let context = ContextV7::new();
            let id = Uuid::new_v7(Timestamp::from_unix(&context, 1497624119, 1234)).to_string();
            match id.parse() {
                Ok(v) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, v);
                    ()
                }
                Err(e) => {
                    warn!("Failed to generate request id: {}", e)
                }
            };

            id.as_bytes().to_vec()
        }
    };

    // add the x-request-id header to the response
    let mut res = next.run(req).await;
    match HeaderValue::from_bytes(&req_id) {
        Ok(v) => {
            res.headers_mut().insert(REQUEST_ID_HEADER, v);
            ()
        }
        Err(e) => warn!("Failed to parse request id: {}", e),
    }
    res
}
