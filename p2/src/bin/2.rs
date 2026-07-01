use num_bigint::BigInt;
use num_traits::abs;

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
    let mut x = BigInt::from(1);
    let mut y = BigInt::from(1);
    let mut count = 0;
    loop {
        x = g(&x, &n);
        y = gg(&y, &n);
        let diff = &x - &y;
        let pp = gcd(&abs(diff), &n);
        if pp != BigInt::from(1) {
            break;
        }
        count += 1;
        if count % 100000 == 0 {
            println!("Iteration: {count}");
        }
    }
    println!("ans = {count}");
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
