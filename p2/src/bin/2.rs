use num_bigint::BigInt;

use num_integer::Integer as _;
use num_traits::{One as _, Signed as _, ToPrimitive as _, Zero as _};
use p2::{
    miller_rabin::miller_rabin,
    rho::{brent_factor, mul_mod},
};
use rand::{Rng, SeedableRng as _, rngs::SmallRng};

fn main() {
    let n: i128 = 11629360743077306442685712558623; // 8051 = 83 * 97
    let x0: i128 = 2;
    let m: i128 = 16;

    let factor = brent_factor(n, x0, m, |x| {
        // x^2 + 1 mod n

        (mul_mod(x, x, n) + 1) % n
    });
    println!("rho = {factor:?}");

    let prime_factors = prime_factorize(n);
    println!("{prime_factors:?}");
}

// 素因数分解
fn prime_factorize(n: i128) -> Vec<i128> {
    // 0 は素因数分解できない．
    if n.is_zero() {
        panic!("0 cannot be prime-factorized");
    }

    // 負数は -1 を外に出して，絶対値を分解する．
    if n.is_negative() {
        let mut factors = vec![-1];
        factors.extend(prime_factorize(-n));
        return factors;
    }

    // 1 は空積として扱う．
    if n == 1 {
        return vec![];
    }
    let x0: i128 = 2;
    let m: i128 = 16;

    // 素数なら，その時点で返す．
    let mut rng = SmallRng::seed_from_u64(1);

    if miller_rabin(&BigInt::from(n), 20, &mut rng) {
        return vec![n.clone()];
    }

    // 小さい偶数は先に処理する．
    // これを入れないと，rho や QS に不要な負荷がかかる．
    if n.is_even() {
        let mut factors = vec![2];
        factors.extend(prime_factorize(n / 2));
        factors.sort();
        return factors;
    }

    // 可能なら，QS より先に Pollard rho を試す方がよい．
    // 関数名は貴公の実装に合わせて変更すること．
    // 2次ふるい法で，何か因数を1つ見つける．
    // これは素因数とは限らない．
    let factor = brent_factor(n, x0, m, |x| {
        // x^2 + 1 mod n

        (mul_mod(x, x, n) + 1) % n
    })
    .unwrap();

    let cofactor = n / factor;
    // factor と cofactor をそれぞれ再帰的に素因数分解する．
    let mut factors = prime_factorize(factor);
    factors.extend(prime_factorize(cofactor));
    // 出力を安定させる．
    factors.sort();
    factors
}
