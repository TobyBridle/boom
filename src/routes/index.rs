use std::{ops::Index, time::Instant};

use ntex::{
    http::header,
    web::{self, HttpRequest},
};
use tracing::info;

use crate::cache::{DEFAULT_QUERY, REDIRECT_LIST, get_bang, insert_bang};

#[web::get("/")]
pub async fn redirector(r: HttpRequest) -> Option<web::HttpResponse> {
    let now = Instant::now();
    let query = urlencoding::decode(r.query_string()).ok()?.into_owned();

    let res = if query.is_empty() || query.len() < 4 {
        web::HttpResponse::BadRequest()
            .content_type("text/html")
            .body("<h1>Query was invalid</h1>")
    } else {
        let query_len = "?q=".len();
        let start = query_len;
        let mut end = query_len;

        if query.chars().nth(start - 1).unwrap_or(' ') == '!' {
            for (i, ch) in query[start..].char_indices() {
                if ch == ' ' {
                    end = start + i;
                    break;
                }
            }
        }

        let bang = &query[start..end].to_ascii_lowercase();
        if end == start {
            info!("Quitting eatly.");
            return Some(
                web::HttpResponse::PermanentRedirect()
                    .header(
                        header::LOCATION,
                        DEFAULT_QUERY.replace("{{{s}}}", &query[query_len - 1..]),
                    )
                    .finish(),
            );
        }

        let rlock = REDIRECT_LIST.try_read().ok()?;
        let redirect =
            if let Some((start_index, end_index, template_index)) = get_bang(bang).ok()? {
                info!("Found {bang} in the cache. URL Index: {template_index}.");
                let template = rlock.index(template_index).url_template.to_owned();

                concat_string!(
                    &template[0..start_index],
                    &query[end..],
                    &template[end_index..]
                )
            } else {
                let redirect = rlock
                    .iter()
                    .enumerate()
                    .find(|(_, redirect)| redirect.trigger == *bang)?;
                info!(
                    "Inserting {bang} into the cache and calculating indexes. URL Index: {}",
                    redirect.0
                );
                let start_index = redirect.1.url_template.find("{{{s}}}")?;
                let end_index = start_index + "{{{s}}}".len();
                insert_bang(bang.to_string(), redirect.0, start_index, end_index).ok()?;
                let template = redirect.1.url_template.to_owned();
                concat_string!(
                    &template[0..start_index],
                    urlencoding::encode(&query[end..]),
                    &template[end_index..]
                )
            };
        drop(rlock);

        info!("Redirecting. Took {:?}", now.elapsed());
        web::HttpResponse::PermanentRedirect()
            .header(header::LOCATION, redirect)
            .finish()
    };

    Some(res)
}
