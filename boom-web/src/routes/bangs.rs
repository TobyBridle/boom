use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;
use boom_core::{Redirect, cache::get_redirects};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize, Debug)]
struct TemplateData {
    bangs: Vec<Redirect>,
}

impl TemplateData {
    #[allow(dead_code)]
    pub const fn new(bangs: Vec<Redirect>) -> Self {
        Self { bangs }
    }
}

impl Default for TemplateData {
    fn default() -> Self {
        Self {
            bangs: get_redirects().unwrap().clone(),
        }
    }
}

/// [`list_bangs`] - a quite self-explanatory name.
pub async fn list_bangs(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("/bangs", state.engine, TemplateData::default())
}
