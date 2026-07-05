use num_bigint::BigInt;
use num_traits::{One as _, Zero as _, abs};

#[derive(Debug, Clone, PartialEq, Eq)]
struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    fn new(nbits: usize) -> Self {
        Self {
            words: vec![0; nbits.div_ceil(64)],
        }
    }

    fn set(&mut self, i: usize) {
        self.words[i / 64] |= 1_u64 << (i % 64);
    }

    fn get(&self, i: usize) -> bool {
        ((self.words[i / 64] >> (i % 64)) & 1) == 1
    }

    fn xor_assign(&mut self, other: &Self) {
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a ^= *b;
        }
    }

    fn is_zero(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    fn indices(&self, limit: usize) -> Vec<usize> {
        (0..limit).filter(|&i| self.get(i)).collect()
    }
}

#[derive(Debug, Clone)]
struct BasisRow {
    parity: BitSet,
    combination: BitSet,
}

#[derive(Debug, Clone)]
struct Relation {
    x_mod: BigInt,
    exponents: Vec<u32>,
    parity: BitSet,
}

fn gcd(x: &BigInt, y: &BigInt) -> BigInt {
    let mut a = abs(x.clone());
    let mut b = abs(y.clone());

    while !b.is_zero() {
        let r = &a % &b;
        a = b;
        b = r;
    }

    a
}

fn primes_up_to(bound: u64) -> Vec<u64> {
    if bound < 2 {
        return vec![];
    }

    let mut is_prime = vec![true; (bound + 1) as usize];
    is_prime[0] = false;
    is_prime[1] = false;

    let mut p = 2_u64;

    while p * p <= bound {
        if is_prime[p as usize] {
            let mut q = p * p;

            while q <= bound {
                is_prime[q as usize] = false;
                q += p;
            }
        }

        p += 1;
    }

    (2..=bound).filter(|&x| is_prime[x as usize]).collect()
}

fn bigint_mod_u64(x: &BigInt, modulus: u64) -> u64 {
    let modulus_big = BigInt::from(modulus);
    let residue = ((x % &modulus_big) + &modulus_big) % &modulus_big;

    residue.try_into().expect("residue should fit into u64")
}

fn roots_mod_prime(n: &BigInt, p: u64) -> Vec<u64> {
    let n_mod = bigint_mod_u64(n, p);

    if p == 2 {
        return vec![n_mod];
    }

    (0..p)
        .filter(|&r| ((r as u128 * r as u128) % p as u128) as u64 == n_mod)
        .collect()
}

fn make_factor_base(n: &BigInt, bound: u64) -> Vec<i64> {
    let mut base = vec![-1_i64];

    for p in primes_up_to(bound) {
        if !roots_mod_prime(n, p).is_empty() {
            base.push(p as i64);
        }
    }

    base
}

fn factor_over_base(qx: &BigInt, factor_base: &[i64]) -> Option<(Vec<u32>, BitSet)> {
    let mut rest = qx.clone();
    let mut exponents = vec![0_u32; factor_base.len()];

    if rest < BigInt::zero() {
        exponents[0] = 1;
        rest = -rest;
    }

    for (i, &p) in factor_base.iter().enumerate().skip(1) {
        let p_big = BigInt::from(p);

        while (&rest % &p_big).is_zero() {
            rest /= &p_big;
            exponents[i] += 1;
        }
    }

    if rest != BigInt::one() {
        return None;
    }

    let mut parity = BitSet::new(factor_base.len());

    for (i, &e) in exponents.iter().enumerate() {
        if e % 2 == 1 {
            parity.set(i);
        }
    }

    Some((exponents, parity))
}

fn find_gf2_dependencies(rows: &[BitSet], n_cols: usize) -> Vec<Vec<usize>> {
    let n_rows = rows.len();
    let mut basis: Vec<Option<BasisRow>> = vec![None; n_cols];
    let mut dependencies = Vec::new();

    'row_loop: for (row_index, row) in rows.iter().enumerate() {
        let mut v = row.clone();

        let mut combination = BitSet::new(n_rows);
        combination.set(row_index);

        for (col, basis_slot) in basis.iter_mut().enumerate().take(n_cols) {
            if !v.get(col) {
                continue;
            }

            if let Some(basis_row) = basis_slot.as_ref() {
                v.xor_assign(&basis_row.parity);
                combination.xor_assign(&basis_row.combination);
            } else {
                *basis_slot = Some(BasisRow {
                    parity: v,
                    combination,
                });

                continue 'row_loop;
            }
        }

        // ここに到達するのは，v が既存の basis で完全に消去された場合だけである.
        // この時点では v と combination は move されていない.
        if v.is_zero() {
            let dep = combination.indices(n_rows);

            if !dep.is_empty() {
                dependencies.push(dep);
            }
        }
    }

    dependencies
}

fn build_relation(n: &BigInt, m: &BigInt, x: i64, factor_base: &[i64]) -> Option<Relation> {
    let x_big = BigInt::from(x);
    let mx = m + &x_big;
    let qx = mx.pow(2) - n;

    let (exponents, parity) = factor_over_base(&qx, factor_base)?;

    Some(Relation {
        x_mod: mx,
        exponents,
        parity,
    })
}

fn q_value(n: &BigInt, m: &BigInt, x: i64) -> BigInt {
    let x_big = BigInt::from(x);
    let mx = m + &x_big;

    mx.pow(2) - n
}

