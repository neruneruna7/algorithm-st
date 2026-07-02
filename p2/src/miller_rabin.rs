use num_bigint::BigInt;
use num_traits::{One, Zero};

use num_bigint::RandBigInt as _;
use rand::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MRResult {
    MayBePrime,
    Composite,
}

fn my_miller_rabin1(p: &BigInt, rng: &mut impl Rng) -> MRResult {
    // pを，p-1 = 2^s * d　に分解
    let p_minus_1 = p - BigInt::one();
    let (d, s) = decompose(&p_minus_1);
    // [1, p-1]の範囲からランダムにaを選ぶ
    let a = rng.gen_bigint_range(&BigInt::from(1_usize), &p_minus_1);
    // a^d != 1 (mod p)
    // かつ
    // 0 <= forall i < s , a^{2^i d} != -1 (mod p)
    // mod p の -1 なので，これは　p-1　である
    // であるならば，合成数
    // 出なければ，多分素数

    // a^d != 1 (mod p)
    let result_1 = a.modpow(&d, p) != BigInt::one();
    // a^d, a^{2 d}, a^{4 d} ...
    //
    let result_2 = (0..s)
        // 指数の計算
        .map(|i| 2_usize.pow(i) * &d)
        // 0 <= forall i < s , a^{2^i d} != -1 (mod p)　を計算
        .all(|exp| a.modpow(&exp, p) != p_minus_1);
    // let x0 = a.modpow(&d, &p);
    // let y = std::iter::successors(Some(x0), |x| Some((x * x) % &p))
    //     .take(s as usize)
    //     .any(|x| x == p_minus_1);

    if result_1 && result_2 {
        MRResult::Composite
    } else {
        MRResult::MayBePrime
    }
}

fn decompose(p: &BigInt) -> (BigInt, u32) {
    let two = BigInt::from(2_u32);

    let mut s = 0;
    let mut d = p.clone();
    while (&d % &two).is_zero() {
        d = &d / &two;
        s += 1;
    }

    (d, s)
}

pub fn miller_rabin(p: &BigInt, itertions: usize, rng: &mut impl Rng) -> bool {
    if p < &BigInt::from(2) {
        return false;
    }

    if p == &BigInt::from(2) || p == &BigInt::from(3) {
        return true;
    }

    if (p % 2_u32).is_zero() {
        return false;
    }
    for _ in 0..itertions {
        let my_result = my_miller_rabin1(p, rng);
        if my_result == MRResult::Composite {
            return false;
        }
    }
    true
}

// fn miller_rabin(p: BigInt, itertions: usize, rng: &mut impl Rng) -> bool {
//     for _ in 0..itertions {
//         let my_result = my_miller_rabin1(p.clone(), rng);
//         if my_result == MRResult::Composite {
//             return false;
//         }
//     }
//     return true;
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng as _, rngs::SmallRng};

    fn bigint(n: i128) -> BigInt {
        BigInt::from(n)
    }

    #[test]
    fn miller_rabin_rejects_numbers_less_than_2() {
        let mut rng = SmallRng::seed_from_u64(1);

        assert!(!miller_rabin(&bigint(-10), 20, &mut rng));
        assert!(!miller_rabin(&bigint(-1), 20, &mut rng));
        assert!(!miller_rabin(&bigint(0), 20, &mut rng));
        assert!(!miller_rabin(&bigint(1), 20, &mut rng));
    }

    #[test]
    fn miller_rabin_accepts_2_and_3() {
        let mut rng = SmallRng::seed_from_u64(2);

        assert!(miller_rabin(&bigint(2), 20, &mut rng));
        assert!(miller_rabin(&bigint(3), 20, &mut rng));
    }

    #[test]
    fn miller_rabin_rejects_even_composites() {
        let mut rng = SmallRng::seed_from_u64(3);

        for n in [4, 6, 8, 10, 100, 1024, 1_000_000] {
            assert!(!miller_rabin(&bigint(n), 20, &mut rng), "n = {n}");
        }
    }

    #[test]
    fn miller_rabin_accepts_small_primes() {
        let mut rng = SmallRng::seed_from_u64(4);

        let primes = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97,
        ];

        for p in primes {
            assert!(miller_rabin(&bigint(p), 20, &mut rng), "p = {p}");
        }
    }

    #[test]
    fn miller_rabin_rejects_small_odd_composites() {
        let mut rng = SmallRng::seed_from_u64(5);

        let composites = [
            9, 15, 21, 25, 27, 33, 35, 39, 45, 49, 51, 55, 57, 63, 65, 69, 75, 77, 81, 85, 87, 91,
            93, 95, 99,
        ];

        for n in composites {
            assert!(!miller_rabin(&bigint(n), 20, &mut rng), "n = {n}");
        }
    }
    #[test]
    fn miller_rabin_rejects_carmichael_numbers() {
        let mut rng = SmallRng::seed_from_u64(6);

        let carmichael_numbers = [
            561_i128, 1105, 1729, 2465, 2821, 6601, 8911, 10585, 15841, 29341, 41041, 46657, 52633,
            62745, 63973,
        ];

        for n in carmichael_numbers {
            assert!(!miller_rabin(&BigInt::from(n), 20, &mut rng), "n = {n}");
        }
    }
    #[test]
    fn miller_rabin_handles_slide_values() {
        let mut rng = SmallRng::seed_from_u64(7);

        let a = BigInt::parse_bytes(b"92429849809837999", 10).unwrap();
        let b = BigInt::parse_bytes(b"98943752524593761", 10).unwrap();
        let n = &a * &b;

        assert!(miller_rabin(&a, 20, &mut rng));
        assert!(miller_rabin(&b, 20, &mut rng));
        assert!(!miller_rabin(&n, 20, &mut rng));
    }
}
