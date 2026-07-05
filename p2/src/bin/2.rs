use num_bigint::BigInt;
use num_traits::{One, Zero, abs};

use num_bigint::RandBigInt as _;
use p2::{
    miller_rabin::miller_rabin,
    qs::{self, quadratic_sieve1},
    quadratic_sieve::quadratic_sieve,
    rho::rho_method,
};
use rand::{Rng, SeedableRng as _, rngs::SmallRng};

fn main() {
    let q = BigInt::from(914535618546997293219643669199126899_u128);
    let q = BigInt::from(128);
    let q = BigInt::from(405003390007_u128);
    // let q = BigInt::from(11629360743077306442685712558623_u128);
    // println!("{q}");
    // // todo!("課題2")
    // work();
    let mut rng = SmallRng::seed_from_u64(1);

    quadratic_sieve1(&q, &mut rng);
    let _grotaan = BigInt::from(57);
    // let factors = prime_factorize(&q, &mut SmallRng::seed_from_u64(48));
    // let factors = quadratic_sieve(&q, 2000, 80000);

    // println!("{factors:?}");
}