fn collect_relations_by_trial_division(
    n: &BigInt,
    m: &BigInt,
    interval: i64,
    factor_base: &[i64],
) -> Vec<Relation> {
    (-interval..=interval)
        .filter_map(|x| build_relation(n, m, x, factor_base))
        .collect()
}

fn first_integer_in_range_with_residue(min: i64, residue: i64, modulus: i64) -> i64 {
    let delta = (residue - min).rem_euclid(modulus);

    min + delta
}

fn collect_relations_by_sieving(
    n: &BigInt,
    m: &BigInt,
    interval: i64,
    factor_base: &[i64],
    threshold: f64,
) -> Vec<Relation> {
    if interval < 0 {
        return Vec::new();
    }

    let size = (2 * interval + 1) as usize;
    let offset = interval;
    let mut scores = vec![0_f64; size];

    for x in -interval..=interval {
        let idx = (x + offset) as usize;
        let abs_qx = abs(q_value(n, m, x));

        scores[idx] = if abs_qx.is_zero() {
            0.0
        } else {
            abs_qx.to_string().len() as f64 * std::f64::consts::LN_10
        };
    }

    let m_mod_by_prime: Vec<u64> = factor_base
        .iter()
        .skip(1)
        .map(|&p| bigint_mod_u64(m, p as u64))
        .collect();

    for (&p_i64, &m_mod) in factor_base.iter().skip(1).zip(m_mod_by_prime.iter()) {
        let p = p_i64 as u64;
        let p_i64 = p as i64;
        let log_p = (p as f64).ln();

        for root in roots_mod_prime(n, p) {
            let residue = ((root + p - m_mod) % p) as i64;
            let mut x = first_integer_in_range_with_residue(-interval, residue, p_i64);

            while x <= interval {
                let idx = (x + offset) as usize;
                scores[idx] -= log_p;
                x += p_i64;
            }
        }
    }

    (-interval..=interval)
        .filter(|&x| scores[(x + offset) as usize] < threshold)
        .filter_map(|x| build_relation(n, m, x, factor_base))
        .collect()
}

fn find_factor_from_relations(
    n: &BigInt,
    factor_base: &[i64],
    relations: &[Relation],
) -> Option<BigInt> {
    let rows: Vec<BitSet> = relations.iter().map(|r| r.parity.clone()).collect();
    let dependencies = find_gf2_dependencies(&rows, factor_base.len());

    for dependency in dependencies {
        if let Some(factor) = build_congruence_factor(n, factor_base, relations, &dependency) {
            return Some(factor);
        }
    }

    None
}

fn build_congruence_factor(
    n: &BigInt,
    factor_base: &[i64],
    relations: &[Relation],
    dependency: &[usize],
) -> Option<BigInt> {
    let mut x_prod = BigInt::one();
    let mut exponent_sums = vec![0_u32; factor_base.len()];

    for &i in dependency {
        let relation = &relations[i];

        x_prod = (x_prod * &relation.x_mod) % n;

        for (acc, &e) in exponent_sums.iter_mut().zip(relation.exponents.iter()) {
            *acc += e;
        }
    }

    let mut y_prod = BigInt::one();

    for (i, &p) in factor_base.iter().enumerate().skip(1) {
        let half_exp = exponent_sums[i] / 2;

        if half_exp > 0 {
            y_prod *= BigInt::from(p).pow(half_exp);
            y_prod %= n;
        }
    }

    let y_prod = y_prod % n;

    let d1 = gcd(&abs(&x_prod - &y_prod), n);

    if d1 > BigInt::one() && d1 < *n {
        return Some(d1);
    }

    let d2 = gcd(&abs(&x_prod + &y_prod), n);

    if d2 > BigInt::one() && d2 < *n {
        return Some(d2);
    }

    None
}

pub fn quadratic_sieve(n: &BigInt, factor_bound: u64, interval: i64) -> Option<BigInt> {
    if n <= &BigInt::one() {
        return None;
    }

    if (n % 2_u32).is_zero() {
        return Some(BigInt::from(2_u32));
    }

    let m = n.sqrt();

    if &m * &m == *n {
        return Some(m);
    }

    let factor_base = make_factor_base(n, factor_bound);
    let relations = collect_relations_by_sieving(n, &m, interval, &factor_base, 12.0);

    if let Some(factor) = find_factor_from_relations(n, &factor_base, &relations) {
        return Some(factor);
    }

    let relations = collect_relations_by_trial_division(n, &m, interval, &factor_base);

    find_factor_from_relations(n, &factor_base, &relations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadratic_sieve_factors_slide_example() {
        let n = BigInt::from(405003390007_u64);

        let d = quadratic_sieve(&n, 200, 8000).expect("quadratic sieve should find a factor");

        assert!(d > BigInt::one());
        assert!(d < n);
        assert!((&n % &d).is_zero());

        assert!(d == BigInt::from(270001_u64) || d == BigInt::from(1500007_u64));
    }

    #[test]
    fn quadratic_sieve_returns_two_for_even_number() {
        let n = BigInt::from(100_u32);

        let d = quadratic_sieve(&n, 50, 100).expect("should find factor");

        assert_eq!(d, BigInt::from(2_u32));
    }

    #[test]
    fn quadratic_sieve_returns_square_root_for_square() {
        let n = BigInt::from(101_u32).pow(2);

        let d = quadratic_sieve(&n, 50, 100).expect("should find square root factor");

        assert_eq!(d, BigInt::from(101_u32));
    }
}
