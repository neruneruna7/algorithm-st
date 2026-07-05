// use num_bigint::BigInt;
// use num_integer::Integer;
// use num_traits::{One as _, Signed as _, ToPrimitive as _, Zero as _};
// use rand::Rng;

// use crate::{miller_rabin::miller_rabin, rho::rho_method};

// /// 素数生成
// pub fn primes_leq(limit: u128) -> Vec<u128> {
//     let mut primes = Vec::new();

//     'outer: for x in 2..=limit {
//         for &p in &primes {
//             if p * p > x {
//                 break;
//             }

//             if x % p == 0 {
//                 continue 'outer;
//             }
//         }

//         primes.push(x);
//     }

//     primes
// }

// fn prime_factorize(n: &BigInt, rng: &mut impl Rng) -> Vec<BigInt> {
//     let one = BigInt::one();
//     let two = BigInt::from(2_u32);

//     let mut stack = vec![n.clone()];
//     let mut factors = Vec::new();

//     while let Some(x) = stack.pop() {
//         if x < two {
//             continue;
//         }

//         if x == two {
//             factors.push(two.clone());
//             continue;
//         }

//         if (&x % 2_u32).is_zero() {
//             factors.push(two.clone());
//             stack.push(&x / 2_u32);
//             continue;
//         }

//         if miller_rabin(&x, 20, rng) {
//             factors.push(x);
//             continue;
//         }

//         let d = loop {
//             match rho_method(&x, rng) {
//                 Some(d) if d > one && d < x => break d,
//                 _ => continue,
//             }
//         };

//         let q = &x / &d;

//         stack.push(d);
//         stack.push(q);
//     }

//     factors.sort();
//     factors
// }

// /// 素因数を指数としたベクトルと、各素因数の偶奇性を表すベクトルに変換する
// fn factors_to_vectors(factors: &[BigInt], factor_base: &[u128]) -> Option<(Vec<u32>, Vec<u8>)> {
//     let minus_one = BigInt::from(-1);
//     let mut exponents = vec![0_u32; factor_base.len() + 1];

//     for f in factors {
//         if f == &minus_one {
//             exponents[0] += 1;
//             continue;
//         }

//         let p = f.to_u128()?;

//         let pos = factor_base.iter().position(|&q| q == p)?;

//         exponents[pos + 1] += 1;
//     }

//     let parity = exponents.iter().map(|e| (e & 1) as u8).collect();

//     Some((exponents, parity))
// }

// #[derive(Debug, Clone)]
// struct Relation {
//     x: i128,
//     qx: BigInt,
//     exponents: Vec<u32>,
//     parity: Vec<u8>,
// }

// #[derive(Debug, Clone)]
// struct GF2Row {
//     bits: Vec<u8>,
//     combo: Vec<u8>,
// }

// /// GF(2)
// fn find_dependencies_gf2(parities: &[Vec<u8>]) -> Vec<Vec<usize>> {
//     let row_count = parities.len();

//     if row_count == 0 {
//         return Vec::new();
//     }

//     let col_count = parities[0].len();

//     let mut rows = parities
//         .iter()
//         .enumerate()
//         .map(|(i, parity)| {
//             let mut combo = vec![0_u8; row_count];
//             combo[i] = 1;

//             GF2Row {
//                 bits: parity.clone(),
//                 combo,
//             }
//         })
//         .collect::<Vec<_>>();

//     let mut pivot_row = 0;

//     for col in 0..col_count {
//         let pivot = (pivot_row..row_count).find(|&r| rows[r].bits[col] == 1);

//         let Some(pivot) = pivot else {
//             continue;
//         };

//         rows.swap(pivot_row, pivot);

//         for r in 0..row_count {
//             if r != pivot_row && rows[r].bits[col] == 1 {
//                 for c in col..col_count {
//                     rows[r].bits[c] ^= rows[pivot_row].bits[c];
//                 }

