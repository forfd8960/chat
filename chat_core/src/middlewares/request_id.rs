use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    // if the x-request-id header is not set, generate a new one
    let req_id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(v) => Some(v.clone()),
        None => {
            // generate a new x-request-id header
            let id = uuid::Uuid::now_v7().to_string();
            match HeaderValue::from_str(&id) {
                Ok(hdr) => {
                    req.headers_mut().insert(REQUEST_ID_HEADER, hdr.clone());
                    Some(hdr)
                }
                Err(e) => {
                    warn!("parse generated request id failed: {}", e);
                    None
                }
            }
        }
    };

    // add the x-request-id header to the response
    let mut res = next.run(req).await;
    let Some(req_id) = req_id else {
        return res;
    };

    res.headers_mut().insert(REQUEST_ID_HEADER, req_id);
    res
}
