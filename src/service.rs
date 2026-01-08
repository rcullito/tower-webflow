use crate::util::{WebflowHeaders, compare_signatures, from_request_parts};
use axum::http::Request;
use axum::{
    body::{self, Body},
    response::Response,
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{BoxError, Service};

const BODY_LIMIT: usize = 1_048_576;

/// Middleware that validates the x-webflow-signature header
#[derive(Clone)]
pub struct WebflowService<S> {
    pub(crate) inner: S,
    pub secret: String
}

impl<S> Service<Request<Body>> for WebflowService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Into<BoxError>,
{
    type Response = Response;
    type Error = BoxError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let secret = self.secret.clone();
        Box::pin(async move {
            let (mut parts, body) = req.into_parts();

            let body_bytes = body::to_bytes(body, BODY_LIMIT)
                .await
                .map_err(Into::<BoxError>::into)?;

            let body_string =
                String::from_utf8(body_bytes.to_vec()).map_err(Into::<BoxError>::into)?;

            tracing::debug!(body_string, "webflow-body");

            let WebflowHeaders(signature, timestamp) = from_request_parts(&mut parts)?;

            tracing::debug!(signature, "webflow-header-signature");
            tracing::debug!(timestamp, "webflow-header-timestamp");

            let message_to_verify = format!("{timestamp}:{body_string}");

            if compare_signatures(&message_to_verify, secret.as_bytes(), &signature) {
                let new_req = Request::from_parts(parts, Body::from(body_bytes));
                inner.call(new_req).await.map_err(Into::into)
            } else {
                tracing::warn!("Webflow signature validation failed");
                Err("Could not verify signature".into())
            }
        })
    }
}
