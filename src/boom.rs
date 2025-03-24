use std::{env, fs::File, path::PathBuf};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Redirect {
    /// The short name or abbreviation of the bang command.
    #[serde(rename = "s")]
    pub short_name: String,
    /// The trigger text for the bang command (e.g., "g" for Google).
    #[serde(rename = "t")]
    pub trigger: String,
    /// The URL template where the search term is inserted.
    #[serde(rename = "u")]
    pub url_template: String,
}

/// Parses bangs from using a path to a JSON file OR `./default_bangs.json`
/// and returns a vector of [Redirect]
///
/// # Panics
///
/// It will fail when:
/// - The file does not exist (potentially misspelt path)
/// - The path is not to a file (potentailly a directory)
///
/// # Errors
///
/// Returns an [`Err`](https://doc.rust-lang.org/stable/core/result/enum.Result.html#variant.Err) in the following cases:
/// - The path is unable to be opened
/// - The contents of the file were unable to be converted to valid json
/// - The current working directory value is invalid.
///     * Possible cases:
///     * Current directory does not exist.
///     * There are insufficient permissions to access the current directory.
///
/// # Example
/// ```
/// // Use default bangs file
/// const vec: Vec<Redirect> = parse_bangs(None)?;
/// ```
pub fn parse_bangs(
    bang_path: Option<PathBuf>,
) -> Result<Vec<Redirect>, Box<dyn std::error::Error>> {
    let bangs = if let Some(p) = bang_path {
        p
    } else {
        let mut cwd = env::current_dir()?;
        cwd.push("default_bangs.json");
        cwd
    };

    assert!(bangs.exists(), "File {bangs:?} does not exist.",);
    assert!(bangs.is_file(), "{bangs:?} is not a file.");

    let bang_file = File::open(bangs)?;
    let breader = std::io::BufReader::new(bang_file);

    let redirects: Vec<Redirect> = serde_json::from_reader(breader)?;
    Ok(redirects)
}
