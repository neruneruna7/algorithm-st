use num_bigint::BigInt;
use num_traits::{One, Zero, abs};

use num_bigint::RandBigInt as _;
use p2::{miller_rabin::miller_rabin, rho::rho_method};
use rand::{Rng, SeedableRng as _, rngs::SmallRng};

// 素因数分解を再帰で実装する
// fn prime_factorize(n: &BigInt, rng: &mut impl Rng) -> Vec<BigInt> {
//     let is_prime = miller_rabin(n, 20, rng);
//     if is_prime {
//         return vec![n.clone()];
//     }
//     let left_factor = rho_method(n);
//     let right_factor = n / &left_factor;
//     let left_factors = prime_factorize(&left_factor, rng);
//     let right_factors = prime_factorize(&right_factor, rng);
//     left_factors
//         .into_iter()
//         .chain(right_factors.into_iter())
//         .collect()
// }
//
fn prime_factorize(n: &BigInt, rng: &mut impl Rng) -> Vec<BigInt> {
    let one = BigInt::one();
    let two = BigInt::from(2_u32);

    let mut stack = vec![n.clone()];
    let mut factors = Vec::new();

    while let Some(x) = stack.pop() {
        if x < two {
            continue;
        }

        if x == two {
            factors.push(two.clone());
            continue;
        }

        if (&x % 2_u32).is_zero() {
            factors.push(two.clone());
            stack.push(&x / 2_u32);
            continue;
        }

        if miller_rabin(&x, 20, rng) {
            factors.push(x);
            continue;
        }

        let d = loop {
            match rho_method(&x, rng) {
                Some(d) if d > one && d < x => break d,
                _ => continue,
            }
        };

        let q = &x / &d;

        stack.push(d);
        stack.push(q);
    }

    factors.sort();
    factors
}
fn quadratic_sieve(n: BigInt) {
    let mut rng = SmallRng::seed_from_u64(48);
    // nの平方根を取り，それをmとする
    let m = n.clone().sqrt();
    let x = -8000..=8000;
    let _q_x_vec = x
        // Q(x) = (m + x)^2  n
        .map(|x| (&m + BigInt::from(x)).pow(2) - &n)
        // それぞれに素因数分解する
        .map(|qx| prime_factorize(&qx, &mut rng))
        .collect::<Vec<_>>();

    todo!()
}

fn main() {
    let q = BigInt::from(914535618546997293219643669199126899_u128);
    // println!("{q}");
    // // todo!("課題2")
    // work();

    let _grotaan = BigInt::from(57);
    let factors = prime_factorize(&q, &mut SmallRng::seed_from_u64(48));
    println!("{factors:?}");
}
