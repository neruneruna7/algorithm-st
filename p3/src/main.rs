use std::fmt::Display;

use rayon::iter::{
    IndexedParallelIterator as _, IntoParallelIterator, IntoParallelRefIterator as _,
    ParallelIterator as _,
};

fn main() {
    println!("Hello, world!");
    let input = (true, false, true, false);
    let output = sorting_network_4(input);
    println!("{:?}", output);

    // let input_bitonic = None;
    let harf_cleaner_input =
        Bitonic::new(vec![false, false, false, true, true, true, false, false]).unwrap();
    let half_cleaner_output = half_cleaner(harf_cleaner_input);
    println!("{}", half_cleaner_output);

    let input_bitonic =
        Bitonic::new(vec![false, false, false, true, true, true, false, false]).unwrap();
    let bitonic_out = bitonic_sorter(input_bitonic);
    println!("{}", bitonic_out);
}

/// 左を上，右を下とみなす．
fn comparator(x: bool, y: bool) -> (bool, bool) {
    if x > y { (x, y) } else { (y, x) }
}

fn sorting_network_4(input: (bool, bool, bool, bool)) -> (bool, bool, bool, bool) {
    let (step1_1, step1_2) = comparator(input.0, input.1);
    let (step1_3, step1_4) = comparator(input.2, input.3);

    let (step2_1, step2_2) = comparator(step1_1, step1_3);
    let (step2_3, step2_4) = comparator(step1_2, step1_4);

    let (step3_2, step3_3) = comparator(step2_2, step2_3);
    let (step3_1, step3_4) = (step2_1, step2_4);

    (step3_1, step3_2, step3_3, step3_4)
}

#[derive(Debug, Clone)]
struct Bitonic(Vec<bool>);

impl Display for Bitonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // trueを1, falseを0に変換する．
        let transformed: Vec<u8> = self.0.iter().map(|&b| if b { 1 } else { 0 }).collect();
        write!(f, "{:?}", transformed)
    }
}

impl Bitonic {
    fn new(input: Vec<bool>) -> Result<Self, String> {
        // 要素数が2^nでない場合，エラー
        if input.len().count_ones() != 1 {
            return Err("要素数が2^nでない".to_string());
        }
        Ok(Self(input))
    }
}

fn half_cleaner(input: Bitonic) -> Bitonic {
    let length = input.0.len();
    let half_length = length / 2;
    let (left, right) = input.0.split_at(half_length);

    let (left_out, right_out): (Vec<bool>, Vec<bool>) = left
        .par_iter()
        .zip(right.par_iter())
        .map(|(&x, &y)| comparator(x, y))
        .unzip();
    let bitonic_out = left_out.into_iter().chain(right_out).collect();
    // 入力と同じであることが保証されているので，再度バリデーションは不要
    Bitonic(bitonic_out)
}

fn bitonic_sorter(input: Bitonic) -> Bitonic {
    // println!("process: {}", input);
    let half_length = input.0.len() / 2;
    if half_length < 1 {
        return input;
    }
    let cleaned = half_cleaner(input);
    // println!("cleaned: {}", cleaned);

    let (left, right) = cleaned.0.split_at(half_length);
    let left = Bitonic(left.to_vec());
    let right = Bitonic(right.to_vec());
    // println!("left: {}, right: {}", left, right);
    let out = rayon::join(|| bitonic_sorter(left), || bitonic_sorter(right));

    // println!("join: {} {}", out.0, out.1);

    let bitonic_out = out.0.0.into_iter().chain(out.1.0).collect();
    let bitonic_out = Bitonic(bitonic_out);

    bitonic_out
}
