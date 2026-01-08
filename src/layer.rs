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
pub struct WebflowLayer<Secret> {
    webflow_form_secret: Secret,
}

impl<Secret> WebflowLayer<Secret> {
    pub fn new(webflow_form_secret: Secret) -> Self
    where
        Secret: AsRef<[u8]> + Clone,
    {
        Self {
            webflow_form_secret,
        }
    }
}

impl<S, Secret> Layer<S> for WebflowLayer<Secret>
where
    Secret: AsRef<[u8]> + Clone,
{
    type Service = WebflowService<S, Secret>;

    fn layer(&self, inner: S) -> Self::Service {
        WebflowService {
            inner,
            secret: self.webflow_form_secret.clone(),
        }
    }
}
