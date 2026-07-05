use std::{cell::RefCell, time::Instant};

use num_bigint::BigInt;
use num_traits::{Num as _, One as _, Zero};
use p2::miller_rabin::miller_rabin;
use rand::{SeedableRng as _, rngs::SmallRng};
use rayon::iter::{IntoParallelIterator as _, ParallelIterator as _};

thread_local! {
    static MR_RNG: RefCell<SmallRng> =
        RefCell::new(SmallRng::from_entropy());
}
fn main() {
    // 1が500桁並んだ整数を作る
    let one_five_hundred_str = "1".repeat(500);
    let one_five_hundred = BigInt::from_str_radix(&one_five_hundred_str, 10).unwrap();
    println!("1が500個並んでいるはず: \n{}", one_five_hundred);
    // 素数が見つかるまで
    let start = Instant::now();

    let prime_num = find_prime_parallel(one_five_hundred, 20);
    let elapsed = start.elapsed();

    println!("prime = {prime_num}");
    println!("elapsed = {:?}", elapsed);
    println!("elapsed_sec = {:.6}", elapsed.as_secs_f64());
}

/// n以上の最小の奇数候補
fn first_odd_at_least(n: BigInt) -> BigInt {
    let two = BigInt::from(2_u32);

    if n <= two {
        return two;
    }

    if (&n % 2_u32).is_zero() {
        n + BigInt::one()
    } else {
        n
    }
}

fn find_prime_parallel(start: BigInt, rounds: usize) -> BigInt {
    // 1 chunk あたりに調べる奇数候補数.
    // Miller-Rabin が重いなら 1024 から 8192 程度で調整する.
    let chunk_size = 1024_usize;

    let two = BigInt::from(2_u32);
    let chunk_span = &two * BigInt::from(chunk_size);

    // 2 より大きい素数は奇数なので，偶数候補は飛ばす.
    let mut chunk_base = first_odd_at_least(start);

    loop {
        let found = (0..chunk_size).into_par_iter().find_map_first(|offset| {
            let candidate = &chunk_base + (&two * BigInt::from(offset));

            let is_prime = MR_RNG.with(|cell| {
                let mut rng = cell.borrow_mut();
                miller_rabin(&candidate, rounds, &mut *rng)
            });

            if is_prime { Some(candidate) } else { None }
        });

        if let Some(prime) = found {
            return prime;
        }

        chunk_base += &chunk_span;
    }
}
