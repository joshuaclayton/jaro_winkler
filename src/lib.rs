enum DataWrapper {
    Vec(Vec<bool>),
    Bitwise(u128),
}

impl DataWrapper {
    fn build(len: usize) -> Self {
        if len <= 128 {
            DataWrapper::Bitwise(0)
        } else {
            let mut internal = Vec::with_capacity(len);
            internal.extend(std::iter::repeat(false).take(len));
            DataWrapper::Vec(internal)
        }
    }

    fn get(&self, idx: usize) -> bool {
        match self {
            DataWrapper::Vec(v) => v[idx],
            DataWrapper::Bitwise(v1) => (v1 >> idx) & 1 == 1,
        }
    }

    fn set_true(&mut self, idx: usize) {
        match self {
            DataWrapper::Vec(v) => v[idx] = true,
            DataWrapper::Bitwise(v1) => *v1 |= 1 << idx,
        }
    }
}

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

    if left == right {
        return 1.0;
    }

    let range = matching_distance(s1_len, s2_len);
    let mut s1m = DataWrapper::build(s1_len);
    let mut s2m = DataWrapper::build(s2_len);
    let mut matching: f64 = 0.0;
    let mut transpositions: f64 = 0.0;
    let left_as_bytes = left.as_bytes();
    let right_as_bytes = right.as_bytes();

    for i in 0..s2_len {
        let mut j = (i as isize - range as isize).max(0) as usize;
        let l = (i + range + 1).min(s1_len);
        while j < l {
            if right_as_bytes[i] == left_as_bytes[j] && !s1m.get(j) {
                s1m.set_true(j);
                s2m.set_true(i);
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
        if s2m.get(i) {
            let mut j = l;

            while j < s1_len {
                if s1m.get(j) {
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

fn matching_distance(s1_len: usize, s2_len: usize) -> usize {
    let max = s1_len.max(s2_len) as f32;
    ((max / 2.0).floor() - 1.0) as usize
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

    macro_rules! assert_within {
        ($x:expr, $y:expr, delta=$d:expr) => {
            assert!(($x - $y).abs() <= $d)
        };
    }

    #[test]
    fn test_boundary() {
        let long_value = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s";
        let longer_value = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s";
        let result = jaro_winkler(long_value, longer_value);
        assert_within!(result, 0.82, delta = 0.01);
    }

    #[test]
    fn test_close_to_boundary() {
        let long_value = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test";
        assert_eq!(long_value.len(), 129);
        let longer_value = "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured;test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s";
        let result = jaro_winkler(long_value, longer_value);
        assert_within!(result, 0.8, delta = 0.001);
    }
}
