use crate::TokenVeirfy;
use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

pub async fn verify_token<T>(State(state): State<T>, req: Request, next: Next) -> Response
where
    T: TokenVeirfy + Clone + Send + Sync + 'static,
{
    let (mut parts, body) = req.into_parts();
    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_string(),
            Err(e) => {
                let msg = format!("parse authorization error: {}", e);
                return (StatusCode::UNAUTHORIZED, msg).into_response();
            }
        };

    let req = match state.vetify(&token) {
        Ok(user) => {
            let mut req = Request::from_parts(parts, body);
            req.extensions_mut().insert(user);
            req
        }
        Err(e) => {
            let msg = format!("verify token failed: {:?}", e);
            return (StatusCode::FORBIDDEN, msg).into_response();
        }
    };

    next.run(req).await
}
