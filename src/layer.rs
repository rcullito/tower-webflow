use crate::service::WebflowService;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tower::Layer;

/// Layer that applies the [WebflowService] middleware.
#[derive(Clone)]
pub struct WebflowLayer<Secret> {
    webflow_form_secret: Secret,
}

impl<Secret> WebflowLayer<Secret> {
    pub fn new(webflow_form_secret: Secret) -> Self {
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
