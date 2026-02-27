use axum::{Json, extract::State, http::Response, response::IntoResponse};
use boom_config::{BangCustomConfig, ConfigBuilder};
use boom_core::{
    Redirect,
    cache::{get_bang, get_redirects, update_redirect},
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

use crate::AppState;

#[derive(Deserialize, Clone, Debug)]
pub struct RedirectExtras {
    #[serde(flatten)]
    base: Redirect,
    #[serde(rename = "overwrite")]
    allow_overwrite: Option<String>,
}

pub async fn add_bang(
    State(state): State<AppState>,
    Json(req): Json<RedirectExtras>,
) -> impl IntoResponse {
    let new_bang = req.base;

    let handle_update_redirect = |b| match update_redirect(b) {
        Ok(()) => {
            let mut cfg_builder: ConfigBuilder = state.shared_config.read().unwrap().clone().into();
            cfg_builder.add_custom_bang(
                b.trigger.clone(),
                BangCustomConfig {
                    template: b.url_template.clone(),
                    short_name: b.short_name.clone(),
                },
            );
            cfg_builder.serialize();

            Response::builder()
                .status(StatusCode::CREATED)
                .body(
                    json!({
                        "s": new_bang.short_name,
                        "t": new_bang.trigger,
                        "u": new_bang.url_template,
                    })
                    .to_string(),
                )
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(
                "The bang wasn't a duplicate... we still couldn't add it, for some reason."
                    .to_string(),
            )
            .unwrap(),
    };

    match req.allow_overwrite {
        Some(_) => handle_update_redirect(&new_bang),
        None => get_bang(&new_bang.trigger).unwrap_or_default().map_or_else(
            || handle_update_redirect(&new_bang),
            |idx| {
                let bang = get_redirects().expect("Read Lock on Redirects");
                Response::builder()
                    .status(StatusCode::CONFLICT)
                    .body(
                        json!({
                            "s": bang[idx].short_name,
                            "t": bang[idx].trigger,
                            "u": bang[idx].url_template,
                        })
                        .to_string(),
                    )
                    .unwrap()
            },
        ),
    }
}
