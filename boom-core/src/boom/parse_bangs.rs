#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use std::arch::x86_64::{
    __m128i, _MM_HINT_NTA, _mm_cmpeq_epi8, _mm_load_si128, _mm_loadu_si128, _mm_movemask_epi8,
    _mm_prefetch, _mm_set1_epi8,
};

use std::path::PathBuf;

use crate::Redirect;

use super::Match;

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
/// use std::env;
/// use boom::boom::{Redirect, parse_bangs::parse_bang_file};
/// // Use default bangs file
//  let vec: Vec<Redirect> = parse_bang_file(&{
//      let mut d = env::current_dir().unwrap();
//      d.push("default_bangs.json");
//      d
//  }).unwrap_or(vec![]);
/// ```
pub fn parse_bang_file(bangs: &PathBuf) -> Result<Vec<Redirect>, Box<dyn std::error::Error>> {
    assert!(bangs.exists(), "File {} does not exist.", bangs.display());
    assert!(bangs.is_file(), "{} is not a file.", bangs.display());

    Ok(serde_json::from_str(
        std::fs::read_to_string(bangs)?.as_str(),
    )?)
}

#[inline]
fn parse_bang_indexes_iter(bang: &str) -> Option<Match> {
    let bytes = bang.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    while i < len {
        if bytes[i] == b'!' && (i == 0 || bytes[i - 1] == b' ') && (i < len && bytes[i + 1] != b' ')
        {
            let start = i;
            let mut end = start + 1;
            while end < len && bytes[end] != b' ' {
                end += 1;
            }
            return Some(Match { start, end });
        }
        i += 1;
    }
    None
}

/// Parse the indexes of the bang within the string.
/// Will use a SIMD approach when possible, defaulting to an iterative one.
/// The performance between the two is negligible.
///
/// **NOTE**: the start index is __inclusive__, whereas the end is __exclusive__.
///
/// # Example
/// ```rs
/// let bang_str = "!gh tobybridle";
/// if let Some(indexes) = parse_bang_indexes(bang_str) {
///     dbg!(indexes); // Match { start: 0, end: 3}
/// }
/// ```
///
#[inline]
#[must_use]
pub fn parse_bang_indexes(bang: &str) -> Option<Match> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    return unsafe {
        let bytes = bang.as_bytes();
        let len = bytes.len();
        let mut i = 0usize;
        let mask = _mm_set1_epi8(b'!' as i8);

        while i + 16 <= len {
            let ptr = bytes[i..].as_ptr() as *const __m128i;
            if i + 32 <= len {
                _mm_prefetch(ptr.byte_add(16) as *const i8, _MM_HINT_NTA);
            }

            let chunk = {
                if ptr.align_offset(16) == 0 {
                    _mm_load_si128(ptr) // Aligned load
                } else {
                    _mm_loadu_si128(ptr) // Unaligned load fallback
                }
            };

            let cmp_mask = _mm_cmpeq_epi8(chunk, mask);
            let mask_map = _mm_movemask_epi8(cmp_mask);

            if mask_map != 0 {
                let first_bit = mask_map.trailing_zeros() as usize;
                let start = i + first_bit;

                if start == 0 || bytes[start - 1] == b' ' {
                    let mut end = start + 1;
                    while end < len && bytes[end] != b' ' {
                        end += 1;
                    }
                    return Some(Match { start, end });
                }
            }

            i += 16;
        }

        parse_bang_indexes_iter(&bang[i..]).map(|Match { start, end }| Match {
            start: start + i,
            end: end + i,
        })
    };

    parse_bang_indexes_iter(bang)
}
