use axum::{extract::State, response::IntoResponse};
use axum_template::RenderHtml;
use boom_core::{HistoryEntry, cache::SEARCH_HISTORY_CACHE};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Serialize, Deserialize)]
struct TemplateData {
    history: Vec<HistoryEntry>,
}

impl Default for TemplateData {
    fn default() -> Self {
        Self {
            history: SEARCH_HISTORY_CACHE.read().unwrap().clone(),
        }
    }
}

pub async fn list_history(State(state): State<AppState>) -> impl IntoResponse {
    RenderHtml("/history", state.engine, TemplateData::default())
}
