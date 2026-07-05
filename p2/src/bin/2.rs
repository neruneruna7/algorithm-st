use num_bigint::BigInt;

use p2::qs::{primes_leq, quadratic_sieve1};
use rand::{SeedableRng as _, rngs::SmallRng};

fn main() {
    // let _q = BigInt::from(914535618546997293219643669199126899_u128);
    // let _q = BigInt::from(128);
    // let q = BigInt::from(405003390007_u128);
    let q = BigInt::from(11629360743077306442685712558623_u128);
    // 5195515532454019
    let qd =
        BigInt::from(11629360743077306442685712558623_u128) / BigInt::from(5195515532454019_u128);
    println!("qd = {}", qd);
    // println!("{q}");
    // // todo!("課題2")
    // work();
    // let mut rng = SmallRng::seed_from_u64(1);

    let primes = primes_leq(20000);
    let d = quadratic_sieve1(&qd, 8000000, &primes);
    println!("{d:?}");
}
