#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use std::arch::x86_64::*;

use std::time::Instant;

#[inline(always)]
fn iterative_parse_template_indexes(template: &str) -> Option<Vec<(usize, usize)>> {
    let mut results = Vec::new();
    let mut slice_index = 0;

    while let Some(start_index) = template[slice_index..].find("{{{s}}}") {
        let start = start_index + slice_index;
        let end = (start + "{{{s}}}".len()).min(template.len());
        results.push((start, end));

        // Move the slice_index forward to continue searching after the current match
        slice_index = end;
    }

    if results.is_empty() {
        None
    } else {
        Some(results)
    }
}

#[inline(always)]
fn parse_template_indexes(template: &str) -> Option<Vec<(usize, usize)>> {
    #[cfg(all(
        any(target_arch = "x86", target_arch = "x86_64"),
        target_feature = "avx2"
    ))]
    return match template.len() {
        0..16 => iterative_parse_template_indexes(template),
        16..38 => {
            let (prefix, aligned, suffix) = unsafe { template.as_bytes().align_to::<[u8; 16]>() };
            let mut offset = prefix.len();

            let mask = unsafe { _mm_set1_epi8('{' as i8) };

            let mut results: Vec<(usize, usize)> = Vec::new();

            for chunk in aligned {
                let simd_chunk = unsafe { _mm_load_si128(chunk.as_ptr() as *const __m128i) };
                let eq_chunk = unsafe { _mm_cmpeq_epi8(mask, simd_chunk) };
                let mask_chunk = unsafe { _mm_movemask_epi8(eq_chunk) };

                if mask_chunk != 0 {
                    let idx: usize = offset + mask_chunk.trailing_zeros() as usize;
                    if &template[idx..idx + "{{{s}}}".len()] == "{{{s}}}" {
                        results.push((idx, idx + 7))
                    }
                }

                offset += 16;
            }

            if let Some(suffix_pairs) = iterative_parse_template_indexes(
                &template[template.len() - suffix.len()..],
            )
            .map(|offset_pairs| {
                offset_pairs
                    .iter()
                    .map(|(start, end)| (offset + start, offset + end))
                    .collect::<Vec<(usize, usize)>>()
            }) {
                results.extend(suffix_pairs);
            }

            if results.is_empty() {
                None
            } else {
                Some(results)
            }
        }
        38.. => {
            let (prefix, aligned, suffix) = unsafe { template.as_bytes().align_to::<[u8; 32]>() };
            let mut offset = prefix.len();

            let mask = unsafe { _mm256_set1_epi8('{' as i8) };

            let mut results: Vec<(usize, usize)> = Vec::new();

            for chunk in aligned {
                let simd_chunk = unsafe { _mm256_load_si256(chunk.as_ptr() as *const __m256i) };
                let eq_chunk = unsafe { _mm256_cmpeq_epi8(mask, simd_chunk) };
                let mask_chunk = unsafe { _mm256_movemask_epi8(eq_chunk) };

                if mask_chunk != 0 {
                    let idx: usize = offset + mask_chunk.trailing_zeros() as usize;
                    if &template[idx..idx + "{{{s}}}".len()] == "{{{s}}}" {
                        results.push((idx, idx + 7))
                    }
                }

                offset += 32;
            }

            if let Some(suffix_pairs) = iterative_parse_template_indexes(
                &template[template.len() - suffix.len()..],
            )
            .map(|offset_pairs| {
                offset_pairs
                    .iter()
                    .map(|(start, end)| (offset + start, offset + end))
                    .collect::<Vec<(usize, usize)>>()
            }) {
                results.extend(suffix_pairs);
            }

            if results.is_empty() {
                None
            } else {
                Some(results)
            }
        }
    };

    iterative_parse_template_indexes(template)
}

#[test]
fn test_empty_template() {
    let template = "https://trika.gay";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!("Took {:?} to get template indices (EMPTY)", timer.elapsed());
    assert_eq!(indices, None);
}

#[test]
fn test_template_suffix() {
    let template = "https://google.com/search?q={{{s}}}";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (SUFFIX)",
        timer.elapsed()
    );
    assert_eq!(
        indices,
        Some(vec![(template.len() - "{{{s}}}".len(), template.len())])
    );
}

#[test]
fn test_template_suffix_long() {
    let template = "http://shop.zuckerzauber.at/epages/es121414.sf/de_AT/?ObjectPath=/Shops/es121414_Caros_Zuckerzauber&ViewAction=DetailSearchProducts&Search=SF-AllStrings&SearchString={{{s}}}";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (SUFFIX LONG)",
        timer.elapsed()
    );
    assert_eq!(
        indices,
        Some(vec![(template.len() - "{{{s}}}".len(), template.len())])
    );
}

#[test]
fn test_template_infix() {
    let template = "http://www.db.yugioh-card.com/{{{s}}}/card_search.action";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!("Took {:?} to get template indices (INFIX)", timer.elapsed());
    assert_eq!(indices, Some(vec![(30, 37)]));
}

#[test]
fn test_template_infix_long() {
    let template = "http://www.db.yugioh-card.com/yugiohdb/card_search.action?ope=1&sess=1&keyword={{{s}}}&stype=1&ctype=&starfr=&starto=&atkfr=&atkto=&deffr=&defto=&othercon=1";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (INFIX LONG)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(vec![(79, 86)]));
}

#[test]
fn test_template_infix_multiple() {
    let template = "http://www.db.yugioh-card.com/{{{s}}}/card_search.action?ope=1&sess=1&keyword={{{s}}}&stype=1&ctype=&starfr=&starto=&atkfr=&atkto=&deffr=&defto=&othercon=1";
    let timer = Instant::now();
    let indices = parse_template_indexes(template);
    eprintln!(
        "Took {:?} to get template indices (INFIX LONG MULTIPLE)",
        timer.elapsed()
    );
    assert_eq!(indices, Some(vec![(30, 37), (78, 85)]));
#[cfg(feature = "measure-allocs")]
mod tests {
    use super::*;

    #[test]
    fn test_empty_template_memory() {
        let alloc = allocation_counter::measure(|| {
            test_empty_template();
        });
        eprintln!(
            "`test_empty_template` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }

    #[test]
    fn test_template_infix_long_memory() {
        let alloc = allocation_counter::measure(|| {
            test_template_infix_long();
        });
        eprintln!(
            "`test_template_infix_long` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }

    #[test]
    fn test_template_suffix_long_memory() {
        let alloc = allocation_counter::measure(|| {
            test_template_suffix_long();
        });
        eprintln!(
            "`test_template_suffix_long` used a max of {} bytes and {} bytes over its lifetime",
            alloc.bytes_max, alloc.bytes_total
        )
    }
}
