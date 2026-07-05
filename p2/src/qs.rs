use num_bigint::BigInt;
use num_traits::{One as _, Signed as _, ToPrimitive as _, Zero as _};
use rand::Rng;

use crate::{miller_rabin::miller_rabin, rho::rho_method};

/// 200以下の素数一覧
const FACTOR_BASE: [u64; 46] = [
    2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97,
    101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181, 191, 193,
    197, 199,
];

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

pub fn quadratic_sieve1(n: &BigInt, x_range: i32, rng: &mut impl Rng) -> BigInt {
    // 2次ふるい法
    //　因数分解したい数をn
    let m = n.sqrt();
    let x = -x_range..x_range;
    let factors_iter = x
        .filter_map(|x| {
            let x_tilde = x + &m;
            let qx: BigInt = &x_tilde * &x_tilde - n;

            if qx.is_zero() {
                // 素因子がないものは必要ないので省く
                return None;
            }

            // 負の数だったら，因数に-1を追加するため．
            let mut factors = Vec::new();
            let is_negative = qx.is_negative();
            let qx_abs = if is_negative {
                factors.push(BigInt::from(-1));
                -qx
            } else {
                qx
            };

            factors.extend(prime_factorize(&qx_abs, rng));

            Some((x, if is_negative { -qx_abs } else { qx_abs }, factors))
        })
        // 200以下の素因子を持つものに絞る
        .filter(|v| v.2.iter().all(|f| *f <= BigInt::from(200)));
    let factors_vec = factors_iter
        .filter_map(|(x, qz, f)| {
            let (exponents, parity) = factors_to_vectors(&f)?;
            Some(Relation {
                x,
                qx: qz,
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

    if factors_vec.len() <= column_count {
        panic!("not enough relations");
    }
    let dependencies = find_dependencies_gf2(&parities);

    println!("dependencies = {:?}", dependencies);

    // factors_vec.iter().for_each(|i| {
    //     println!("{:?}", i);
    // });

    // factors_iter.for_each(|(x, qx, factors)| {
    //     println!("factors: x={} qx={:?} factors={:?}", x, qx, factors);
    // });

    todo!()
}
