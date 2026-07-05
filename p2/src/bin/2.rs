use num_bigint::BigInt;

use p2::{miller_rabin::miller_rabin, rho::pollard_rho};
use rand::{SeedableRng as _, rngs::SmallRng};

fn main() {
    // let n: i128 = 11629360743077306442685712558623;
    let n = 914535618546997293219643669199126899_i128;

    println!("n = {n}");
    let factor = pollard_rho(n);
    println!("rho = {factor:?}");

    let prime_factors = prime_factorize(n);
    println!("{prime_factors:?}");
}

/// 素因数分解
fn prime_factorize(n: i128) -> Vec<i128> {
    if n == 0 {
        panic!("0 cannot be prime-factorized");
    }

    let mut factors = Vec::new();
    let mut stack = Vec::new();

    if n < 0 {
        factors.push(-1);
        stack.push(-n);
    } else {
        stack.push(n);
    }

    let mut rng = SmallRng::seed_from_u64(1);
    // スタック型で管理して，木構造の再帰的な処理を行う．
    while let Some(n) = stack.pop() {
        if n == 1 {
            continue;
        }

        if miller_rabin(&BigInt::from(n), 20, &mut rng) {
            factors.push(n);
            continue;
        }

        let d = pollard_rho(n).expect("failed to find a factor");

        if d <= 1 || d >= n || n % d != 0 {
            panic!("invalid factor found: d = {d}, n = {n}");
        }

        stack.push(d);
        stack.push(n / d);
    }

    factors.sort();
    factors
}
