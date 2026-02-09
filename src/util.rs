use axum::http::{HeaderValue, request::Parts};
use std::fmt::Write;

use anyhow::anyhow;
use hmac::{Hmac, Mac};
use sha2::Sha256;
type HmacSha256 = Hmac<Sha256>;

type Signature = String;
type Timestamp = String;

pub struct WebflowHeaders(pub Signature, pub Timestamp);

impl TryFrom<(&HeaderValue, &HeaderValue)> for WebflowHeaders {
    type Error = anyhow::Error;
    fn try_from(t: (&HeaderValue, &HeaderValue)) -> Result<Self, Self::Error> {
        match (t.0.to_str(), t.1.to_str()) {
            (Ok(signature), Ok(timestamp)) => {
                Ok(Self(signature.to_string(), timestamp.to_string()))
            }
            _ => Err(anyhow!(
                "signature and/or timestamp cannot be converted to a string"
            )),
        }
    }
}

pub fn from_request_parts(Parts { headers, .. }: &mut Parts) -> anyhow::Result<WebflowHeaders> {
    headers
        .get("x-webflow-signature")
        .zip(headers.get("x-webflow-timestamp"))
        .ok_or(anyhow!(
            "`x-webflow-signature` and `x-webflow-timestamp` are required."
        ))
        .map(TryInto::try_into)?
}

pub fn compare_signatures(message: &str, secret: &[u8], provided_signature: &str) -> bool {
    let mut mac = match HmacSha256::new_from_slice(secret) {
        Ok(mac) => mac,
        Err(e) => {
            tracing::error!(error = ?e, "invalid secret");
            return false;
        }
    };
    mac.update(message.as_bytes());

    let result = mac.finalize();

    let hash = result
        .into_bytes()
        .iter()
        .fold(String::new(), |mut acc, b| {
            let _ = write!(acc, "{b:02x}");
            acc
        });

    let signature_match = hash == provided_signature;
    tracing::debug!(signature_match, "webflow-match-result");
    signature_match
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_compare_signatures() {
        let payload = r#"1765925777826:{"triggerType":"form_submission","payload":{"name":"Careers Form","siteId":"691b7f4f00f2a80eded550da","data":{"Name":"Brian Faga","Email":"bfaga@txse.com","Message 2":"Howdy","Reason for Inquiry":"Careers"},"submittedAt":"2025-12-16T22:56:17.609Z","id":"6941e3911a5714d0f068a7b3","formId":"691b821e6c69c60878eb9cfe","formElementId":"700c51ea-1523-0542-a665-324c3ef9a4a1","pageId":"691b7f4f00f2a80eded550fe","publishedPath":"/contact","pageUrl":"https://txse-82dfd6ed14c32c7cf5cf69f55ea38f7a.webflow.io/contact","schema":[]}}"#;
        assert!(compare_signatures(
            payload,
            b"dummy-test-secret",
            "a2bb26f5617de0a58c9c115524a3b782bc914024aa4a42f3f3953a37e0140f06"
        ));
    }
}
