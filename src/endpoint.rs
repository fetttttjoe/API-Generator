// endpoint.rs
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Endpoint {
    pub route: String,
    pub method: String,
    pub purpose: String, // New field for the purpose or use case
}