use crate::cache::REDIRECT_LIST;
use ntex::{
    http::{StatusCode, body::Body},
    web,
};

#[web::get("/bangs")]
pub async fn list_bangs() -> web::HttpResponse {
    REDIRECT_LIST.try_read().map_or_else( |_|
        web::HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR).set_body(Body::from_message(
            "<h1>Oops. Something went wrong on the server.</h1>",
        ))
        , |lock| {
        let mut buffer =
            "<style>table { font-family: monospace; } table th { text-align: left; padding: 1rem 0; font-size: 1.25rem; } table tr:nth-child(2n) { background: #161616; } table tr:nth-child(2n+1) { background: #181818; }</style><table><thead><th>Abbr.</th><th>Trigger</th><th>Template URL</th></thead>".to_string();
        lock.iter().for_each(|redirection| {
            buffer.push_str(
                format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
                    redirection.short_name, redirection.trigger, redirection.url_template
                )
                .as_str(),
            );
        });
        buffer.push_str("</table>");
        web::HttpResponse::Ok()
            .content_type("text/html")
            .body(buffer)
    })
}
