use boom::boom::{
    DEFAULT_SEARCH_INDEXES, DEFAULT_SEARCH_TEMPLATE, parse_bangs::parse_bang_indexes,
};
use ntex::{
    http::header::LOCATION,
    web::{self, HttpRequest},
};

#[web::get("/")]
pub async fn redirector(r: HttpRequest) -> Option<web::HttpResponse> {
    let query = &urlencoding::decode(r.query_string()).ok()?[2..];
    let bang_idx = parse_bang_indexes(query).unwrap_or_default();

    if bang_idx.is_empty() {
        return Some(
            web::HttpResponse::PermanentRedirect()
                .header(
                    LOCATION,
                    concat_string!(
                        &DEFAULT_SEARCH_TEMPLATE[..DEFAULT_SEARCH_INDEXES.start],
                        query
                    ),
                )
                .finish(),
        );
    }

    None
    // let res = if query.is_empty() || query.len() < 4 {
    //     web::HttpResponse::BadRequest()
    //         .content_type("text/html")
    //         .body("<h1>Query was invalid</h1>")
    // } else {
    //     let query_len = "?q=".len() - 1;
    //     let bmatch = parse_bang_indexes(&query).unwrap_or_default();
    //
    //     let bang = &query[bmatch.to_indices(query_len)].to_ascii_lowercase();
    //     info!("Bang was {bang}. Match {bmatch:?}");
    //
    //     if bmatch.is_empty() {
    //         info!("Quitting eatly.");
    //         return Some(
    //             web::HttpResponse::PermanentRedirect()
    //                 .header(
    //                     header::LOCATION,
    //                     DEFAULT_QUERY.replace("{{{s}}}", &query[query_len - 1..]),
    //                 )
    //                 .finish(),
    //         );
    //     }
    //
    //     let rlock = REDIRECT_LIST.try_read().ok()?;
    //     let redirect =
    //         if let Some((start_index, end_index, template_index)) = get_bang(bang).ok()? {
    //             info!("Found {bang} in the cache. URL Index: {template_index}.");
    //             let template = rlock.index(template_index).url_template.to_owned();
    //
    //             concat_string!(
    //                 &template[0..start_index],
    //                 &query[bmatch.end..],
    //                 &template[end_index..]
    //             )
    //         } else {
    //             let redirect = rlock
    //                 .iter()
    //                 .enumerate()
    //                 .find(|(_, redirect)| redirect.trigger == *bang)?;
    //             info!(
    //                 "Inserting {bang} into the cache and calculating indexes. URL Index: {}",
    //                 redirect.0
    //             );
    //             let tmatch = parse_template_indexes(&redirect.1.url_template).unwrap_or_default();
    //             insert_bang(bang.to_string(), redirect.0, tmatch[0].start, tmatch[0].end).ok()?;
    //             let template = redirect.1.url_template.to_owned();
    //             concat_string!(
    //                 &template[0..tmatch[0].start],
    //                 urlencoding::encode(&query[bmatch.end..]),
    //                 &template[tmatch[0].end..]
    //             )
    //         };
    //     drop(rlock);
    //
    //     info!("Redirecting. Took {:?}", now.elapsed());
    //     web::HttpResponse::PermanentRedirect()
    //         .header(header::LOCATION, redirect)
    //         .finish()
    // };

    // Some(res)
}
