use std::time::Instant;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_template::RenderHtml;
use boom_core::boom::resolver::resolve;
use serde::Deserialize;
use tower::util::Either;
use tracing::info;

use crate::{AppState, EitherResponse};

use super::bangs::TemplateData;

#[derive(Deserialize)]
pub struct SearchParams {
    #[serde(rename = "q")]
    query: Option<String>,
}

pub async fn redirector(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let res = if let Some(query) = params.query {
        let timer = Instant::now();
        let resolved = resolve(
            query.as_str(),
            &state
                .shared_config
                .read()
                .expect("Shared Config should not be poisoned")
                .clone(),
        );
        info!("Redirecting to {resolved} took {:?}", timer.elapsed());
        Either::Left(Redirect::to(resolved.as_str()))
    } else {
        Either::Right(RenderHtml("/bangs", state.engine, TemplateData::default()))
    };

    EitherResponse(res)
}
