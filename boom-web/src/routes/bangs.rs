use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;
use boom_core::{Redirect, cache::get_redirects};
use serde::Serialize;

use crate::AppState;

#[derive(Serialize, Debug)]
pub struct TemplateData {
    bangs: Vec<Redirect>,
}

impl TemplateData {
    pub fn new(bangs: Vec<Redirect>) -> Self {
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

pub async fn list_bangs(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("/bangs", state.engine, TemplateData::default())
}
