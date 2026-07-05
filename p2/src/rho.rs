use num_bigint::{BigInt, RandBigInt as _};
use num_traits::{One as _, Zero as _, abs};
use rand::Rng;

fn gcd(x: &BigInt, y: &BigInt) -> BigInt {
    let mut a = abs(x.clone());
    let mut b = abs(y.clone());
    while !b.is_zero() {
        let r = &a % &b;
        a = b;
        b = r;
    }
    a
}
fn g(x: &BigInt, c: &BigInt, n: &BigInt) -> BigInt {
    (x * x + c) % n
}

pub fn rho_method(n: &BigInt, rng: &mut impl Rng) -> Option<BigInt> {
    let one = BigInt::one();
    let two = BigInt::from(2_u32);
    let three = BigInt::from(3_u32);

    if n <= &BigInt::from(3_u32) {
        return None;
    }

    if (n % 2_u32).is_zero() {
        return Some(BigInt::from(2_u32));
    }
    for _ in 0..64 {
        let c = rng.gen_bigint_range(&one, n);
        let mut x = rng.gen_bigint_range(&two, &(n - &one));
        let mut y = x.clone();

        loop {
            x = g(&x, &c, n);
            y = g(&g(&y, &c, n), &c, n);

            let d = gcd(&abs(&x - &y), n);

            if d > one && d < *n {
                return Some(d);
            }

            if d == *n {
                break;
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{SeedableRng as _, rngs::SmallRng};

    fn bi(n: i128) -> BigInt {
        BigInt::from(n)
    }

    fn assert_nontrivial_factor(n: &BigInt, d: &BigInt) {
        assert!(d > &BigInt::one(), "factor must be > 1: d = {d}");
        assert!(d < n, "factor must be < n: d = {d}, n = {n}");
        assert!((n % d).is_zero(), "d must divide n: d = {d}, n = {n}");
    }

    #[test]
    fn rho_returns_none_for_too_small_inputs() {
        let mut rng = SmallRng::seed_from_u64(1);

        assert_eq!(rho_method(&bi(-10), &mut rng), None);
        assert_eq!(rho_method(&bi(-1), &mut rng), None);
        assert_eq!(rho_method(&bi(0), &mut rng), None);
        assert_eq!(rho_method(&bi(1), &mut rng), None);
    }

    #[test]
    fn rho_returns_two_for_even_composites() {
        let mut rng = SmallRng::seed_from_u64(2);

        for n in [4, 6, 8, 10, 100, 1024, 1_000_000] {
            let n = bi(n);
            let d = rho_method(&n, &mut rng).expect("rho should find factor 2");

            assert_eq!(d, bi(2));
            assert_nontrivial_factor(&n, &d);
        }
    }

    #[test]
    fn rho_finds_factor_of_small_semiprime() {
        let mut rng = SmallRng::seed_from_u64(3);

        let n = bi(91); // 7 * 13
        let d = rho_method(&n, &mut rng).expect("rho should find a factor");

        assert_nontrivial_factor(&n, &d);
        assert!(d == bi(7) || d == bi(13));
    }

    #[test]
    fn rho_finds_factor_of_another_small_semiprime() {
        let mut rng = SmallRng::seed_from_u64(4);

        let n = bi(8051); // 83 * 97
        let d = rho_method(&n, &mut rng).expect("rho should find a factor");

        assert_nontrivial_factor(&n, &d);
        assert!(d == bi(83) || d == bi(97));
    }

    #[test]
    fn rho_returns_nontrivial_factor_not_necessarily_prime() {
        let mut rng = SmallRng::seed_from_u64(5);

        let n = bi(3 * 5 * 7 * 11);
        let d = rho_method(&n, &mut rng).expect("rho should find a factor");

        assert_nontrivial_factor(&n, &d);
    }
    #[test]
    fn rho_finds_factor_of_slide_semiprime() {
        let mut rng = SmallRng::seed_from_u64(6);

        let p = BigInt::parse_bytes(b"92429849809837999", 10).unwrap();
        let q = BigInt::parse_bytes(b"98943752524593761", 10).unwrap();
        let n = &p * &q;

        let d = rho_method(&n, &mut rng).expect("rho should find a factor");

        assert_nontrivial_factor(&n, &d);
        assert!(d == p || d == q);
    }
}
