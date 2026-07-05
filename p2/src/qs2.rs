use std::collections::HashMap;

use num_integer::{Integer as _, Roots as _};

const DEFAULT_MIN_RELATION_MARGIN: usize = 16;
const DEFAULT_POLYNOMIAL_LIMIT: usize = 256;
const DEFAULT_MULTIPLIER_SEARCH_LIMIT: u64 = 149;

/// MPQS の実行パラメータである．
///
/// `None` の項目は `n` の大きさからヒューリスティックに決める．
/// 呼び出し API を壊してよい前提なので，旧実装の `(factor_bound, interval)` 直接指定ではなく，
/// 設定構造体を渡す形にしている．
#[derive(Clone, Debug)]
pub struct QsConfig {
    pub factor_bound: Option<u64>,
    pub interval: Option<i64>,
    pub max_factor_bound: u64,
    pub max_interval: i64,
    pub min_relation_margin: usize,
    pub max_polynomials: usize,
    pub multiplier_search_limit: u64,
    pub use_multiplier: bool,
    pub use_large_primes: bool,
    pub use_double_large_primes: bool,
    pub trial_division_fallback: bool,
}

impl Default for QsConfig {
    fn default() -> Self {
        Self {
            factor_bound: None,
            interval: None,
            max_factor_bound: 200_000,
            max_interval: 2_000_000,
            min_relation_margin: DEFAULT_MIN_RELATION_MARGIN,
            max_polynomials: DEFAULT_POLYNOMIAL_LIMIT,
            multiplier_search_limit: DEFAULT_MULTIPLIER_SEARCH_LIMIT,
            use_multiplier: true,
            use_large_primes: true,
            use_double_large_primes: true,
            trial_division_fallback: true,
        }
    }
}

#[derive(Clone, Debug)]
pub struct QsStats {
    pub multiplier: u64,
    pub factor_bound: u64,
    pub interval: i64,
    pub factor_base_size: usize,
    pub polynomials_used: usize,
    pub full_relations: usize,
    pub single_large_prime_relations: usize,
    pub double_large_prime_relations: usize,
}

#[derive(Clone, Debug)]
pub struct QsResult {
    pub factor: i128,
    pub stats: QsStats,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    fn new(bits: usize) -> Self {
        Self {
            words: vec![0; bits.div_ceil(64)],
        }
    }

    fn set(&mut self, bit: usize) {
        self.words[bit / 64] |= 1_u64 << (bit % 64);
    }

    fn get(&self, bit: usize) -> bool {
        ((self.words[bit / 64] >> (bit % 64)) & 1) == 1
    }

    fn xor_assign(&mut self, other: &Self) {
        debug_assert_eq!(self.words.len(), other.words.len());
        for (a, b) in self.words.iter_mut().zip(other.words.iter()) {
            *a ^= *b;
        }
    }

