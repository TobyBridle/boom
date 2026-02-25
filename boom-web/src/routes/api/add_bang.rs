use axum::{Json, http::Response, response::IntoResponse};
use boom_core::{
    Redirect,
    cache::{get_bang, get_redirects, update_redirect},
};
use reqwest::StatusCode;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize, Clone, Debug)]
pub(crate) struct RedirectExtras {
    #[serde(flatten)]
    base: Redirect,
    #[serde(rename = "overwrite")]
    allow_overwrite: Option<String>,
}

pub async fn add_bang(Json(req): Json<RedirectExtras>) -> impl IntoResponse {
    let new_bang = req.base;

    let handle_update_redirect = |b| match update_redirect(b) {
        Ok(()) => Response::builder()
            .status(StatusCode::CREATED)
            .body(
                json!({
                    "s": new_bang.short_name,
                    "t": new_bang.trigger,
                    "u": new_bang.url_template,
                })
                .to_string(),
            )
            .unwrap(),
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
        None => {
            if let Some(b_idx) = get_bang(&new_bang.trigger).unwrap_or_default() {
                let bang = get_redirects().expect("Read Lock on Redirects");
                Response::builder()
                    .status(StatusCode::CONFLICT)
                    .body(
                        json!({
                            "s": bang[b_idx].short_name,
                            "t": bang[b_idx].trigger,
                            "u": bang[b_idx].url_template,
                        })
                        .to_string(),
                    )
                    .unwrap()
            } else {
                handle_update_redirect(&new_bang)
            }
        }
    }
}
