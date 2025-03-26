#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use std::arch::x86_64::*;

use std::time::Instant;

#[inline(always)]
fn iterative_retrieve_bang(str: &str, offset: Option<usize>) -> Option<(usize, usize)> {
    let bytes = str.as_bytes();
    let len = bytes.len();
    let offset = offset.unwrap_or(0);
    let chunk_size = 16; // Adjustable for tuning performance

    let mut i = 0;

    // Check for a bang at the start (ignoring empty bangs like "! ")
    if bytes[0] == b'!' && bytes.get(1) != Some(&b' ') {
        if let Some(end) = bytes.iter().position(|c| *c == b' ') {
            return Some((0, end));
        }
    }

    while i < len {
        let end = std::cmp::min(i + chunk_size, len);
        let chunk = &bytes[i..end];

        for (j, &b) in chunk.iter().enumerate() {
            let base_idx = i + j + offset; // Adjust base index with offset
            if b == b'!' && (j == 0 || chunk.get(j - 1) == Some(&b' ')) {
                // Ensure there are characters after '!'
                if j + 1 < chunk.len() && chunk[j + 1] != b' ' {
                    let start = base_idx;
                    let mut end = start;
                    while end < len && bytes[end] != b' ' {
                        end += 1;
                    }
                    return Some((start, end));
                }
            }
        }
        i += chunk_size;
    }

    None
}

#[inline(always)]
fn retrieve_bang(str: &str) -> Option<(usize, usize)> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    if str.len() >= 16 {
        let (prefix, aligned, suffix) = unsafe { str.as_bytes().align_to::<[u8; 16]>() };
        let mut offset = prefix.len();

        let mask = unsafe { _mm_set1_epi8('!' as i8) };

        for chunk in aligned {
            let simd_chunk = unsafe { _mm_load_si128(chunk.as_ptr() as *const __m128i) };
            let eq_chunk = unsafe { _mm_cmpeq_epi8(mask, simd_chunk) };
            let mask_chunk = unsafe { _mm_movemask_epi8(eq_chunk) };

            if mask_chunk != 0 {
                let idx: usize = offset + mask_chunk.trailing_zeros() as usize;
                return if str.chars().nth(idx - 1).unwrap_or(' ') != ' ' {
                    None
                } else {
                    Some((idx, idx + str[idx..].find(' ').unwrap_or(str.len() - idx)))
                };
            }

            offset += 16;
        }

        return iterative_retrieve_bang(str, Some(suffix.len()));
    }

    iterative_retrieve_bang(str, None)
}

#[test]
fn test_bang_retrieval_none() {
    let str = "youtube";
    let timer = Instant::now();
    let indices = retrieve_bang(str);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (NONE)",
        timer.elapsed()
    );
    assert_eq!(indices, None)
}

#[test]
fn test_bang_retrieval_prefix() {
    let prefix = "!local youtube";
    let timer = Instant::now();
    let indices = retrieve_bang(prefix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (PREFIX)",
        timer.elapsed()
    );
    assert_eq!(indices, Some((0, 6)))
}

#[test]
fn test_bang_retrieval_suffix() {
    let suffix = "youtube !local";
    let timer = Instant::now();
    let indices = retrieve_bang(suffix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (SUFFIX)",
        timer.elapsed()
    );
    assert_eq!(indices, Some((8, suffix.len())))
}

#[test]
fn test_bang_retrieval_suffix_long() {
    let suffix = "a long query that i want search lol on youtube !local";
    let timer = Instant::now();
    let indices = retrieve_bang(suffix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (SUFFIX LONG)",
        timer.elapsed()
    );
    assert_eq!(indices, Some((suffix.len() - 6, suffix.len())))
}

#[test]
fn test_bang_retrieval_suffix_invalid() {
    let invalid_suffix = "test!gh";
    let timer = Instant::now();
    let indices = retrieve_bang(invalid_suffix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (SUFFIX INVALID)",
        timer.elapsed()
    );
    assert_eq!(indices, None)
}

#[test]
fn test_bang_retrieval_suffix_long_invalid() {
    let suffix = "a long query that i want search lol on youtube!local";
    let timer = Instant::now();
    let indices = retrieve_bang(suffix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (SUFFIX INVALID LONG)",
        timer.elapsed()
    );
    assert_eq!(indices, None)
}

#[test]
fn test_bang_retrieval_infix() {
    let infix = "search for this on !g please and !dont let others !work";
    let timer = Instant::now();
    let indices = retrieve_bang(infix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (INFIX)",
        timer.elapsed()
    );
    assert_eq!(indices, Some((19, 21)))
}

#[test]
fn test_bang_invalid_proceeding_space() {
    let infix = "test! ";
    let timer = Instant::now();
    let indices = retrieve_bang(infix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (INVALID PROCEEDING SPACE)",
        timer.elapsed()
    );
    assert_eq!(indices, None)
}

#[test]
fn test_bang_invalid_preceeding_space() {
    let infix = "test! !gh";
    let timer = Instant::now();
    let indices = retrieve_bang(infix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (INVALID PRECEEDING SPACE)",
        timer.elapsed()
    );
    assert_eq!(indices, Some((6, infix.len())))
}

#[test]
fn test_bang_invalid_single_char() {
    let infix = "! test !gh";
    let timer = Instant::now();
    let indices = retrieve_bang(infix);
    eprintln!(
        "Took {:?} to retrieve the bang indices. (INVALID SINGLE CHAR)",
        timer.elapsed()
    );
    assert_eq!(indices, Some((7, infix.len())))
}

#[cfg(feature = "measure-allocs")]
mod tests {
    use super::*;

    #[test]
    fn test_bang_retrieval_none_memory() {
        let alloc = allocation_counter::measure(|| {
            test_bang_retrieval_none();
        });
        eprintln!(
            "`test_bang_retrieval_none` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }

    #[test]
    fn test_bang_retrieval_prefix_memory() {
        let alloc = allocation_counter::measure(|| {
            test_bang_retrieval_prefix();
        });
        eprintln!(
            "`test_bang_retrieval_prefix` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }

    #[test]
    fn test_bang_retrieval_infix_memory() {
        let alloc = allocation_counter::measure(|| {
            test_bang_retrieval_infix();
        });
        eprintln!(
            "`test_bang_retrieval_infix` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }

    #[test]
    fn test_bang_retrieval_suffix_long_memory() {
        let alloc = allocation_counter::measure(|| {
            test_bang_retrieval_suffix();
        });
        eprintln!(
            "`test_bang_retrieval_suffix_long` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }
}
