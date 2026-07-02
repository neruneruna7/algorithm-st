use num_bigint::BigInt;
use num_traits::{One, Zero, abs};

use std::cell::RefCell;

use num_bigint::RandBigInt as _;
use rand::{Rng, SeedableRng as _, rngs::SmallRng};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

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
    let result_1 = a.modpow(&d, &p) != BigInt::one();
    // a^d, a^{2 d}, a^{4 d} ...
    //
    let result_2 = (0..s)
        // 指数の計算
        .map(|i| 2_usize.pow(i) * &d)
        // 0 <= forall i < s , a^{2^i d} != -1 (mod p)　を計算
        .all(|exp| a.modpow(&exp, &p) != p_minus_1);
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

fn miller_rabin(p: &BigInt, itertions: usize, rng: &mut impl Rng) -> bool {
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
    return true;
}

fn gcd(x: &BigInt, y: &BigInt) -> BigInt {
    match (x, y) {
        (x, y) if y == &BigInt::from(0) => x.clone(),
        (x, y) if x == &BigInt::from(0) => y.clone(),
        (x, y) => gcd(&y, &(x % y)),
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
    let mut start = BigInt::from(0);
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

        for i in ite {
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

fn rho_method(n: &BigInt) -> BigInt {
    let mut width = BigInt::from(1);
    let mut start = BigInt::from(0);
    let mut count = 0;
    let mut x0 = g(&BigInt::from(3), n);
    let mut p = BigInt::from(1);
    loop {
        width *= &BigInt::from(2);
        println!("width = {}", &width);
        let mut x = x0.clone();

        let end = &width - &BigInt::one();
        // let i = (BigInt::zero()..(&width - &BigInt::one()));
        let ite = std::iter::successors(Some(BigInt::zero()), |x| Some(x + BigInt::one()))
            .take_while(move |x| x < &end);

        for _ in ite {
            x = g(&x, &n);
            let diff = &x - &x0;
            p = (&p * &abs(diff)) % n;
            count += 1;
            if count % 100 == 0 {
                let k = gcd(&p, &n);
                if k != BigInt::one() {
                    // println!("{k}");
                    return k;
                }
                p = BigInt::one();
            }
            // if count % 1000000 == 0 {
            //     println!("{count}");
            // }
        }
        x0 = g(&x, &n)
    }
    // println!("ans = {count}");
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
    let zero = BigInt::zero();
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
            let d = rho_method(&x);

            if d > one && d < x {
                break d;
            }

            // d == x または d == 1 の場合は Rho 失敗である.
            // 本来は rho_method に乱数を渡してパラメータを変えるべきである.
        };

        let q = &x / &d;

        stack.push(d);
        stack.push(q);
    }

    factors.sort();
    factors
}

// fn quadratic_sieve(n: BigInt) {
//     // nの平方根を取り，それをmとする
//     let m = n.clone().sqrt();
//     let x = (-8000..=8000);
//     let q_x_vec = x
//         // Q(x) = (m + x)^2  n
//         .map(|x| (&m + BigInt::from(x)).pow(2) - &n)
//     // それぞれに素因酢分解する
//         .map(|qx| {

//         })

//     todo!()
// }

fn main() {
    let q = BigInt::from(914535618546997293219643669199126899_u128);
    // println!("{q}");
    // // todo!("課題2")
    // work();

    let grotaan = BigInt::from(57);
    let factors = prime_factorize(&grotaan, &mut SmallRng::seed_from_u64(0));
    println!("{factors:?}");
}
