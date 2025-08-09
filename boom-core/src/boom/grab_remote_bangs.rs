use std::{
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
};

use reqwest::Client;

/// Downloads an external bang json file (`url`) into the designated file (`out`)
///
/// # Errors
/// If the initial request to `url` doesn't succeed.
/// If the output file cannot be created/written to
pub async fn download_remote(
    url: &String,
    out: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut res = client.get(url).send().await?;

    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut writer = BufWriter::new(
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(out)?,
    );
    while let Some(chunk) = res.chunk().await? {
        let _ = writer.write(&chunk)?;
    }

    Ok(())
}

// #[inline]
// async fn setup(config: Config) -> Result<(), Box<dyn std::error::Error>> {
//     let mut bangs = if config.bangs.default.enabled {
//         grab_remote(&config.bangs.default.remote, &config.bangs.default.filepath).await?;
//
//         parse_bang_file(&config.bangs.default.filepath)
//             .map_err(|e| {
//                 error!("Could not parse bangs! {:?}", e);
//             })
//             .unwrap()
//     } else {
//         info!("[bangs.default.enabled] = false");
//         vec![]
//     };
//
//     info!(name: "Boom", "Parsing Bangs!");
//     let now = Instant::now();
//
//     dbg!(&config);
//     config
//         .bangs
//         .custom
//         .iter()
//         .for_each(|(short_name, custom_config)| {
//             bangs.push(Redirect {
//                 short_name: short_name.clone(),
//                 trigger: custom_config.trigger.clone(),
//                 url_template: custom_config.template.clone(),
//             });
//         });
//
//     let bangs_len = bangs.len();
//     info!(
//         name: "Boom",
//         "Parsed {} bangs in {:?}!",
//         bangs_len,
//         Instant::now().duration_since(now)
//     );
//
//     init_list(bangs.clone(), false).ok();
//
//     bangs.iter().enumerate().for_each(|(idx, bang)| {
//         insert_bang(bang.trigger.clone(), idx).unwrap();
//     });
//
//     Ok(())
// }
