use axum::{Json, http::Response, response::IntoResponse};
use boom_core::{
    Redirect,
    cache::{get_bang, get_redirects, update_redirect},
};
use reqwest::StatusCode;
use serde_json::json;

pub async fn add_bang(Json(new_bang): Json<Redirect>) -> impl IntoResponse {
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
        match update_redirect(&new_bang) {
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
        }
    }
}
