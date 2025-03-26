use std::time::Instant;

use boom::boom::{Match, parse_bangs::parse_bang_indexes};

#[test]
fn test_bang_retrieval_none() {
    let str = "youtube";
    let timer = Instant::now();
    let indices = unsafe { parse_bang_indexes(str) };
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
    let indices = unsafe { parse_bang_indexes(prefix) };
    eprintln!(
        "Took {:?} to retrieve the bang indices. (PREFIX)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(Match::new(0, 6)))
}

#[test]
fn test_bang_retrieval_suffix() {
    let suffix = "youtube !local";
    let timer = Instant::now();
    let indices = unsafe { parse_bang_indexes(suffix) };
    eprintln!(
        "Took {:?} to retrieve the bang indices. (SUFFIX)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(Match::new(8, suffix.len())))
}

#[test]
fn test_bang_retrieval_suffix_long() {
    let suffix = "a long query that i want search lol on youtube !local";
    let timer = Instant::now();
    let indices = unsafe { parse_bang_indexes(suffix) };
    eprintln!(
        "Took {:?} to retrieve the bang indices. (SUFFIX LONG)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(Match::new(suffix.len() - 6, suffix.len())))
}

#[test]
fn test_bang_retrieval_suffix_invalid() {
    let invalid_suffix = "test!gh";
    let timer = Instant::now();
    let indices = unsafe { parse_bang_indexes(invalid_suffix) };
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
    let indices = unsafe { parse_bang_indexes(suffix) };
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
    let indices = unsafe { parse_bang_indexes(infix) };
    eprintln!(
        "Took {:?} to retrieve the bang indices. (INFIX)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(Match::new(19, 21)))
}

#[test]
fn test_bang_invalid_proceeding_space() {
    let infix = "test! ";
    let timer = Instant::now();
    let indices = unsafe { parse_bang_indexes(infix) };
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
    let indices = unsafe { parse_bang_indexes(infix) };
    eprintln!(
        "Took {:?} to retrieve the bang indices. (INVALID PRECEEDING SPACE)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(Match::new(6, infix.len())))
}

#[test]
fn test_bang_invalid_single_char() {
    let infix = "! test !gh";
    let timer = Instant::now();
    let indices = unsafe { parse_bang_indexes(infix) };
    eprintln!(
        "Took {:?} to retrieve the bang indices. (INVALID SINGLE CHAR)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(Match::new(7, infix.len())))
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
