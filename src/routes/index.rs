use std::time::Instant;

use axum::{extract::Query, response::Redirect};
use serde::Deserialize;
use tracing::info;

use crate::boom::resolver::resolve;

#[derive(Deserialize)]
pub struct SearchParams {
    #[serde(rename = "q")]
    query: String,
}

pub async fn redirector(Query(params): Query<SearchParams>) -> Redirect {
    let timer = Instant::now();
    let resolved = resolve(params.query.as_str());
    info!("Redirecting to {resolved} took {:?}", timer.elapsed());
    Redirect::to(resolved.as_str())
}
