#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
use std::arch::x86_64::*;
use std::ptr;

use super::Match;

const MAX_TEMPLATE_TERMS: usize = 2;

#[inline(always)]
fn merge_slices(a: &mut [Match], b: &[Match]) {
    let mut a_idx = 0usize;
    let mut b_idx = 0usize;

    let a_len = a.len();
    let b_len = b.len();

    unsafe {
        while a_idx < a_len && b_idx < b_len {
            let a_ptr = a.as_mut_ptr().add(a_idx);
            let b_val = *b.get_unchecked(b_idx);

            if *a_ptr == Match::new(0, 0) {
                ptr::write(a_ptr, b_val);
                b_idx += 1;
            }
            a_idx += 1;
        }
    }
}

#[inline(always)]
fn iterative_parse_template_indexes(template: &str) -> Option<[Match; MAX_TEMPLATE_TERMS]> {
    let mut results = [Match::new(0, 0); MAX_TEMPLATE_TERMS];
    let mut idx = 0usize;
    let mut slice_index = 0;

    while let Some(start_index) = template[slice_index..].find("{{{s}}}") {
        let start = start_index + slice_index;
        let end = (start + "{{{s}}}".len()).min(template.len());
        results[idx] = Match::new(start, end);

        // Move the slice_index forward to continue searching after the current match
        slice_index = end;
        idx += 1;
    }

    if results[0].is_empty() {
        None
    } else {
        Some(results)
    }
}

#[inline(always)]
pub fn parse_template_indexes(template: &str) -> Option<[Match; MAX_TEMPLATE_TERMS]> {
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

            let mut results = [Match::new(0, 0); MAX_TEMPLATE_TERMS];
            let mut res_idx = 0usize;

            for chunk in aligned {
                let simd_chunk = unsafe { _mm_load_si128(chunk.as_ptr() as *const __m128i) };
                let eq_chunk = unsafe { _mm_cmpeq_epi8(mask, simd_chunk) };
                let mask_chunk = unsafe { _mm_movemask_epi8(eq_chunk) };

                if mask_chunk != 0 {
                    let idx: usize = offset + mask_chunk.trailing_zeros() as usize;
                    if &template[idx..idx + "{{{s}}}".len()] == "{{{s}}}" {
                        results[res_idx] = Match::new(idx, idx + 7);
                        res_idx += 1;
                    }
                }

                offset += 16;
            }

            if let Some(suffix_pairs) = iterative_parse_template_indexes(
                &template[template.len() - suffix.len()..],
            )
            .map(|offset_pairs| {
                let mut result = [Match::new(0, 0); MAX_TEMPLATE_TERMS]; // Preallocate stack memory
                for (i, Match { start, end }) in
                    offset_pairs.iter().enumerate().take(MAX_TEMPLATE_TERMS)
                {
                    if *end == 0 {
                        continue;
                    }
                    result[i] = Match::new(offset + start, offset + end);
                }
                result
            }) {
                merge_slices(&mut results, &suffix_pairs);
            }

            if results[0].is_empty() {
                None
            } else {
                Some(results)
            }
        }
        38.. => {
            let (prefix, aligned, suffix) = unsafe { template.as_bytes().align_to::<[u8; 32]>() };
            let mut offset = prefix.len();

            let mask = unsafe { _mm256_set1_epi8('{' as i8) };

            let mut results = [Match::new(0, 0); MAX_TEMPLATE_TERMS];
            let mut res_idx = 0usize;

            for chunk in aligned {
                let simd_chunk = unsafe { _mm256_load_si256(chunk.as_ptr() as *const __m256i) };
                let eq_chunk = unsafe { _mm256_cmpeq_epi8(mask, simd_chunk) };
                let mask_chunk = unsafe { _mm256_movemask_epi8(eq_chunk) };

                if mask_chunk != 0 {
                    let idx: usize = offset + mask_chunk.trailing_zeros() as usize;
                    if &template[idx..idx + "{{{s}}}".len()] == "{{{s}}}" {
                        results[res_idx] = Match::new(idx, idx + 7);
                        res_idx += 1;
                    }
                }

                offset += 32;
            }

            if let Some(suffix_pairs) = iterative_parse_template_indexes(
                &template[template.len() - suffix.len()..],
            )
            .map(|offset_pairs| {
                let mut result = [Match::new(0, 0); MAX_TEMPLATE_TERMS]; // Preallocate stack memory
                for (i, Match { start, end }) in
                    offset_pairs.iter().enumerate().take(MAX_TEMPLATE_TERMS)
                {
                    if *end == 0 {
                        continue;
                    }
                    result[i] = Match::new(offset + start, offset + end);
                }
                result
            }) {
                merge_slices(&mut results, &suffix_pairs);
            }

            if results[0].is_empty() {
                None
            } else {
                Some(results)
            }
        }
    };

    iterative_parse_template_indexes(template)
}
