use crate::service::WebflowService;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tower::{BoxError, Layer};

pub async fn box_err_to_res(err: BoxError) -> Response {
    tracing::error!(?err, "middleware error");
    (StatusCode::BAD_REQUEST, err.to_string()).into_response()
}

/// Layer that applies the [WebflowService] middleware.
#[derive(Clone)]
pub struct WebflowLayer{
    webflow_form_secret: String,
}

impl<S> Layer<S> for WebflowLayer
{
    type Service = WebflowService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        WebflowService { inner, secret: self.webflow_form_secret.clone() }
    }
}
