use num_bigint::BigInt;
use num_traits::{One, Zero, abs};

use num_bigint::RandBigInt as _;
use p2::miller_rabin::miller_rabin;
use rand::{Rng, SeedableRng as _, rngs::SmallRng};

fn gcd(x: &BigInt, y: &BigInt) -> BigInt {
    match (x, y) {
        (x, y) if y == &BigInt::from(0) => x.clone(),
        (x, y) if x == &BigInt::from(0) => y.clone(),
        (x, y) => gcd(y, &(x % y)),
    }
}

fn g(x: &BigInt, n: &BigInt) -> BigInt {
    (x * x + 1) % n
}

fn gg(x: &BigInt, n: &BigInt) -> BigInt {
    g(&g(x, n), n)
}

fn work() {
    let n = BigInt::from(9145356185469980673640298696124239_u128);
    let mut width = BigInt::from(1);
    let _start = BigInt::from(0);
    let mut count = 0;
    let mut x0 = g(&BigInt::from(3), &n);
    let mut p = BigInt::from(1);
    loop {
        width *= &BigInt::from(2);
        println!("width = {}", &width);
        let mut x = x0.clone();

        let end = &width - &BigInt::one();
        // let i = (BigInt::zero()..(&width - &BigInt::one()));
        let ite = std::iter::successors(Some(BigInt::zero()), |x| Some(x + BigInt::one()))
            .take_while(move |x| x < &end);

        for _i in ite {
            x = g(&x, &n);
            let diff = &x - &x0;
            p = (&p * &abs(diff)) % &n;
            count += 1;
            if count % 100 == 0 {
                let k = gcd(&p, &n);
                if k != BigInt::one() {
                    println!("{k}");
                    return;
                }
                p = BigInt::one();
            }
            if count % 1000000 == 0 {
                println!("{count}");
            }
        }
        x0 = g(&x, &n)
    }
    // println!("ans = {count}");
}

fn rho_method(n: &BigInt, rng: &mut impl Rng) -> Option<BigInt> {
    let one = BigInt::one();
    let two = BigInt::from(2_u32);

    if n <= &one {
        return None;
    }

    if (n % 2_u32).is_zero() {
        return Some(two);
    }

    for _ in 0..64 {
        let _c = rng.gen_bigint_range(&one, n);
        let mut x = rng.gen_bigint_range(&two, &(n - &one));
        let mut y = x.clone();

        for _ in 0..100_000 {
            x = g(&x, n);
            y = g(&g(&y, n), n);

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
    let x = -8000..=8000 ;
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
