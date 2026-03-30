#![deny(missing_docs)]

//! `jaro_winkler` is a crate for calculating Jaro-Winkler distance of two strings.
//!
//! # Examples
//!
//! ```
//! use jaro_winkler::jaro_winkler;
//!
//! assert_eq!(jaro_winkler("martha", "marhta"), 0.9611111111111111);
//! assert_eq!(jaro_winkler("", "words"), 0.0);
//! assert_eq!(jaro_winkler("same", "same"), 1.0);
//! ```

/// Calculates the Jaro-Winkler distance of two strings.
///
/// The return value is between 0.0 and 1.0, where 1.0 means the strings are equal.
pub fn jaro_winkler(left_: &str, right_: &str) -> f64 {
    let llen = left_.len();
    let rlen = right_.len();

    let (left, right, s1_len, s2_len) = if llen < rlen {
        (right_, left_, rlen, llen)
    } else {
        (left_, right_, llen, rlen)
    };

    match (s1_len, s2_len) {
        (0, 0) => return 1.0,
        (0, _) | (_, 0) => return 0.0,
        (_, _) => (),
    }

    let left_as_bytes = left.as_bytes();
    let right_as_bytes = right.as_bytes();

    if s1_len == s2_len && left_as_bytes == right_as_bytes {
        return 1.0;
    }

    let range = matching_distance(s1_len, s2_len);

    if s1_len > 128 {
        return jaro_winkler_long(left_as_bytes, right_as_bytes, s1_len, s2_len, range);
    }

    // Both strings fit in 128 bits — use raw u128 bitmasks directly,
    // no enum dispatch overhead.
    let mut s1m: u128 = 0;
    let mut s2m: u128 = 0;
    let mut matching: f64 = 0.0;
    let mut transpositions: f64 = 0.0;

    for i in 0..s2_len {
        let mut j = i.saturating_sub(range);
        let l = (i + range + 1).min(s1_len);
        while j < l {
            if right_as_bytes[i] == left_as_bytes[j] && (s1m >> j) & 1 != 1 {
                s1m |= 1 << j;
                s2m |= 1 << i;
                matching += 1.0;
                break;
            }

            j += 1;
        }
    }

    if matching == 0.0 {
        return 0.0;
    }

    let mut l = 0;

    for i in 0..s2_len - 1 {
        if (s2m >> i) & 1 == 1 {
            let mut j = l;

            while j < s1_len {
                if (s1m >> j) & 1 == 1 {
                    l = j + 1;
                    break;
                }

                j += 1;
            }

            if right_as_bytes[i] != left_as_bytes[j] {
                transpositions += 1.0;
            }
        }
    }
    transpositions = (transpositions / 2.0).ceil();

    winkler(
        matching,
        transpositions,
        s1_len,
        s2_len,
        left_as_bytes,
        right_as_bytes,
    )
}

/// Bit-parallel Jaro-Winkler for strings where s1_len > 128.
///
/// Instead of scanning the match window byte-by-byte, we precompute a bitmask
/// for each byte value recording all positions where it occurs in the longer
/// string. Matching then reduces to bitwise AND/NOT operations per 64-bit word,
/// replacing ~range byte comparisons with ~(range/64) word operations.
fn jaro_winkler_long(
    left_as_bytes: &[u8],
    right_as_bytes: &[u8],
    s1_len: usize,
    s2_len: usize,
    range: usize,
) -> f64 {
    let s1_words = (s1_len + 63) >> 6;
    let s2_words = (s2_len + 63) >> 6;

    // Precompute: for each byte value, a bitmask of positions in left where it occurs
    let mut char_masks = vec![0u64; 256 * s1_words];
    for (j, &b) in left_as_bytes.iter().enumerate() {
        char_masks[(b as usize) * s1_words + (j >> 6)] |= 1u64 << (j & 63);
    }

    let mut s1m = vec![0u64; s1_words];
    let mut s2m = vec![0u64; s2_words];
    let mut matching: f64 = 0.0;

    for i in 0..s2_len {
        let target = right_as_bytes[i] as usize;
        let j_start = i.saturating_sub(range);
        let j_end = (i + range + 1).min(s1_len);

        let start_word = j_start >> 6;
        let end_word = ((j_end - 1) >> 6) + 1;

        for w in start_word..end_word.min(s1_words) {
            let mut candidates = char_masks[target * s1_words + w] & !s1m[w];

            // Mask out bits outside the match window in boundary words
            let lo = if w == start_word { j_start & 63 } else { 0 };
            let hi = if w == end_word - 1 {
                let b = j_end & 63;
                if b == 0 {
                    64
                } else {
                    b
                }
            } else {
                64
            };

            if lo > 0 {
                candidates &= !((1u64 << lo) - 1);
            }
            if hi < 64 {
                candidates &= (1u64 << hi) - 1;
            }

            if candidates != 0 {
                let bit = candidates.trailing_zeros();
                s1m[w] |= 1u64 << bit;
                s2m[i >> 6] |= 1u64 << (i & 63);
                matching += 1.0;
                break;
            }
        }
    }

    if matching == 0.0 {
        return 0.0;
    }

    // Count transpositions
    let mut transpositions: f64 = 0.0;
    let mut l = 0usize;

    for i in 0..s2_len - 1 {
        if (s2m[i >> 6] >> (i & 63)) & 1 == 1 {
            let mut j = l;
            while j < s1_len {
                if (s1m[j >> 6] >> (j & 63)) & 1 == 1 {
                    l = j + 1;
                    break;
                }
                j += 1;
            }
            if right_as_bytes[i] != left_as_bytes[j] {
                transpositions += 1.0;
            }
        }
    }
    transpositions = (transpositions / 2.0).ceil();

    winkler(
        matching,
        transpositions,
        s1_len,
        s2_len,
        left_as_bytes,
        right_as_bytes,
    )
}

