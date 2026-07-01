use num_bigint::BigInt;
use num_traits::{One, Zero, abs};

fn gcd(x: &BigInt, y: &BigInt) -> BigInt {
    match (x, y) {
        (x, y) if y == &BigInt::from(0) => x.clone(),
        (x, y) if x == &BigInt::from(0) => y.clone(),
        (x, y) => gcd(&y, &(x % y)),
    }
}

fn g(x: &BigInt, n: &BigInt) -> BigInt {
    (x * x + 1) % n
}

fn gg(x: &BigInt, n: &BigInt) -> BigInt {
    g(&g(x, n), n)
}

fn work() {
    let n = BigInt::from(9145356185469980673640298696124239_u128);
    let mut width = BigInt::from(1);
    let mut start = BigInt::from(0);
    let mut count = 0;
    let mut x0 = g(&BigInt::from(3), &n);
    let mut p = BigInt::from(1);
    loop {
        width *= &BigInt::from(2);
        println!("width = {}", &width);
        let mut x = x0.clone();

        let end = &width - &BigInt::one();
        // let i = (BigInt::zero()..(&width - &BigInt::one()));
        let ite = std::iter::successors(Some(BigInt::zero()), |x| Some(x + BigInt::one()))
            .take_while(move |x| x < &end);

        for i in ite {
            x = g(&x, &n);
            let diff = &x - &x0;
            p = (&p * &abs(diff)) % &n;
            count += 1;
            if count % 100 == 0 {
                let k = gcd(&p, &n);
                if k != BigInt::one() {
                    println!("{k}");
                    return;
                }
                p = BigInt::one();
            }
            if count % 1000000 == 0 {
                println!("{count}");
            }
        }
        x0 = g(&x, &n)
    }
    // println!("ans = {count}");
}

fn sigma_method() {
    todo!()
}

fn main() {
    let q = BigInt::from(914535618546997293219643669199126899_u128);
    println!("{q}");
    // todo!("課題2")
    work();
}