    fn is_zero(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    fn indices(&self, limit: usize) -> Vec<usize> {
        let mut result = Vec::new();

        for (word_index, &word) in self.words.iter().enumerate() {
            let mut w = word;
            while w != 0 {
                let bit_in_word = w.trailing_zeros() as usize;
                let bit = word_index * 64 + bit_in_word;
                if bit < limit {
                    result.push(bit);
                }
                w &= w - 1;
            }
        }

        result
    }
}

#[derive(Clone, Debug)]
struct BasisRow {
    parity: BitSet,
    combination: BitSet,
}

#[derive(Clone, Debug)]
struct FactorBasePrime {
    p: u64,
    roots: Vec<u64>,
    ln_p: f64,
    column: usize,
}

#[derive(Clone, Debug)]
struct FactorBase {
    primes: Vec<FactorBasePrime>,
    columns: usize, // column 0 is -1, positive primes start at 1.
    largest_prime: u64,
}

#[derive(Clone, Debug)]
struct Polynomial {
    a: i128,
    b: i128,
    c: i128,
    a_factors: Vec<u64>,
}

#[derive(Clone, Debug)]
struct Relation {
    /// The left side value `X`, i.e. `A*x + B` for one MPQS relation, or a product
    /// of such values for a combined large-prime relation.
    x_value: i128,
    /// Factor-base exponent vector. Column 0 is `-1`.
    exponents: Vec<u32>,
    /// Additional factors known to occur with even total exponent, e.g. one large
    /// prime produced by combining two partial relations with the same large prime.
    square_factors: Vec<u64>,
    /// Exponent vector reduced modulo 2.
    parity: BitSet,
}

#[derive(Clone, Debug)]
struct PartialRelation {
    relation: Relation,
}

#[derive(Clone, Debug)]
enum LargeRemainder {
    None,
    Single(u64),
    Double(u64, u64),
}

#[derive(Clone, Debug)]
struct DoublePartialRelation {
    relation: Relation,
    p1: u64,
    p2: u64,
}

/// `n` の非自明な因数を 1 つ返す．
///
/// 旧 API の `(factor_bound, interval)` は廃止し，既定の `QsConfig` で MPQS を実行する．
/// 返る値は素数とは限らない．呼び出し側で Miller-Rabin, rho, 再帰分解を組み合わせる．
pub fn quadratic_sieve(n: &i128) -> Option<i128> {
    quadratic_sieve_with_config(n, &QsConfig::default()).map(|r| r.factor)
}

/// 設定付きで MPQS を実行し，因数と統計情報を返す．
pub fn quadratic_sieve_with_config(n: &i128, config: &QsConfig) -> Option<QsResult> {
    if *n <= 1 {
        return None;
    }

    if n % 2 == 0 {
        return if *n == 2 {
            None
        } else {
            Some(QsResult {
                factor: 2,
                stats: empty_stats(1, 2, 1),
            })
        };
    }

    let root = n.sqrt();
    if root.checked_mul(root) == Some(*n) {
        return if root > 1 && root < *n {
            Some(QsResult {
                factor: root,
                stats: empty_stats(1, 2, 1),
            })
        } else {
            None
        };
    }

    let mut bound = config
        .factor_bound
        .unwrap_or_else(|| choose_factor_bound(n))
        .clamp(128, config.max_factor_bound.max(128));
    let mut interval = config
        .interval
        .unwrap_or_else(|| ((bound as i64) * 8).max(1_024))
        .clamp(1, config.max_interval.max(1));

    loop {
        let run_config = ResolvedQsConfig { bound, interval };
        println!("{run_config:?}");
        if let Some(result) = quadratic_sieve_once(n, config, run_config) {
            return Some(result);
        }

        if config.factor_bound.is_some() && config.interval.is_some() {
            return None;
        }
        if bound >= config.max_factor_bound && interval >= config.max_interval {
            return None;
        }
        if bound < config.max_factor_bound {
            bound = bound.saturating_mul(2).min(config.max_factor_bound);
        }
        if interval < config.max_interval {
            interval = interval.saturating_mul(2).min(config.max_interval);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct ResolvedQsConfig {
    bound: u64,
    interval: i64,
}

fn empty_stats(multiplier: u64, factor_bound: u64, interval: i64) -> QsStats {
    QsStats {
        multiplier,
        factor_bound,
        interval,
        factor_base_size: 0,
        polynomials_used: 0,
        full_relations: 0,
        single_large_prime_relations: 0,
        double_large_prime_relations: 0,
    }
}

fn quadratic_sieve_once(
    n: &i128,
    config: &QsConfig,
    resolved: ResolvedQsConfig,
) -> Option<QsResult> {
    let multiplier = if config.use_multiplier {
        choose_multiplier(n, resolved.bound, config.multiplier_search_limit)
    } else {
        1
    };
    if multiplier > 1 && (n % multiplier as i128) == 0 {
        return Some(QsResult {
            factor: multiplier as i128,
            stats: empty_stats(multiplier, resolved.bound, resolved.interval),
        });
    }

    let kn = n.checked_mul(multiplier as i128)?;
    let factor_base = make_factor_base(&kn, resolved.bound);
    if factor_base.primes.len() < 6 {
        return None;
    }

    let needed_relations = factor_base.columns + config.min_relation_margin;
    let mut relations = Vec::with_capacity(needed_relations + 64);
    let mut single_partials: HashMap<u64, PartialRelation> = HashMap::new();
    let mut double_partials = Vec::<DoublePartialRelation>::new();
    let mut double_adjacency = HashMap::<u64, Vec<(u64, usize)>>::new();
    let mut single_large_count = 0_usize;
    let mut double_large_count = 0_usize;
    let mut polynomials_used = 0_usize;

    let polynomial_count = config.max_polynomials.max(needed_relations * 2);
    let polynomials = generate_polynomials(&kn, &factor_base, resolved.interval, polynomial_count);

    for polynomial in polynomials {
        polynomials_used += 1;
        collect_relations_mpqs(
            &kn,
            &polynomial,
            resolved.interval,
            &factor_base,
            needed_relations,
            config,
            &mut relations,
            &mut single_partials,
            &mut double_partials,
            &mut double_adjacency,
            &mut single_large_count,
            &mut double_large_count,
        );

        if let Some(d) = find_factor_from_relations(n, &kn, &factor_base, &relations) {
            return Some(QsResult {
                factor: d,
                stats: QsStats {
                    multiplier,
                    factor_bound: resolved.bound,
                    interval: resolved.interval,
                    factor_base_size: factor_base.primes.len(),
                    polynomials_used,
                    full_relations: relations.len(),
                    single_large_prime_relations: single_large_count,
                    double_large_prime_relations: double_large_count,
                },
            });
        }
    }

    if config.trial_division_fallback && resolved.interval <= 50_000 {
        let m = kn.sqrt().checked_add(1)?;
        let fallback_poly = Polynomial {
            a: 1,
            b: m,
            c: m.checked_mul(m)?.checked_sub(kn)?,
            a_factors: Vec::new(),
        };
        collect_relations_by_trial_division(
            &kn,
            &fallback_poly,
            resolved.interval,
            &factor_base,
            config,
            needed_relations,
            &mut relations,
        );
        if let Some(d) = find_factor_from_relations(n, &kn, &factor_base, &relations) {
            return Some(QsResult {
                factor: d,
                stats: QsStats {
                    multiplier,
                    factor_bound: resolved.bound,
                    interval: resolved.interval,
                    factor_base_size: factor_base.primes.len(),
                    polynomials_used,
                    full_relations: relations.len(),
                    single_large_prime_relations: single_large_count,
                    double_large_prime_relations: double_large_count,
                },
            });
        }
    }

    None
}

fn choose_factor_bound(n: &i128) -> u64 {
    let ln_n = int_ln_approx(*n).max(8.0);
    let ln_ln_n = ln_n.ln().max(1.0);
    let l_n = (ln_n * ln_ln_n).sqrt().exp();
    let b = (l_n / 6.0).sqrt();
    b.clamp(128.0, 1_000_000.0) as u64
}

fn choose_multiplier(n: &i128, factor_bound: u64, search_limit: u64) -> u64 {
    // Knuth/Contini style scoring in a small, deliberately simple form: prefer k such
    // that k*n is a quadratic residue for many small primes, while penalizing large k.
    let primes = primes_up_to(factor_bound.min(300));
    let candidates = (1..=search_limit.max(1))
        .filter(|&k| k == 1 || k % 2 == 1)
        .collect::<Vec<_>>();

    let mut best_k = 1_u64;
    let mut best_score = f64::NEG_INFINITY;

    for k in candidates {
        let Some(kn) = n.checked_mul(k as i128) else {
            continue;
        };
        let mut score = -0.5 * (k as f64).ln();

        for &p in primes.iter().take(80) {
            if p == 2 {
                let r = int_mod_u64(kn, 8);
                score += match r {
                    1 => 2.0 * std::f64::consts::LN_2,
                    3 | 5 => 0.5 * std::f64::consts::LN_2,
                    _ => 0.0,
                };
            } else {
                let r = int_mod_u64(kn, p);
                if r == 0 {
                    score += (p as f64).ln();
                } else if legendre_symbol(r, p) == 1 {
                    score += 2.0 * (p as f64).ln() / ((p - 1) as f64);
                }
            }
        }

        if score > best_score {
            best_score = score;
            best_k = k;
        }
    }

    best_k
}

fn primes_up_to(bound: u64) -> Vec<u64> {
    if bound < 2 {
        return Vec::new();
    }

    let mut sieve = vec![true; (bound + 1) as usize];
    sieve[0] = false;
    sieve[1] = false;

    let mut p = 2_u64;
    while p <= bound / p {
        if sieve[p as usize] {
            let mut q = p * p;
            while q <= bound {
                sieve[q as usize] = false;
                q += p;
            }
        }
        p += 1;
    }

    (2..=bound).filter(|&x| sieve[x as usize]).collect()
}

fn make_factor_base(n: &i128, bound: u64) -> FactorBase {
    let mut entries = Vec::new();

    for p in primes_up_to(bound) {
        let roots = roots_mod_prime(n, p);
        if roots.is_empty() {
            continue;
        }
        let column = entries.len() + 1;
        entries.push(FactorBasePrime {
            p,
            roots,
            ln_p: (p as f64).ln(),
            column,
        });
    }

    let largest_prime = entries.last().map(|e| e.p).unwrap_or(2);
    FactorBase {
        columns: entries.len() + 1,
        primes: entries,
        largest_prime,
    }
}

fn int_mod_u64(x: i128, modulus: u64) -> u64 {
    debug_assert!(modulus > 0);
    x.rem_euclid(modulus as i128) as u64
}

fn mod_pow_u64(base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }

    let mut acc = 1_u128;
    let mut b = (base % modulus) as u128;
    let m = modulus as u128;

    while exp > 0 {
        if exp & 1 == 1 {
            acc = (acc * b) % m;
        }
        b = (b * b) % m;
        exp >>= 1;
    }

    acc as u64
}

fn add_mod_u128(a: u128, b: u128, modulus: u128) -> u128 {
    debug_assert!(a < modulus);
    debug_assert!(b < modulus);
    if a >= modulus - b {
        a - (modulus - b)
    } else {
        a + b
    }
}

fn mul_mod_u128(mut a: u128, mut b: u128, modulus: u128) -> u128 {
    debug_assert!(modulus > 0);
    a %= modulus;
    b %= modulus;
    let mut acc = 0_u128;

    while b > 0 {
        if b & 1 == 1 {
            acc = add_mod_u128(acc, a, modulus);
        }
        a = add_mod_u128(a, a, modulus);
        b >>= 1;
    }

    acc
}

fn mod_pow_u128(mut base: u128, mut exp: u32, modulus: u128) -> u128 {
    if modulus == 1 {
        return 0;
    }

    base %= modulus;
    let mut acc = 1_u128 % modulus;

    while exp > 0 {
        if exp & 1 == 1 {
            acc = mul_mod_u128(acc, base, modulus);
        }
        base = mul_mod_u128(base, base, modulus);
        exp >>= 1;
    }

    acc
}

fn mod_inverse_u64(a: u64, modulus: u64) -> Option<u64> {
    if modulus == 0 {
        return None;
    }
    let mut old_r = a as i128;
    let mut r = modulus as i128;
    let mut old_s = 1_i128;
    let mut s = 0_i128;

    while r != 0 {
        let q = old_r / r;
        let new_r = old_r - q * r;
        old_r = r;
        r = new_r;
        let new_s = old_s - q * s;
        old_s = s;
        s = new_s;
    }

    if old_r != 1 {
        return None;
    }

    let m = modulus as i128;
    Some(old_s.rem_euclid(m) as u64)
}

fn legendre_symbol(a: u64, p: u64) -> i8 {
    debug_assert!(p > 2 && p % 2 == 1);
    let a = a % p;
    if a == 0 {
        return 0;
    }

    match mod_pow_u64(a, (p - 1) / 2, p) {
        1 => 1,
        x if x == p - 1 => -1,
        _ => 0,
    }
}

fn sqrt_mod_prime_odd(n: u64, p: u64) -> Option<u64> {
    let n = n % p;
    if n == 0 {
        return Some(0);
    }
    if legendre_symbol(n, p) != 1 {
        return None;
    }
    if p % 4 == 3 {
        return Some(mod_pow_u64(n, (p + 1) / 4, p));
    }

    let mut q = p - 1;
    let mut s = 0_u32;
    while q.is_multiple_of(2) {
        q /= 2;
        s += 1;
    }

    let mut z = 2_u64;
    while legendre_symbol(z, p) != -1 {
        z += 1;
    }

    let mut c = mod_pow_u64(z, q, p);
    let mut x = mod_pow_u64(n, q.div_ceil(2), p);
    let mut t = mod_pow_u64(n, q, p);
    let mut m = s;

    while t != 1 {
        let mut i = 1_u32;
        let mut t2i = ((t as u128 * t as u128) % p as u128) as u64;
        while i < m && t2i != 1 {
            t2i = ((t2i as u128 * t2i as u128) % p as u128) as u64;
            i += 1;
        }
        if i == m {
            return None;
        }

        let b = mod_pow_u64(c, 1_u64 << (m - i - 1), p);
        x = ((x as u128 * b as u128) % p as u128) as u64;
        c = ((b as u128 * b as u128) % p as u128) as u64;
        t = ((t as u128 * c as u128) % p as u128) as u64;
        m = i;
    }

    Some(x)
}

fn roots_mod_prime(n: &i128, p: u64) -> Vec<u64> {
    let n_mod = int_mod_u64(*n, p);

    if p == 2 {
        return vec![n_mod & 1];
    }

    let Some(r) = sqrt_mod_prime_odd(n_mod, p) else {
        return Vec::new();
    };

    let r2 = if r == 0 { 0 } else { p - r };
    if r == r2 { vec![r] } else { vec![r, r2] }
}

fn crt_pair(r1: i128, m1: i128, r2: u64, m2: u64) -> Option<i128> {
    let m1_mod = int_mod_u64(m1, m2);
    let inv = mod_inverse_u64(m1_mod, m2)?;
    let r1_mod = int_mod_u64(r1, m2);
    let t = ((r2 + m2 - r1_mod) as u128 * inv as u128 % m2 as u128) as u64;
    r1.checked_add(m1.checked_mul(t as i128)?)
}

fn generate_polynomials(
    n: &i128,
    factor_base: &FactorBase,
    interval: i64,
    max_count: usize,
) -> Vec<Polynomial> {
    let odd_primes = factor_base
        .primes
        .iter()
        .filter(|entry| entry.p > 2 && entry.roots.len() >= 2)
        .cloned()
        .collect::<Vec<_>>();

    if odd_primes.len() < 3 {
        let m = n.sqrt().checked_add(1).unwrap_or(*n);
        let Some(c) = m.checked_mul(m).and_then(|v| v.checked_sub(*n)) else {
            return Vec::new();
        };
        return vec![Polynomial {
            a: 1,
            b: m,
            c,
            a_factors: Vec::new(),
        }];
    }

    let target_a_ln =
        (0.5 * int_ln_approx(*n) + 0.5 * 2.0_f64.ln() - (interval.max(1) as f64).ln()).max(1.0);

    let target_k = (target_a_ln / (factor_base.largest_prime as f64).ln().max(2.0))
        .ceil()
        .clamp(3.0, 10.0) as usize;
    let target_prime_ln = target_a_ln / target_k as f64;

    let mut sorted = odd_primes;
    sorted.sort_by(|a, b| {
        let da = ((a.p as f64).ln() - target_prime_ln).abs();
        let db = ((b.p as f64).ln() - target_prime_ln).abs();
        da.total_cmp(&db)
    });

    let pool_len = sorted.len().min((target_k * 8).max(24));
    let pool = &sorted[..pool_len];
    let mut polynomials = Vec::new();
    let mut used_a = HashMap::<i128, usize>::new();

    for start in 0..pool.len() {
        if polynomials.len() >= max_count {
            break;
        }
        let mut factors = Vec::new();
        let mut a = 1_i128;

        for j in 0..target_k {
            let entry = &pool[(start + j * 3) % pool.len()];
            factors.push(entry.clone());
            let Some(next_a) = a.checked_mul(entry.p as i128) else {
                factors.clear();
                break;
            };
            a = next_a;
        }
        factors.sort_by_key(|e| e.p);
        factors.dedup_by_key(|e| e.p);
        if factors.len() < 2 {
            continue;
        }
        let Some(recomputed_a) = factors
            .iter()
            .try_fold(1_i128, |acc, entry| acc.checked_mul(entry.p as i128))
        else {
            continue;
        };
        a = recomputed_a;

        let counter = used_a.entry(a).or_insert(0);
        let sign_masks_to_try = 1_usize << factors.len().min(8);
        let first_mask = *counter;
        *counter += sign_masks_to_try;

        for local_mask in 0..sign_masks_to_try {
            if polynomials.len() >= max_count {
                break;
            }
            let mask = first_mask + local_mask;
            let mut b = 0_i128;
            let mut modulus = 1_i128;
            let mut ok = true;

            for (i, entry) in factors.iter().enumerate() {
                let roots = &entry.roots;
                let selected = if ((mask >> i) & 1) == 0 {
                    roots[0]
                } else {
                    *roots.last().unwrap_or(&roots[0])
                };
                let Some(new_b) = crt_pair(b, modulus, selected, entry.p) else {
                    ok = false;
                    break;
                };
                b = new_b;
                let Some(new_modulus) = modulus.checked_mul(entry.p as i128) else {
                    ok = false;
                    break;
                };
                modulus = new_modulus;
            }

            if !ok || modulus != a {
                continue;
            }

            // Use a centered B to keep |Q(x)| smaller near x=0.
            if b.checked_mul(2).is_some_and(|v| v > a) {
                b -= a;
            }

            let Some(numerator) = b.checked_mul(b).and_then(|v| v.checked_sub(*n)) else {
                continue;
            };
            if numerator % a != 0 {
                continue;
            }
            let c = numerator / a;
            let a_factors = factors.iter().map(|e| e.p).collect::<Vec<_>>();
            polynomials.push(Polynomial { a, b, c, a_factors });
        }
    }

    if polynomials.is_empty() {
        let m = n.sqrt().checked_add(1).unwrap_or(*n);
        if let Some(c) = m.checked_mul(m).and_then(|v| v.checked_sub(*n)) {
            polynomials.push(Polynomial {
                a: 1,
                b: m,
                c,
                a_factors: Vec::new(),
            });
        }
    }

    polynomials
}

fn polynomial_x(poly: &Polynomial, x: i64) -> Option<i128> {
    poly.a
        .checked_mul(x as i128)
        .and_then(|v| v.checked_add(poly.b))
}

fn polynomial_q(poly: &Polynomial, x: i64) -> Option<i128> {
    let x = x as i128;
    let x2 = x.checked_mul(x)?;
    let ax2 = poly.a.checked_mul(x2)?;
    let two_bx = 2_i128.checked_mul(poly.b)?.checked_mul(x)?;
    ax2.checked_add(two_bx)?.checked_add(poly.c)
}

fn add_a_factor_exponents(exponents: &mut [u32], factor_base: &FactorBase, a_factors: &[u64]) {
    for &p in a_factors {
        if let Some(entry) = factor_base.primes.iter().find(|entry| entry.p == p) {
            exponents[entry.column] += 1;
        }
    }
}

fn parity_from_exponents(exponents: &[u32]) -> BitSet {
    let mut parity = BitSet::new(exponents.len());
    for (i, &e) in exponents.iter().enumerate() {
        if e % 2 == 1 {
            parity.set(i);
        }
    }
    parity
}

fn factor_q_over_base(
    qx: i128,
    poly: &Polynomial,
    factor_base: &FactorBase,
    config: &QsConfig,
    allow_large_prime: bool,
) -> Option<(Vec<u32>, BitSet, LargeRemainder)> {
    if qx == 0 {
        return None;
    }

    let mut rest = qx;
    let mut exponents = vec![0_u32; factor_base.columns];

    add_a_factor_exponents(&mut exponents, factor_base, &poly.a_factors);

    if rest < 0 {
        exponents[0] += 1;
        rest = rest.checked_neg()?;
    }

    for entry in &factor_base.primes {
        let p = entry.p as i128;
        while rest % p == 0 {
            rest /= p;
            exponents[entry.column] += 1;
        }
    }

    let parity = parity_from_exponents(&exponents);
    if rest == 1 {
        return Some((exponents, parity, LargeRemainder::None));
    }

    if allow_large_prime
        && config.use_large_primes
        && let Ok(lp) = u64::try_from(rest)
    {
        let b = factor_base.largest_prime.max(2);
        let single_limit = b.saturating_mul(b).saturating_mul(128).max(b + 1);
        if lp > b && lp <= single_limit && is_prime_u64(lp) {
            return Some((exponents, parity, LargeRemainder::Single(lp)));
        }

        let double_limit = single_limit.saturating_mul(b.max(2)).min(u64::MAX / 4);
        if config.use_double_large_primes
            && lp > b
            && lp <= double_limit
            && let Some((p1, p2)) = split_two_large_primes(lp, b)
        {
            return Some((exponents, parity, LargeRemainder::Double(p1, p2)));
        }
    }

    None
}

fn is_prime_u64(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n.is_multiple_of(2) {
        return n == 2;
    }
    let mut d = 3_u64;
    while d <= n / d {
        if n.is_multiple_of(d) {
            return false;
        }
        d += 2;
    }
    true
}

fn split_two_large_primes(n: u64, factor_base_bound: u64) -> Option<(u64, u64)> {
    if n <= factor_base_bound.saturating_mul(factor_base_bound) {
        return None;
    }
    if is_prime_u64(n) {
        return None;
    }

    let mut d = 2_u64;
    while d <= n / d {
        if n.is_multiple_of(d) {
            let q = n / d;
            if d > factor_base_bound && q > factor_base_bound && is_prime_u64(d) && is_prime_u64(q)
            {
                return Some((d.min(q), d.max(q)));
            }
            return None;
        }
        d += if d == 2 { 1 } else { 2 };
    }

    None
}

fn build_relation(
    n: &i128,
    poly: &Polynomial,
    x: i64,
    factor_base: &FactorBase,
    config: &QsConfig,
    allow_large_prime: bool,
) -> Option<(Relation, LargeRemainder)> {
    let qx = polynomial_q(poly, x)?;
    let (exponents, parity, large_remainder) =
        factor_q_over_base(qx, poly, factor_base, config, allow_large_prime)?;
    let x_value = polynomial_x(poly, x)?.mod_floor(n);

    Some((
        Relation {
            x_value,
            exponents,
            square_factors: Vec::new(),
            parity,
        },
        large_remainder,
    ))
}

fn collect_relations_by_trial_division(
    n: &i128,
    poly: &Polynomial,
    interval: i64,
    factor_base: &FactorBase,
    config: &QsConfig,
    needed_relations: usize,
    relations: &mut Vec<Relation>,
) {
    for x in -interval..=interval {
        if let Some((relation, LargeRemainder::None)) =
            build_relation(n, poly, x, factor_base, config, false)
        {
            relations.push(relation);
            if relations.len() >= needed_relations {
                return;
            }
        }
    }
}

fn first_integer_in_range_with_residue(min: i64, residue: i64, modulus: i64) -> i64 {
    min + (residue - min).rem_euclid(modulus)
}

fn int_ln_approx(x: i128) -> f64 {
    if x == 0 {
        return f64::NEG_INFINITY;
    }
    let v = (x as f64).abs();
    if v.is_finite() && v > 0.0 {
        return v.ln();
    }
    let digits = x.unsigned_abs().to_string().len() as f64;
    digits * std::f64::consts::LN_10
}

fn collect_relations_mpqs(
    n: &i128,
    poly: &Polynomial,
    interval: i64,
    factor_base: &FactorBase,
    needed_relations: usize,
    config: &QsConfig,
    relations: &mut Vec<Relation>,
    single_partials: &mut HashMap<u64, PartialRelation>,
    double_partials: &mut Vec<DoublePartialRelation>,
    double_adjacency: &mut HashMap<u64, Vec<(u64, usize)>>,
    single_large_count: &mut usize,
    double_large_count: &mut usize,
) {
    if interval <= 0 {
        return;
    }
    let size = match 2_i64.checked_mul(interval).and_then(|v| v.checked_add(1)) {
        Some(v) if v > 0 => v as usize,
        _ => return,
    };
    let offset = interval;
    let mut residual_logs = vec![0.0_f64; size];

    for x in -interval..=interval {
        residual_logs[(x + offset) as usize] = polynomial_q(poly, x)
            .map(int_ln_approx)
            .unwrap_or(f64::INFINITY);
    }

    for entry in &factor_base.primes {
        let p = entry.p;
        if p == 0 {
            continue;
        }
        let a_mod = int_mod_u64(poly.a, p);
        let Some(a_inv) = mod_inverse_u64(a_mod, p) else {
            // p divides A. The A contribution is already known; exact verification will
            // handle any additional power of p. Skipping this prime keeps root handling simple.
            continue;
        };
        let b_mod = int_mod_u64(poly.b, p);
        let p_i64 = p as i64;

        for &root in &entry.roots {
            let residue = (((root + p - b_mod) as u128 * a_inv as u128) % p as u128) as i64;
            let mut x = first_integer_in_range_with_residue(-interval, residue, p_i64);
            while x <= interval {
                residual_logs[(x + offset) as usize] -= entry.ln_p;
                x += p_i64;
            }
        }
    }

    let log_b = (factor_base.largest_prime as f64).ln().max(1.0);
    let threshold = 1.8 * log_b;
    let lp_threshold = if config.use_double_large_primes {
        3.4 * log_b
    } else {
        2.8 * log_b
    };
    let mut candidates = (-interval..=interval)
        .filter_map(|x| {
            let score = residual_logs[(x + offset) as usize];
            (score <= lp_threshold).then_some((x, score))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| a.1.total_cmp(&b.1));

    for (x, score) in candidates {
        let allow_large = score > threshold || relations.len() + 8 < needed_relations;
        let Some((relation, large_remainder)) =
            build_relation(n, poly, x, factor_base, config, allow_large)
        else {
            continue;
        };

        match large_remainder {
            LargeRemainder::None => {
                relations.push(relation);
            }
            LargeRemainder::Single(lp) => {
                if let Some(previous) = single_partials.remove(&lp) {
                    let combined =
                        combine_single_large_prime_relations(n, previous.relation, relation, lp);
                    relations.push(combined);
                    *single_large_count += 1;
                } else {
                    single_partials.insert(lp, PartialRelation { relation });
                }
            }
            LargeRemainder::Double(p1, p2) => {
                if let Some(path) = find_double_large_prime_path(double_adjacency, p1, p2) {
                    let combined = combine_double_large_prime_cycle(
                        n,
                        relation,
                        p1,
                        p2,
                        &path,
                        double_partials,
                    );
                    relations.push(combined);
                    *double_large_count += 1;
                } else {
                    let edge_id = double_partials.len();
                    double_partials.push(DoublePartialRelation { relation, p1, p2 });
                    double_adjacency.entry(p1).or_default().push((p2, edge_id));
                    double_adjacency.entry(p2).or_default().push((p1, edge_id));
                }
            }
        }

        if relations.len() >= needed_relations + 32 {
            return;
        }
    }
}

fn find_double_large_prime_path(
    adjacency: &HashMap<u64, Vec<(u64, usize)>>,
    start: u64,
    goal: u64,
) -> Option<Vec<usize>> {
    use std::collections::{HashSet, VecDeque};

    if start == goal {
        return Some(Vec::new());
    }

    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    let mut parent = HashMap::<u64, (u64, usize)>::new();

    visited.insert(start);
    queue.push_back(start);

    while let Some(v) = queue.pop_front() {
        for &(next, edge_id) in adjacency.get(&v).into_iter().flatten() {
            if visited.insert(next) {
                parent.insert(next, (v, edge_id));
                if next == goal {
                    let mut path = Vec::new();
                    let mut cur = goal;
                    while cur != start {
                        let (prev, e) = parent[&cur];
                        path.push(e);
                        cur = prev;
                    }
                    return Some(path);
                }
                queue.push_back(next);
            }
        }
    }

    None
}

fn combine_single_large_prime_relations(
    n: &i128,
    left: Relation,
    right: Relation,
    large_prime: u64,
) -> Relation {
    let mut exponents = left.exponents.clone();
    for (a, b) in exponents.iter_mut().zip(right.exponents.iter()) {
        *a += *b;
    }
    let parity = parity_from_exponents(&exponents);
    let mut square_factors = left.square_factors;
    square_factors.extend(right.square_factors);
    square_factors.push(large_prime);
    let modulus = *n as u128;
    let x_value = mul_mod_u128(left.x_value as u128, right.x_value as u128, modulus) as i128;

    Relation {
        x_value,
        exponents,
        square_factors,
        parity,
    }
}

fn combine_double_large_prime_cycle(
    n: &i128,
    current: Relation,
    current_p1: u64,
    current_p2: u64,
    path_edges: &[usize],
    double_partials: &[DoublePartialRelation],
) -> Relation {
    let mut exponents = current.exponents.clone();
    let mut x_value = current.x_value;
    let mut large_counts = HashMap::<u64, u32>::new();

    *large_counts.entry(current_p1).or_insert(0) += 1;
    *large_counts.entry(current_p2).or_insert(0) += 1;
    let modulus = *n as u128;

    for &edge_id in path_edges {
        let edge = &double_partials[edge_id];
        x_value = mul_mod_u128(x_value as u128, edge.relation.x_value as u128, modulus) as i128;
        for (a, b) in exponents.iter_mut().zip(edge.relation.exponents.iter()) {
            *a += *b;
        }
        for &p in &[edge.p1, edge.p2] {
            *large_counts.entry(p).or_insert(0) += 1;
        }
    }

    let parity = parity_from_exponents(&exponents);
    let mut square_factors = current.square_factors;
    for &edge_id in path_edges {
        square_factors.extend(
            double_partials[edge_id]
                .relation
                .square_factors
                .iter()
                .copied(),
        );
    }
    for (p, e) in large_counts {
        for _ in 0..(e / 2) {
            square_factors.push(p);
        }
    }

    Relation {
        x_value,
        exponents,
        square_factors,
        parity,
    }
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

        if v.is_zero() {
            let dependency = combination.indices(n_rows);
            if !dependency.is_empty() {
                dependencies.push(dependency);
            }
        }
    }

    dependencies
}

fn find_factor_from_relations(
    original_n: &i128,
    sieve_n: &i128,
    factor_base: &FactorBase,
    relations: &[Relation],
) -> Option<i128> {
    if relations.len() <= factor_base.columns {
        return None;
    }

    let rows = relations
        .iter()
        .map(|relation| relation.parity.clone())
        .collect::<Vec<_>>();

    for dependency in find_gf2_dependencies(&rows, factor_base.columns) {
        if let Some(d) =
            build_congruence_factor(original_n, sieve_n, factor_base, relations, &dependency)
        {
            return Some(d);
        }
    }

    None
}

fn build_congruence_factor(
    original_n: &i128,
    sieve_n: &i128,
    factor_base: &FactorBase,
    relations: &[Relation],
    dependency: &[usize],
) -> Option<i128> {
    let modulus = u128::try_from(*sieve_n).ok()?;
    let mut x_prod = 1_u128 % modulus;
    let mut exponent_sums = vec![0_u32; factor_base.columns];
    let mut square_factors = Vec::<u64>::new();

    for &i in dependency {
        let relation = &relations[i];
        x_prod = mul_mod_u128(x_prod, u128::try_from(relation.x_value).ok()?, modulus);
        for (sum, &e) in exponent_sums.iter_mut().zip(relation.exponents.iter()) {
            *sum = sum.checked_add(e)?;
        }
        square_factors.extend(relation.square_factors.iter().copied());
    }

    if !exponent_sums[0].is_multiple_of(2) {
        return None;
    }

    let mut y_prod = 1_u128 % modulus;

    for entry in &factor_base.primes {
        let e = exponent_sums[entry.column];
        if !e.is_multiple_of(2) {
            return None;
        }
        let half_exp = e / 2;
        if half_exp > 0 {
            y_prod = mul_mod_u128(
                y_prod,
                mod_pow_u128(entry.p as u128, half_exp, modulus),
                modulus,
            );
        }
    }

    for p in square_factors {
        y_prod = mul_mod_u128(y_prod, p as u128, modulus);
    }

    let x_prod = i128::try_from(x_prod).ok()?;
    let y_prod = i128::try_from(y_prod).ok()?;
    let d1 = (x_prod - y_prod).abs().gcd(original_n);
    if d1 > 1 && d1 < *original_n {
        return Some(d1);
    }

    let original_modulus = u128::try_from(*original_n).ok()?;
    let sum_mod_original = add_mod_u128(
        (x_prod as u128) % original_modulus,
        (y_prod as u128) % original_modulus,
        original_modulus,
    ) as i128;
    let d2 = sum_mod_original.gcd(original_n);
    if d2 > 1 && d2 < *original_n {
        return Some(d2);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_nontrivial_factor(n: i128, d: i128) {
        assert!(d > 1, "d must be > 1: {d}");
        assert!(d < n, "d must be < n: d={d}, n={n}");
        assert!(n % d == 0, "d must divide n: d={d}, n={n}");
    }

    #[test]
    fn qs2_returns_two_for_even_composite() {
        let n = 100_i128;
        let d = quadratic_sieve(&n).expect("should find factor");
        assert_eq!(d, 2);
    }

    #[test]
    fn qs2_returns_square_root_for_square() {
        let n = 101_i128.pow(2);
        let d = quadratic_sieve(&n).expect("should find factor");
        assert_eq!(d, 101);
    }

    #[test]
    fn qs2_factors_small_semiprime() {
        let n = 8051_i128; // 83 * 97
        let d = quadratic_sieve_with_config(
            &n,
            &QsConfig {
                factor_bound: Some(100),
                interval: Some(2000),
                ..Default::default()
            },
        )
        .expect("should find factor")
        .factor;
        assert_nontrivial_factor(n, d);
    }

    #[test]
    fn qs2_factors_medium_semiprime() {
        let n = 405003390007_i128; // 270001 * 1500007
        let d = quadratic_sieve_with_config(
            &n,
            &QsConfig {
                factor_bound: Some(500),
                interval: Some(20_000),
                ..Default::default()
            },
        )
        .expect("should find factor")
        .factor;
        assert_nontrivial_factor(n, d);
    }

    #[test]
    fn qs2_generates_mpqs_polynomials() {
        let n = 8051_i128;
        let fb = make_factor_base(&n, 100);
        let polys = generate_polynomials(&n, &fb, 1000, 10);
        assert!(!polys.is_empty());
    }
}
