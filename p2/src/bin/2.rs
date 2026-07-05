use num_bigint::BigInt;

use num_integer::Integer as _;
use num_traits::{One as _, Signed as _, ToPrimitive as _, Zero as _};
use p2::{
    miller_rabin::miller_rabin,
    qs2::quadratic_sieve,
    rho::{self, rho_method},
    rho2,
};
use rand::{SeedableRng as _, rngs::SmallRng};

fn main() {
    // let q = BigInt::from(914535618546997293219643669199126899_u128);
    // let _q = BigInt::from(128);
    // let q = BigInt::from(405003390007_u128);
    // let q = BigInt::from(2238345871633717_u128);
    let q = BigInt::from(11629360743077306442685712558623_u128);
    let qu = 11629360743077306442685712558623_i128;

    // 5195515532454019
    // let qd =
    //     BigInt::from(11629360743077306442685712558623_u128) / BigInt::from(5195515532454019_u128);
    // println!("qd = {}", qd);
    // println!("{q}");
    // // todo!("課題2")
    // work();
    let mut rng = SmallRng::seed_from_u64(1);

    // let primes = primes_leq(20000);
    // let d = quadratic_sieve1(&qd, 8000000, &primes);
    // let d = quadratic_sieve(&q, 2000, 80000);
    // let d = quadratic_sieve_adaptive(&q, 20000, 8000000);
    // let d = quadratic_sieve(&q);
    // println!("{d:?}");
    // let rho = rho_method(&q, &mut rng);
    let rho = rho::rho_method(qu, &mut rng);
    println!("rho = {rho:?}");

    // println!("n = {}", &q);
    // let prime_factors = prime_factorize(&q);
    // println!("{prime_factors:?}");
}

// 素因数分解

fn prime_factorize(n: &BigInt) -> Vec<BigInt> {
    let one = BigInt::one();
    // 0 は素因数分解できない．
    if n.is_zero() {
        panic!("0 cannot be prime-factorized");
    }

    // 負数は -1 を外に出して，絶対値を分解する．
    if n.is_negative() {
        let mut factors = vec![BigInt::from(-1)];
        factors.extend(prime_factorize(&(-n)));
        return factors;
    }

    // 1 は空積として扱う．
    if *n == one {
        return vec![];
    }

    // 素数なら，その時点で返す．
    let mut rng = SmallRng::seed_from_u64(1);

    if miller_rabin(n, 20, &mut rng) {
        return vec![n.clone()];
    }

    // 小さい偶数は先に処理する．
    // これを入れないと，rho や QS に不要な負荷がかかる．
    if n.is_even() {
        let two = BigInt::from(2);
        let mut factors = vec![two.clone()];
        factors.extend(prime_factorize(&(n / &two)));
        factors.sort();
        return factors;
    }

    // 可能なら，QS より先に Pollard rho を試す方がよい．
    // 関数名は貴公の実装に合わせて変更すること．
    //
    // 例:
    // if let Some(factor) = pollard_rho(n) {
    //     if factor > one && factor < *n {
    //         let mut factors = prime_factorize(&factor);
    //         factors.extend(prime_factorize(&(n / &factor)));
    //         factors.sort();
    //         return factors;
    //     }
    // }
    // 2次ふるい法で，何か因数を1つ見つける．
    // これは素因数とは限らない．
    let n_i128 = n
        .to_i128()
        .unwrap_or_else(|| panic!("quadratic_sieve requires an i128-sized input: {n}"));
    let factor = quadratic_sieve(&n_i128)
        .filter(|f| *f > 1 && *f < n_i128 && n_i128 % *f == 0)
        .map(BigInt::from)
        .unwrap_or_else(|| {
            panic!("quadratic_sieve failed to find a non-trivial factor for {n}");
        });
    let cofactor = n / &factor;
    // factor と cofactor をそれぞれ再帰的に素因数分解する．
    let mut factors = prime_factorize(&factor);
    factors.extend(prime_factorize(&cofactor));
    // 出力を安定させる．
    factors.sort();
    factors
}
