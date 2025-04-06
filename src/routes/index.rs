use std::time::Instant;

use ntex::{
    http::header::LOCATION,
    web::{self, HttpRequest},
};
use tracing::info;

use crate::boom::resolver::resolve;

#[web::get("/")]
pub async fn redirector(r: HttpRequest) -> Option<web::HttpResponse> {
    let query = &urlencoding::decode(r.query_string()).ok()?[2..];
    let timer = Instant::now();
    let resolved = resolve(query);
    info!("Redirecting to {resolved} took {:?}", timer.elapsed());
    Some(
        web::HttpResponse::PermanentRedirect()
            .header(LOCATION, resolved)
            .finish(),
    )
}
