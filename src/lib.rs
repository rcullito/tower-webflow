//! # Overview
//!
//! `tower-webflow` is a crate for verifying signed webhooks received from Webflow.
//!
//! The crate exports two structs: `WebflowLayer` and `WebflowService`. These
//! structs implement `tower_layer::Layer` and `tower_service::Service`, respectively, and so can
//! be used as middleware for any servers that build on top of the Tower ecosystem.


mod layer;
mod service;
mod util;

pub use layer::WebflowLayer;
pub use service::WebflowService;