//                 for k in 0..row_count {
//                     rows[r].combo[k] ^= rows[pivot_row].combo[k];
//                 }
//             }
//         }

//         pivot_row += 1;

//         if pivot_row == row_count {
//             break;
//         }
//     }

//     rows.into_iter()
//         .filter(|row| row.bits.iter().all(|&b| b == 0))
//         .filter_map(|row| {
//             let indices = row
//                 .combo
//                 .iter()
//                 .enumerate()
//                 .filter_map(|(i, &b)| if b == 1 { Some(i) } else { None })
//                 .collect::<Vec<_>>();

//             if indices.is_empty() {
//                 None
//             } else {
//                 Some(indices)
//             }
//         })
//         .collect()
// }

// fn build_xy_from_dependency(
//     n: &BigInt,
//     m: &BigInt,
//     relations: &[Relation],
//     indices: &[usize],
//     factor_base: &[u128],
// ) -> Option<(BigInt, BigInt)> {
//     let mut x_prod = BigInt::one();

//     // exponents[0] は -1 用.
//     let mut exp_sum = vec![0_u32; factor_base.len() + 1];

//     for &i in indices {
//         let rel = &relations[i];

//         // rel.x は offset なので，実際の x_tilde を復元する.
//         let x_tilde = m + rel.x;

//         x_prod = (x_prod * x_tilde) % n;

//         for (s, e) in exp_sum.iter_mut().zip(&rel.exponents) {
//             *s += *e;
//         }
//     }

//     // -1 の指数が奇数なら，積は正の平方数ではない.
//     // 正しく dependency が取れていれば偶数になるはず.
//     if exp_sum[0] % 2 != 0 {
//         return None;
//     }

//     let mut y = BigInt::one();

//     for (j, &p) in factor_base.iter().enumerate() {
//         let e = exp_sum[j + 1];

//         if e % 2 != 0 {
//             return None;
//         }

//         let p_big = BigInt::from(p);

//         for _ in 0..(e / 2) {
//             y *= &p_big;
//         }
//     }

//     Some((x_prod, y % n))
// }

// fn extract_factor(n: &BigInt, x: &BigInt, y: &BigInt) -> Option<BigInt> {
//     let one = BigInt::one();

//     let d1 = (x - y).abs().gcd(n);

//     if d1 > one && d1 < *n {
//         return Some(d1);
//     }

//     let d2 = (x + y).abs().gcd(n);

//     if d2 > one && d2 < *n {
//         return Some(d2);
//     }

//     None
// }

// fn divide_over_factor_base(qx: &BigInt, factor_base: &[u128]) -> Option<(Vec<u32>, Vec<u8>)> {
//     if qx.is_zero() {
//         return None;
//     }

//     // exponents[0] は -1 用.
//     let mut exponents = vec![0_u32; factor_base.len() + 1];

//     let mut rem = if qx.is_negative() {
//         exponents[0] = 1;
//         -qx.clone()
//     } else {
//         qx.clone()
//     };

//     for (i, &p) in factor_base.iter().enumerate() {
//         let p_big = BigInt::from(p);
//         let mut e = 0_u32;

//         while (&rem % &p_big).is_zero() {
//             rem /= &p_big;
//             e += 1;
//         }

//         exponents[i + 1] = e;
//     }

//     if rem != BigInt::one() {
//         return None;
//     }

//     let parity = exponents.iter().map(|e| (e & 1) as u8).collect::<Vec<_>>();

//     Some((exponents, parity))
// }

// #[derive(Debug, Clone)]
// struct Candidate {
//     x: i128,
//     qx: BigInt,
//     rem: BigInt,
//     exponents: Vec<u32>,
// }

// impl Candidate {
//     fn new(n: &BigInt, m: &BigInt, x: i128) -> Option<Self> {
//         let x_tilde = m + BigInt::from(x);
//         let qx = &x_tilde * &x_tilde - n;

