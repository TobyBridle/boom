use std::{
    fs::File,
    io::{self, BufReader, Read},
};

use boom_config::parse_config::parse_config;
use boom_core::boom::resolver::resolve;
use boom_web::serve;
use clap::Parser;
use cli::LaunchType;
use tracing::{Level, error, info};
pub mod cli;

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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(if cfg!(debug_assertions) {
            Level::DEBUG
        } else {
            Level::INFO
        })
        .with_ansi(true)
        .compact()
        .with_writer(io::stderr)
        .init();

    let args = cli::Args::parse();
    // let config = read_config(&args.config).expect("Config path should be valid & readable.");

    match args.launch {
        LaunchType::Serve { addr, port } => serve(addr.as_str(), port).await,
        LaunchType::Resolve { search_query } => {
            println!("Resolved: {:?}", resolve(search_query.as_str()));
        }
        LaunchType::Validate { verbose } => {
            info!("Reading {}", &args.config.display());
            let mut config_buffer = String::new();
            let mut breader = BufReader::new(File::open(&args.config)?);
            breader.read_to_string(&mut config_buffer)?;
            match parse_config(config_buffer) {
                Ok(cfg) => {
                    if verbose {
                        dbg!(cfg);
                    }
                    info!("Parsed config with no errors.")
                }
                Err(e) => error!(e),
            }
        }
    };

    Ok(())
}
