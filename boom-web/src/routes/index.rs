use std::time::Instant;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use axum_template::RenderHtml;
use boom_core::{SourceIdentifier, boom::resolver::resolve};
use serde::{Deserialize, Serialize};
use tower::util::Either;
use tracing::info;

use crate::{AppState, EitherResponse};

#[derive(Deserialize, Debug)]
pub struct SearchParams {
    #[serde(rename = "q")]
    pub(crate) query: Option<String>,

    #[serde(rename = "si")]
    pub(crate) source_identifier: Option<SourceIdentifier>,
}

#[derive(Serialize)]
struct TemplateData {
    source_identifier: String,
}

/// [`redirector`] handles directing the user to the location of their parsed query, or, if no
/// query is provided, showing them to the `boom` homepage.
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
            &params.source_identifier.unwrap_or_default(),
        );
        info!("Redirecting to {resolved} took {:?}", timer.elapsed());
        Either::Left(Redirect::to(resolved.as_str()))
    } else {
        Either::Right(RenderHtml(
            "/",
            state.engine,
            TemplateData {
                source_identifier: params.source_identifier.unwrap_or_default().into(),
            },
        ))
    };

    EitherResponse(res)
}