//         if qx.is_zero() {
//             return None;
//         }

//         let mut exponents = Vec::new();

//         let rem = if qx.is_negative() {
//             exponents.push(1); // -1
//             -qx.clone()
//         } else {
//             exponents.push(0); // -1
//             qx.clone()
//         };

//         Some(Self {
//             x,
//             qx,
//             rem,
//             exponents,
//         })
//     }

//     fn divide_by_prime(&mut self, p: u128) {
//         let p_big = BigInt::from(p);
//         let mut e = 0_u32;

//         while (&self.rem % &p_big).is_zero() {
//             self.rem /= &p_big;
//             e += 1;
//         }

//         self.exponents.push(e);
//     }

//     fn is_full_relation(&self) -> bool {
//         self.rem == BigInt::one()
//     }

//     fn to_relation(&self) -> Option<Relation> {
//         if !self.is_full_relation() {
//             return None;
//         }

//         let parity = self
//             .exponents
//             .iter()
//             .map(|e| (e & 1) as u8)
//             .collect::<Vec<_>>();

//         Some(Relation {
//             x: self.x,
//             qx: self.qx.clone(),
//             exponents: self.exponents.clone(),
//             parity,
//         })
//     }
// }
// use std::collections::BTreeMap;

// struct QSState {
//     n: BigInt,
//     m: BigInt,

//     factor_base: Vec<u128>,

//     searched_start: i128,
//     searched_end: i128, // exclusive

//     candidates: BTreeMap<i128, Candidate>,
// }
// impl QSState {
//     fn new(n: &BigInt, initial_range: u128, initial_prime_bound: u128) -> Option<Self> {
//         let m = n.sqrt();

//         let factor_base = primes_leq(initial_prime_bound);

//         let mut state = Self {
//             n: n.clone(),
//             m,
//             factor_base,
//             searched_start: 0,
//             searched_end: 0,
//             candidates: BTreeMap::new(),
//         };

//         state.extend_x_range_to(initial_range)?;

//         Some(state)
//     }
//     fn extend_x_range_to(&mut self, radius: u128) -> Option<()> {
//         let radius = i128::try_from(radius).ok()?;
//         let new_start = -radius;
//         let new_end = radius;

//         if new_start >= self.searched_start && new_end <= self.searched_end {
//             return Some(());
//         }

//         for x in new_start..self.searched_start {
//             self.add_candidate_if_absent(x);
//         }

//         for x in self.searched_end..new_end {
//             self.add_candidate_if_absent(x);
//         }

//         self.searched_start = self.searched_start.min(new_start);
//         self.searched_end = self.searched_end.max(new_end);

//         Some(())
//     }

//     fn add_candidate_if_absent(&mut self, x: i128) {
//         if self.candidates.contains_key(&x) {
//             return;
//         }

//         let Some(mut cand) = Candidate::new(&self.n, &self.m, x) else {
//             return;
//         };

//         for &p in &self.factor_base {
//             cand.divide_by_prime(p);
//         }

//         debug_assert_eq!(cand.exponents.len(), self.factor_base.len() + 1);

//         self.candidates.insert(x, cand);
//     }
//     fn extend_factor_base_to(&mut self, new_bound: u128) {
//         let old_len = self.factor_base.len();

//         let new_factor_base = primes_leq(new_bound);

//         if new_factor_base.len() <= old_len {
//             return;
//         }

//         let new_primes = &new_factor_base[old_len..];

//         for &p in new_primes {
//             for cand in self.candidates.values_mut() {
//                 cand.divide_by_prime(p);
//             }
//         }

//         self.factor_base = new_factor_base;

//         for cand in self.candidates.values() {
//             debug_assert_eq!(cand.exponents.len(), self.factor_base.len() + 1);
//         }
//     }
//     fn full_relations(&self) -> Vec<Relation> {
//         self.candidates
//             .values()
//             .filter_map(|cand| cand.to_relation())
//             .collect()
//     }

