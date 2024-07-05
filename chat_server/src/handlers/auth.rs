use axum::response::IntoResponse;

#[allow(dead_code)]
pub(crate) async fn signin_handler() -> impl IntoResponse {
    "Signin handler"
}

#[allow(dead_code)]
pub(crate) async fn signup_handler() -> impl IntoResponse {
    "Signup handler"
}
