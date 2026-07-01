use num_bigint::{BigUint, RandBigInt as _, ToBigUint};
use num_traits::{One as _, Zero};
use rand::rngs::ThreadRng;

fn main() {
    let mr = miller_rabin(3500000000000011_u128.to_biguint().unwrap());
    println!("{}", mr);
    let mr = miller_rabin(3500000000000033_u128.to_biguint().unwrap());
    println!("{}", mr);
    let mr = miller_rabin(3500000000000059_u128.to_biguint().unwrap());
    println!("{}", mr);
    let mr = miller_rabin(
        3500000000000011_u128.to_biguint().unwrap() * 3500000000000059_u128.to_biguint().unwrap(),
    );
    println!("{}", mr);
}

fn expr(a: &BigUint, n: &BigUint, p: &BigUint) -> BigUint {
    if *n == BigUint::ZERO {
        return BigUint::one();
    } else {
        let next_n = n / 2.to_biguint().unwrap();
        let m = expr(a, &next_n, p);
        if n % 2.to_biguint().unwrap() == BigUint::zero() {
            return m.pow(2) % p;
        } else {
            return m.pow(2) * a % p;
        }
    }
}

fn miller_rabin1(p: BigUint, rng: &mut ThreadRng) -> bool {
    let low = 0.to_biguint().unwrap();
    let high = 10.to_biguint().unwrap();
    let mut n = &p - BigUint::one();
    while &n % 2.to_biguint().unwrap() == BigUint::zero() {
        n /= 2.to_biguint().unwrap();
    }
    let mut a = BigUint::one();
    for _ in 0..100 {
        let random = rng.gen_biguint_range(&low, &high);
        a = 10.to_biguint().unwrap() * a + random;
    }
    a = a % &p;
    let mut m = expr(&a, &n, &p);
    if &m == &BigUint::one() || &m == &(&p - BigUint::one()) {
        return true;
    }
    while n <= (p.clone() - BigUint::one()) {
        m = m.pow(2) % p.clone();
        if (&m + &BigUint::one()) % &p == BigUint::zero() {
            return true;
        }
        n *= 2.to_biguint().unwrap();
    }
    return false;
}

fn miller_rabin(p: BigUint) -> bool {
    let mut rng = rand::thread_rng();

    for _ in 0..20 {
        if !miller_rabin1(p.clone(), &mut rng) {
            return false;
        }
    }
    return true;
}
