use std::time::Instant;

use axum::{
    extract::{Query, State},
    response::Redirect,
};
use boom_core::boom::resolver::resolve;
use serde::Deserialize;
use tracing::info;

use crate::AppState;

#[derive(Deserialize)]
pub struct SearchParams {
    #[serde(rename = "q")]
    query: String,
}

pub async fn redirector(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Redirect {
    let timer = Instant::now();
    let resolved = resolve(
        params.query.as_str(),
        state
            .shared_config
            .read()
            .expect("Shared Config should not be poisoned")
            .clone(),
    );
    info!("Redirecting to {resolved} took {:?}", timer.elapsed());
    Redirect::to(resolved.as_str())
}
