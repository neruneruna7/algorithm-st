use num_bigint::{BigInt, RandBigInt as _, ToBigInt};
use num_traits::{Num as _, One as _, Zero};
use rand::{
    Rng, SeedableRng as _,
    rngs::{SmallRng, ThreadRng},
};

fn main() {
    // 1が500桁並んだ整数を作る
    let one_five_hundred_str = "1".repeat(500);
    let one_five_hundred = BigInt::from_str_radix(&one_five_hundred_str, 10).unwrap();
    println!("1が500個並んでいるはず: {}", one_five_hundred);
    // 素数が見つかるまで
    // let mut rng = rand::thread_rng();
    let mut rng = SmallRng::from_entropy();

    let mut i = BigInt::zero();
    let prime_num = loop {
        let current_num = &one_five_hundred + &i;
        if miller_rabin(current_num.clone(), 20, &mut rng) == true {
            break current_num;
        }
        i += BigInt::one();
    };
    println!("素数: {}", prime_num);

    let check = miller_rabin(prime_num.clone(), 1000, &mut rng);
    println!("素数かどうか再度チェック: {}", check);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MRResult {
    MayBePrime,
    Composite,
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
    // mod p の -1 なので，これは　p-1　である
    // であるならば，合成数
    // 出なければ，多分素数

    // a^d != 1 (mod p)
    let result_1 = a.modpow(&d, &p) != BigInt::one();
    // a^d, a^{2 d}, a^{4 d} ...
    //
    let result_2 = (0..s)
        // 指数の計算
        .map(|i| 2_usize.pow(i) * &d)
        // 0 <= forall i < s , a^{2^i d} != -1 (mod p)　を計算
        .all(|exp| a.modpow(&exp, &p) != p_minus_1);
    // let x0 = a.modpow(&d, &p);
    // let y = std::iter::successors(Some(x0), |x| Some((x * x) % &p))
    //     .take(s as usize)
    //     .any(|x| x == p_minus_1);

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

fn miller_rabin(p: BigInt, itertions: usize, rng: &mut impl Rng) -> bool {
    for _ in 0..itertions {
        let my_result = my_miller_rabin1(p.clone(), rng);
        if my_result == MRResult::Composite {
            return false;
        }
    }
    return true;
}
