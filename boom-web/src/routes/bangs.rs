use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;
use boom_core::{Redirect, cache::get_redirects};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize, Debug)]
struct TemplateData {
    bangs: Vec<Redirect>,
}

pub async fn list_bangs(State(state): State<AppState>) -> impl IntoResponse {
    let data = TemplateData {
        bangs: get_redirects().unwrap().clone(),
    };

    RenderHtml("/bangs", state.engine, data)
}