//     fn column_count(&self) -> usize {
//         self.factor_base.len() + 1
//     }
// }

// pub fn quadratic_sieve1(n: &BigInt, x_range: u128, primes: &[u128]) -> Option<BigInt> {
//     // 2次ふるい法
//     //　因数分解したい数をn
//     let m = n.sqrt();
//     let x_range = i128::try_from(x_range).ok()?;
//     let factors_vec = (-x_range..x_range)
//         .filter_map(|x| {
//             let x_tilde = &m + BigInt::from(x);
//             let qx: BigInt = &x_tilde * &x_tilde - n;

//             let (exponents, parity) = divide_over_factor_base(&qx, primes)?;

//             Some(Relation {
//                 x,
//                 qx,
//                 exponents,
//                 parity,
//             })
//         })
//         .collect::<Vec<_>>();
//     let parities = factors_vec
//         .iter()
//         .map(|rel| rel.parity.clone())
//         .collect::<Vec<_>>();

//     let column_count = primes.len() + 1;

//     println!("relations={}, columns={}", factors_vec.len(), column_count);

//     let dependencies = find_dependencies_gf2(&parities);

//     if dependencies.is_empty() {
//         panic!("no dependency found; collect more relations");
//     }
//     println!("dependencies = {:?}", dependencies);

//     // factors_vec.iter().for_each(|i| {
//     //     println!("{:?}", i);
//     // });

//     // factors_iter.for_each(|(x, qx, factors)| {
//     //     println!("factors: x={} qx={:?} factors={:?}", x, qx, factors);
//     // });

//     for dependency in dependencies {
//         let Some((x, y)) = build_xy_from_dependency(n, &m, &factors_vec, &dependency, primes)
//         else {
//             continue;
//         };

//         println!("candidate: x={}, y={}, dependency={:?}", x, y, dependency);

//         if let Some(d) = extract_factor(n, &x, &y) {
//             return Some(d);
//         }
//     }

//     None
// }

// pub fn quadratic_sieve_adaptive(
//     n: &BigInt,
//     max_prime_bound: u128,
//     max_x_range: u128,
// ) -> Option<BigInt> {
//     let mut prime_bound = 200_u128;
//     let mut x_range = 8_000_u128;

//     let mut state = QSState::new(n, x_range, prime_bound)?;

//     loop {
//         let relations = state.full_relations();

//         println!(
//             "prime_bound={}, x_range={}, candidates={}, relations={}, columns={}",
//             prime_bound,
//             x_range,
//             state.candidates.len(),
//             relations.len(),
//             state.column_count(),
//         );

//         if let Some(d) = try_relations(&state.n, &state.m, &state.factor_base, &relations) {
//             return Some(d);
//         }

//         if prime_bound >= max_prime_bound && x_range >= max_x_range {
//             return None;
//         }

//         if prime_bound < max_prime_bound {
//             prime_bound = (prime_bound * 2).min(max_prime_bound);
//             state.extend_factor_base_to(prime_bound);
//         }

//         if x_range < max_x_range {
//             x_range = x_range.saturating_mul(2).min(max_x_range);
//             state.extend_x_range_to(x_range)?;
//         }
//     }
// }

// fn try_relations(
//     n: &BigInt,
//     m: &BigInt,
//     factor_base: &[u128],
//     relations: &[Relation],
// ) -> Option<BigInt> {
//     if relations.is_empty() {
//         return None;
//     }

//     let parities = relations
//         .iter()
//         .map(|rel| rel.parity.clone())
//         .collect::<Vec<_>>();

//     let dependencies = find_dependencies_gf2(&parities);

//     for dependency in dependencies {
//         let Some((x, y)) = build_xy_from_dependency(n, m, relations, &dependency, factor_base)
//         else {
//             continue;
//         };

//         if let Some(d) = extract_factor(n, &x, &y) {
//             return Some(d);
//         }
//     }

//     None
// }
