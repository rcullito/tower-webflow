use crate::util::{WebflowHeaders, compare_signatures, from_request_parts};
use axum::http::request::Parts;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use axum::{
    body::{self, Body, Bytes},
    response::Response,
};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::Service;

const BODY_LIMIT: usize = 1_048_576;

/// Middleware that validates the x-webflow-signature header
#[derive(Clone)]
pub struct WebflowService<S, Secret> {
    pub(crate) inner: S,
    pub secret: Secret,
}


pub struct WorkData {
    signature: String,
    message_to_verify: String,
    body_bytes: Bytes,
    parts: Parts,
}

async fn work(req: Request<Body>) -> anyhow::Result<WorkData> {
    let (mut parts, body) = req.into_parts();

    let body_bytes = body::to_bytes(body, BODY_LIMIT).await?;

    let body_string = String::from_utf8(body_bytes.to_vec())?;

    tracing::debug!(body_string, "webflow-body");

    let WebflowHeaders(signature, timestamp) = from_request_parts(&mut parts)?;

    tracing::debug!(signature, "webflow-header-signature");
    tracing::debug!(timestamp, "webflow-header-timestamp");

    Ok(WorkData {
        message_to_verify: format!("{timestamp}:{body_string}"),
        signature,
        body_bytes,
        parts,
    })
}

impl<S, Secret> Service<Request<Body>> for WebflowService<S, Secret>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    Secret: AsRef<[u8]> + Clone + Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx).map_err(Into::into)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();
        let secret = self.secret.clone();
        Box::pin(async move {
            let WorkData {
                message_to_verify,
                signature,
                body_bytes,
                parts,
            } = match work(req).await {
                Ok(data) => data,
                Err(e) => return Ok((StatusCode::BAD_REQUEST, e.to_string()).into_response()),
            };

            if compare_signatures(&message_to_verify, secret.as_ref(), &signature) {
                let new_req = Request::from_parts(parts, Body::from(body_bytes));
                inner.call(new_req).await
            } else {
                Ok((
                    StatusCode::BAD_REQUEST,
                    "Webflow signature validation failed",
                )
                    .into_response())
            }
        })
    }
}
