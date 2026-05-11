use std::collections::HashMap;

/// n = a^2 + b^2 + c^2 + d^2 となる非負整数の組を1つ返す。
///
/// n は正の整数を想定する。
pub fn four_squares(n: u64) -> (u64, u64, u64, u64) {
    let limit = (n as f64).sqrt() as u64;

    // sum -> (a, b)
    // a^2 + b^2 = sum となる組を保存する
    let mut table: HashMap<u64, (u64, u64)> = HashMap::new();

    for a in 0..=limit {
        let aa = a * a;

        for b in a..=limit {
            let sum = aa + b * b;

            if sum > n {
                break;
            }

            table.entry(sum).or_insert((a, b));
        }
    }

    for c in 0..=limit {
        let cc = c * c;

        for d in c..=limit {
            let sum = cc + d * d;

            if sum > n {
                break;
            }

            let rest = n - sum;

            if let Some(&(a, b)) = table.get(&rest) {
                return (a, b, c, d);
            }
        }
    }

    unreachable!("Lagrange's four-square theorem guarantees a solution");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_a_valid_representation_for_small_known_values() {
        // 正しい組は複数あり得るため、返り値そのものではなく二乗和を検査する。
        // ここでは境界に近い小さい値と、平方数を含む代表的な値を確認する。
        for n in [1, 2, 3, 4, 5, 10, 25, 50, 99, 310] {
            assert_represents(n, four_squares(n));
        }
    }

    #[test]
    fn returns_a_valid_representation_for_a_contiguous_range() {
        // ラグランジュの四平方定理により正の整数には必ず解がある。
        // 連続した範囲を検査して、特定の形の入力だけに偶然通る実装を避ける。
        for n in 1..=1_000 {
            assert_represents(n, four_squares(n));
        }
    }

    fn assert_represents(n: u64, tuple: (u64, u64, u64, u64)) {
        let (a, b, c, d) = tuple;
        // 検査側の計算でオーバーフローしないように、二乗和は u128 で計算する。
        let actual = square(a) + square(b) + square(c) + square(d);

        assert_eq!(
            u128::from(n),
            actual,
            "expected {n}, got {tuple:?} whose square sum is {actual}"
        );
    }

    fn square(value: u64) -> u128 {
        let value = u128::from(value);
        value * value
    }
}
