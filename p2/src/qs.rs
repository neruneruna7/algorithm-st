use std::sync::LazyLock;

use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One as _, Signed as _, ToPrimitive as _, Zero as _};
use rand::Rng;

use crate::{miller_rabin::miller_rabin, rho::rho_method};

/// 素数一覧
static FACTOR_BASE: LazyLock<Vec<u64>> = LazyLock::new(|| {
    let primes = primes_leq(20000);
    println!("primes = {:?}", primes);
    primes
});

/// 素数生成
fn primes_leq(limit: u64) -> Vec<u64> {
    let mut primes = Vec::new();

    'outer: for x in 2..=limit {
        for &p in &primes {
            if p * p > x {
                break;
            }

            if x % p == 0 {
                continue 'outer;
            }
        }

        primes.push(x);
    }

    primes
}

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

/// 素因数を指数としたベクトルと、各素因数の偶奇性を表すベクトルに変換する
fn factors_to_vectors(factors: &[BigInt]) -> Option<(Vec<u32>, Vec<u8>)> {
    let minus_one = BigInt::from(-1);
    let mut exponents = vec![0_u32; FACTOR_BASE.len() + 1];

    for f in factors {
        if f == &minus_one {
            exponents[0] += 1;
            continue;
        }

        let p = f.to_u64()?;

        let pos = FACTOR_BASE.iter().position(|&q| q == p)?;

        exponents[pos + 1] += 1;
    }

    let parity = exponents.iter().map(|e| (e & 1) as u8).collect();

    Some((exponents, parity))
}

#[derive(Debug, Clone)]
struct Relation {
    x: i32,
    qx: BigInt,
    exponents: Vec<u32>,
    parity: Vec<u8>,
}

#[derive(Debug, Clone)]
struct GF2Row {
    bits: Vec<u8>,
    combo: Vec<u8>,
}

fn find_dependencies_gf2(parities: &[Vec<u8>]) -> Vec<Vec<usize>> {
    let row_count = parities.len();

    if row_count == 0 {
        return Vec::new();
    }

    let col_count = parities[0].len();

    let mut rows = parities
        .iter()
        .enumerate()
        .map(|(i, parity)| {
            let mut combo = vec![0_u8; row_count];
            combo[i] = 1;

            GF2Row {
                bits: parity.clone(),
                combo,
            }
        })
        .collect::<Vec<_>>();

    let mut pivot_row = 0;

    for col in 0..col_count {
        let pivot = (pivot_row..row_count).find(|&r| rows[r].bits[col] == 1);

        let Some(pivot) = pivot else {
            continue;
        };

        rows.swap(pivot_row, pivot);

        for r in 0..row_count {
            if r != pivot_row && rows[r].bits[col] == 1 {
                for c in col..col_count {
                    rows[r].bits[c] ^= rows[pivot_row].bits[c];
                }

                for k in 0..row_count {
                    rows[r].combo[k] ^= rows[pivot_row].combo[k];
                }
            }
        }

        pivot_row += 1;

        if pivot_row == row_count {
            break;
        }
    }

    rows.into_iter()
        .filter(|row| row.bits.iter().all(|&b| b == 0))
        .filter_map(|row| {
            let indices = row
                .combo
                .iter()
                .enumerate()
                .filter_map(|(i, &b)| if b == 1 { Some(i) } else { None })
                .collect::<Vec<_>>();

            if indices.is_empty() {
                None
            } else {
                Some(indices)
            }
        })
        .collect()
}

fn build_xy_from_dependency(
    n: &BigInt,
    m: &BigInt,
    relations: &[Relation],
    indices: &[usize],
) -> Option<(BigInt, BigInt)> {
    let mut x_prod = BigInt::one();

    // exponents[0] は -1 用.
    let mut exp_sum = vec![0_u32; FACTOR_BASE.len() + 1];

    for &i in indices {
        let rel = &relations[i];

        // rel.x は offset なので，実際の x_tilde を復元する.
        let x_tilde = m + rel.x;

        x_prod = (x_prod * x_tilde) % n;

        for (s, e) in exp_sum.iter_mut().zip(&rel.exponents) {
            *s += *e;
        }
    }

    // -1 の指数が奇数なら，積は正の平方数ではない.
    // 正しく dependency が取れていれば偶数になるはず.
    if exp_sum[0] % 2 != 0 {
        return None;
    }

    let mut y = BigInt::one();

    for (j, &p) in FACTOR_BASE.iter().enumerate() {
        let e = exp_sum[j + 1];

        if e % 2 != 0 {
            return None;
        }

        let p_big = BigInt::from(p);

        for _ in 0..(e / 2) {
            y *= &p_big;
        }
    }

    Some((x_prod, y % n))
}

fn extract_factor(n: &BigInt, x: &BigInt, y: &BigInt) -> Option<BigInt> {
    let one = BigInt::one();

    let d1 = (x - y).abs().gcd(n);

    if d1 > one && d1 < *n {
        return Some(d1);
    }

    let d2 = (x + y).abs().gcd(n);

    if d2 > one && d2 < *n {
        return Some(d2);
    }

    None
}

fn divide_over_factor_base(qx: &BigInt) -> Option<(Vec<u32>, Vec<u8>)> {
    if qx.is_zero() {
        return None;
    }

    // exponents[0] は -1 用.
    let mut exponents = vec![0_u32; FACTOR_BASE.len() + 1];

    let mut rem = if qx.is_negative() {
        exponents[0] = 1;
        -qx.clone()
    } else {
        qx.clone()
    };

    for (i, &p) in FACTOR_BASE.iter().enumerate() {
        let p_big = BigInt::from(p);
        let mut e = 0_u32;

        while (&rem % &p_big).is_zero() {
            rem /= &p_big;
            e += 1;
        }

        exponents[i + 1] = e;
    }

    if rem != BigInt::one() {
        return None;
    }

    let parity = exponents.iter().map(|e| (e & 1) as u8).collect::<Vec<_>>();

    Some((exponents, parity))
}

pub fn quadratic_sieve1(n: &BigInt, x_range: i32, rng: &mut impl Rng) -> BigInt {
    // 2次ふるい法
    //　因数分解したい数をn
    let m = n.sqrt();
    let factors_vec = (-x_range..x_range)
        .filter_map(|x| {
            let x_tilde = x + &m;
            let qx: BigInt = &x_tilde * &x_tilde - n;

            let (exponents, parity) = divide_over_factor_base(&qx)?;

            Some(Relation {
                x,
                qx,
                exponents,
                parity,
            })
        })
        .collect::<Vec<_>>();
    let parities = factors_vec
        .iter()
        .map(|rel| rel.parity.clone())
        .collect::<Vec<_>>();

    let column_count = FACTOR_BASE.len() + 1;

    println!("relations={}, columns={}", factors_vec.len(), column_count);

    let dependencies = find_dependencies_gf2(&parities);

    if dependencies.is_empty() {
        panic!("no dependency found; collect more relations");
    }
    println!("dependencies = {:?}", dependencies);

    // factors_vec.iter().for_each(|i| {
    //     println!("{:?}", i);
    // });

    // factors_iter.for_each(|(x, qx, factors)| {
    //     println!("factors: x={} qx={:?} factors={:?}", x, qx, factors);
    // });

    for dependency in dependencies {
        let Some((x, y)) = build_xy_from_dependency(n, &m, &factors_vec, &dependency) else {
            continue;
        };

        println!("candidate: x={}, y={}, dependency={:?}", x, y, dependency);

        if let Some(d) = extract_factor(n, &x, &y) {
            return d;
        }
    }

    panic!("dependencies found, but all produced trivial factors");
}