fn winkler(
    matching: f64,
    transpositions: f64,
    s1_len: usize,
    s2_len: usize,
    left_as_bytes: &[u8],
    right_as_bytes: &[u8],
) -> f64 {
    let jaro = (matching / (s1_len as f64)
        + matching / (s2_len as f64)
        + (matching - transpositions) / matching)
        / 3.0;

    let prefix_length = left_as_bytes
        .iter()
        .zip(right_as_bytes)
        .take(4)
        .take_while(|(l, r)| l == r)
        .count() as f64;

    jaro + prefix_length * 0.1 * (1.0 - jaro)
}

fn matching_distance(s1_len: usize, s2_len: usize) -> usize {
    (s1_len.max(s2_len) / 2).saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_is_zero() {
        assert_eq!(jaro_winkler("foo", "bar"), 0.0);
    }

    #[test]
    fn same_is_one() {
        assert_eq!(jaro_winkler("foo", "foo"), 1.0);
        assert_eq!(jaro_winkler("", ""), 1.0);
    }

    #[test]
    fn test_hello() {
        assert_eq!(jaro_winkler("hell", "hello"), 0.96);
    }

    /// Reference implementation using the standard (non-bit-parallel) algorithm.
    /// Used to validate the optimized bit-parallel path produces identical results.
    fn jaro_winkler_reference(left_: &str, right_: &str) -> f64 {
        let llen = left_.len();
        let rlen = right_.len();
        let (left, right, s1_len, s2_len) = if llen < rlen {
            (right_, left_, rlen, llen)
        } else {
            (left_, right_, llen, rlen)
        };
        match (s1_len, s2_len) {
            (0, 0) => return 1.0,
            (0, _) | (_, 0) => return 0.0,
            _ => (),
        }
        let left_as_bytes = left.as_bytes();
        let right_as_bytes = right.as_bytes();
        if s1_len == s2_len && left_as_bytes == right_as_bytes {
            return 1.0;
        }
        let range = (s1_len.max(s2_len) / 2).saturating_sub(1);
        let mut s1m = vec![false; s1_len];
        let mut s2m = vec![false; s2_len];
        let mut matching: f64 = 0.0;
        let mut transpositions: f64 = 0.0;
        for i in 0..s2_len {
            let j_start = (i as isize - range as isize).max(0) as usize;
            let j_end = (i + range + 1).min(s1_len);
            for j in j_start..j_end {
                if right_as_bytes[i] == left_as_bytes[j] && !s1m[j] {
                    s1m[j] = true;
                    s2m[i] = true;
                    matching += 1.0;
                    break;
                }
            }
        }
        if matching == 0.0 {
            return 0.0;
        }
        let mut l = 0;
        for i in 0..s2_len - 1 {
            if s2m[i] {
                let mut j = l;
                while j < s1_len {
                    if s1m[j] {
                        l = j + 1;
                        break;
                    }
                    j += 1;
                }
                if right_as_bytes[i] != left_as_bytes[j] {
                    transpositions += 1.0;
                }
            }
        }
        transpositions = (transpositions / 2.0).ceil();
        let jaro = (matching / (s1_len as f64)
            + matching / (s2_len as f64)
            + (matching - transpositions) / matching)
            / 3.0;
        let prefix_length = left_as_bytes
            .iter()
            .zip(right_as_bytes)
            .take(4)
            .take_while(|(l, r)| l == r)
            .count() as f64;
        jaro + prefix_length * 0.1 * (1.0 - jaro)
    }

    /// Validate the optimized jaro_winkler (including bit-parallel path for
    /// long strings) produces identical results to the reference implementation
    /// across a wide range of inputs. Also verifies order independence.
    #[test]
    fn cross_validate_optimized_vs_reference() {
        let pairs: Vec<(&str, &str)> = vec![
            ("", ""),
            ("", "a"),
            ("a", ""),
            ("a", "a"),
            ("a", "b"),
            ("foo", "bar"),
            ("foo", "foo"),
            ("hell", "hello"),
            ("martha", "marhta"),
            ("wonderful", "wonderment"),
            ("hello hi what is going on", "hell"),
            // Long: exercises jaro_winkler_long (223 chars)
            (
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s",
                "wonderment double double",
            ),
            // Both long: both > 128
            (
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s",
                "; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s",
            ),
            // Boundary: exactly 129 chars
            (
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test",
                "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s",
            ),
            // Short left, long right (forces swap)
            (
                "abc",
                "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz0123456789",
            ),
            // Both very long, mostly different
            (
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            ),
        ];

        for (a, b) in &pairs {
            let optimized = jaro_winkler(a, b);
            let reference = jaro_winkler_reference(a, b);
            assert!(
                (optimized - reference).abs() < 1e-10,
                "Mismatch for ({:?}, {:?}): optimized={}, reference={}",
                a,
                b,
                optimized,
                reference
            );
            // Verify order independence
            let optimized_flipped = jaro_winkler(b, a);
            assert!(
                (optimized_flipped - reference).abs() < 1e-10,
                "Mismatch (flipped) for ({:?}, {:?}): optimized={}, reference={}",
                b,
                a,
                optimized_flipped,
                reference
            );
        }
    }
}
