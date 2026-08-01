use std::fmt::Display;

fn main() {
    println!("Hello, world!");
    let input = (true, false, true, false);
    let output = sorting_network_4(input);
    println!("{:?}", output);

    // let input_bitonic = None;
    //
    let harf_cleaner_input = Bitonic(vec![false, false, false, true, true, true, false, false]);
    let half_cleaner_output = half_cleaner(harf_cleaner_input);
    println!("{}", half_cleaner_output);
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
    let mid_point = length / 2;
    let left = input.0[..mid_point].to_vec();
    let right = input.0[mid_point..].to_vec();
    let bitonic_out = left
        .into_iter()
        .zip(right)
        .map(|(x, y)| comparator(x, y))
        .enumerate()
        .fold(vec![false; length], |mut acc, (i, (x, y))| {
            acc[i] = x;
            acc[i + mid_point] = y;
            acc
        });

    Bitonic(bitonic_out)
}
