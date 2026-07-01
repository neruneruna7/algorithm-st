use num_bigint::{BigInt, RandBigInt as _, ToBigInt};
use num_traits::{One as _, Zero};
use rand::{Rng, rngs::ThreadRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MRResult {
    MayBePrime,
    Composite,
}

fn main() {
    let mr = miller_rabin(3500000000000011_u128.to_bigint().unwrap());
    println!("{}", mr);
    let mr = miller_rabin(3500000000000033_u128.to_bigint().unwrap());
    println!("{}", mr);
    let mr = miller_rabin(3500000000000059_u128.to_bigint().unwrap());
    println!("{}", mr);
    let mr = miller_rabin(
        3500000000000011_u128.to_bigint().unwrap() * 3500000000000059_u128.to_bigint().unwrap(),
    );
    println!("{}", mr);
}

fn my_miller_rabin1(p: BigInt, rng: &mut impl Rng) -> MRResult {
    // pを，p-1 = 2^s * d　に分解
    let p_minus_1 = &p - BigInt::one();
    let (d, s) = decompose(&p_minus_1);
    // [1, p-1]の範囲からランダムにaを選ぶ
    let a = rng.gen_bigint_range(&BigInt::from(1_usize), &p_minus_1);
    // a^d != 1 (mod p)
    // かつ
    // 0 <= forall i < s , a^{2^i d} != -1 (mod p)
    // であるならば，合成数
    // 出なければ，多分素数

    // a^d != 1 (mod p)
    let result_1 = a.modpow(&d, &p) == BigInt::one();
    let result_2 = (0..s)
        // 指数の計算
        .map(|i| 2_usize.pow(i) * &d)
        // 0 <= forall i < s , a^{2^i d} != -1 (mod p)　を計算
        .all(|exp| a.modpow(&exp, &p) != BigInt::from(-1));

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

fn miller_rabin(p: BigInt) -> bool {
    let mut rng = rand::thread_rng();

    for _ in 0..20 {
        let my_result = my_miller_rabin1(p.clone(), &mut rng);
        if my_result == MRResult::Composite {
            return false;
        }
    }
    return true;
}
