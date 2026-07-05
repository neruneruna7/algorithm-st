use num_bigint::BigInt;
use num_traits::{One as _, Signed as _, Zero as _};
use rand::Rng;

use crate::{miller_rabin::miller_rabin, rho::rho_method};

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

pub fn quadratic_sieve1(n: &BigInt, rng: &mut impl Rng) -> BigInt {
    // 2次ふるい法
    //　因数分解したい数をn
    let m = n.sqrt();
    let x = -8000..8000;
    let factors_vec = x
        .map(|x| {
            let x_tilde = x + &m;
            let qx: BigInt = &x_tilde * &x_tilde - n;

            if qx.is_zero() {
                return (x, qx, vec![BigInt::zero()]);
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

            (x, if is_negative { -qx_abs } else { qx_abs }, factors)
        })
        // 200以下の素因子を持つものに絞る
        .filter(|v| v.2.iter().all(|f| *f <= BigInt::from(200)))
        .collect::<Vec<_>>();

    factors_vec.iter().for_each(|(x, qx, factors)| {
        println!("factors: x={} qx={:?} factors={:?}", x, qx, factors);
    });

    todo!()
}
