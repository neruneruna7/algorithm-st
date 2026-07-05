use num_traits::{Zero as _, abs};
use rand::Rng;

fn gcd(x: i128, y: i128) -> i128 {
    let mut a = abs(x);
    let mut b = abs(y);
    while !b.is_zero() {
        let r = &a % &b;
        a = b;
        b = r;
    }
    a
}
fn g(x: i128, c: i128, n: i128) -> i128 {
    (x * x + c) % n
}

pub fn rho_method(n: i128, rng: &mut impl Rng) -> Option<i128> {
    if n <= i128::from(3_u32) {
        return None;
    }

    if (n % 2).is_zero() {
        return Some(2);
    }
    for _ in 0..64 {
        let c = rng.gen_range(1..n);
        let mut x = rng.gen_range(2..n);
        let mut y = x;

        let mut count = 0;
        loop {
            count += 1;
            if count % 1000 == 0 {
                println!("count = {count}");
            }

            x = g(x, c, n);
            y = g(g(y, c, n), c, n);

            let d = gcd(abs(x - y), n);

            if d > 1 && d < n {
                return Some(d);
            }

            if d == n {
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

    fn assert_nontrivial_factor(n: i128, d: i128) {
        assert!(d > 1, "factor must be > 1: d = {d}");
        assert!(d < n, "factor must be < n: d = {d}, n = {n}");
        assert!((n % d).is_zero(), "d must divide n: d = {d}, n = {n}");
    }

    #[test]
    fn rho_returns_none_for_too_small_inputs() {
        let mut rng = SmallRng::seed_from_u64(1);

        assert_eq!(rho_method(-10, &mut rng), None);
        assert_eq!(rho_method(-1, &mut rng), None);
        assert_eq!(rho_method(0, &mut rng), None);
        assert_eq!(rho_method(1, &mut rng), None);
    }

    #[test]
    fn rho_returns_two_for_even_composites() {
        let mut rng = SmallRng::seed_from_u64(2);

        for n in [4, 6, 8, 10, 100, 1024, 1_000_000] {
            let d = rho_method(n, &mut rng).expect("rho should find factor 2");

            assert_eq!(d, 2);
            assert_nontrivial_factor(n, d);
        }
    }

    #[test]
    fn rho_finds_factor_of_small_semiprime() {
        let mut rng = SmallRng::seed_from_u64(3);

        let n = 91; // 7 * 13
        let d = rho_method(n, &mut rng).expect("rho should find a factor");

        assert_nontrivial_factor(n, d);
        assert!(d == 7 || d == 13);
    }

    #[test]
    fn rho_finds_factor_of_another_small_semiprime() {
        let mut rng = SmallRng::seed_from_u64(4);

        let n = 8051; // 83 * 97
        let d = rho_method(n, &mut rng).expect("rho should find a factor");

        assert_nontrivial_factor(n, d);
        assert!(d == 83 || d == 97);
    }

    #[test]
    fn rho_returns_nontrivial_factor_not_necessarily_prime() {
        let mut rng = SmallRng::seed_from_u64(5);

        let n = 3 * 5 * 7 * 11;
        let d = rho_method(n, &mut rng).expect("rho should find a factor");

        assert_nontrivial_factor(n, d);
    }
    // #[test]
    // fn rho_finds_factor_of_slide_semiprime() {
    //     let mut rng = SmallRng::seed_from_u64(6);

    //     let p = i128::parse_bytes(b"92429849809837999", 10).unwrap();
    //     let q = i128::parse_bytes(b"98943752524593761", 10).unwrap();
    //     let n = &p * &q;

    //     let d = rho_method(&n, &mut rng).expect("rho should find a factor");

    //     assert_nontrivial_factor(&n, &d);
    //     assert!(d == p || d == q);
    // }
}
