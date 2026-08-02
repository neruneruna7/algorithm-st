use std::fmt::Display;

use rayon::prelude::*;

const COMPARATOR_CHUNK_SIZE: usize = 1024;

fn main() {
    // let input = generate_bitonic(1048576);
    let input = generate_bitonic(33554432);
    // let input = generate_bitonic(1073741824);

    rayon::ThreadPoolBuilder::new().build_global().unwrap();
    // let start = std::time::Instant::now();
    // let bitonic_out = bitonic_sorter_par(Bitonic::new(input.clone()).unwrap());
    // let bitonic_time = start.elapsed();
    // assert!(bitonic_out.0.is_sorted_by(|a, b| a >= b));
    // println!("bitonic par time: {bitonic_time:?}");

    let start = std::time::Instant::now();
    let bitonic_out = bitonic_sorter(Bitonic::new(input.clone()).unwrap());
    let bitonic_time = start.elapsed();
    assert!(bitonic_out.0.is_sorted_by(|a, b| a >= b));
    println!("bitonic time: {bitonic_time:?}");

    let start = std::time::Instant::now();
    let bitonic_out = bitonic_sorter_single(Bitonic::new(input.clone()).unwrap());
    let bitonic_time = start.elapsed();
    assert!(bitonic_out.0.is_sorted_by(|a, b| a >= b));
    println!("bitonic single time: {bitonic_time:?}");

    // 通常ソート
    let start = std::time::Instant::now();
    let mut normal_out = input.clone();
    normal_out.sort_by(|a, b| b.cmp(a));
    let normal_time = start.elapsed();
    assert!(normal_out.is_sorted_by(|a, b| a >= b));
    println!("normal time: {normal_time:?}");

    // let start = std::time::Instant::now();
    // let mut insertion_out = input.clone();
    // insertion_sort(&mut insertion_out);
    // let insertion_time = start.elapsed();
    // assert!(insertion_out.is_sorted_by(|a, b| a >= b));
    // println!("insertion sort time: {insertion_time:?}");

    // let start = std::time::Instant::now();
    // let mut merge_out = input.clone();
    // merge_sort(&mut merge_out);
    // let merge_time = start.elapsed();
    // assert!(merge_out.is_sorted_by(|a, b| a >= b));
    // println!("merge sort time: {merge_time:?}");
}

fn generate_bitonic(len: usize) -> Vec<bool> {
    if len == 0 {
        return Vec::new();
    }

    let rise = len / 4;
    let fall = len * 3 / 4;

    (0..len).map(|i| i >= rise && i < fall).collect()
}

/// 左を上，右を下とみなす．
fn comparator(x: bool, y: bool) -> (bool, bool) {
    let max = x || y;
    let min = x && y;
    (max, min)
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

/// `true` が先に来る降順で挿入ソートする．
fn insertion_sort(input: &mut [bool]) {
    for index in 1..input.len() {
        let value = input[index];
        let mut position = index;

        while position > 0 && input[position - 1] < value {
            input[position] = input[position - 1];
            position -= 1;
        }

        input[position] = value;
    }
}

/// `true` が先に来る降順でマージソートする．
fn merge_sort(input: &mut [bool]) {
    if input.len() <= 1 {
        return;
    }

    let middle = input.len() / 2;
    merge_sort(&mut input[..middle]);
    merge_sort(&mut input[middle..]);

    let mut merged = Vec::with_capacity(input.len());
    let (mut left, mut right) = (0, middle);

    while left < middle && right < input.len() {
        if input[left] >= input[right] {
            merged.push(input[left]);
            left += 1;
        } else {
            merged.push(input[right]);
            right += 1;
        }
    }

    merged.extend_from_slice(&input[left..middle]);
    merged.extend_from_slice(&input[right..]);
    input.copy_from_slice(&merged);
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

fn half_cleaner_slice(input: &mut [bool]) {
    let half_length = input.len() / 2;
    let (left, right) = input.split_at_mut(half_length);

    for (x, y) in left.iter_mut().zip(right.iter_mut()) {
        comparator_mut(x, y);
    }
}

fn comparator_mut(x: &mut bool, y: &mut bool) {
    (*x, *y) = (*x || *y, *x && *y);
}

fn bitonic_sorter_single(mut input: Bitonic) -> Bitonic {
    let mut block_size = input.0.len();

    println!("block_size: {}", block_size);
    while block_size >= 2 {
        input
            .0
            .par_chunks_mut(block_size)
            .for_each(half_cleaner_slice);

        block_size /= 2;
    }

    input
}

fn bitonic_sorter(mut input: Bitonic) -> Bitonic {
    let len = input.0.len();

    debug_assert!(len.is_power_of_two());

    let mut block_size = len;

    let comparator_count = len / 2;
    println!("comparator count: {}", comparator_count);
    let num_threads = rayon::current_num_threads();
    println!("num threads: {}", num_threads);

    let par_chunk_size = comparator_count.div_ceil(num_threads);
    println!("par_chunk_size: {}", par_chunk_size);

    // Rayon の closure に *mut bool を直接 capture させないため、
    // アドレス値として保持する。
    let ptr = input.0.as_mut_ptr() as usize;

    while block_size >= 2 {
        // let start = std::time::Instant::now();

        let half = block_size / 2;

        let chunk_count = comparator_count.div_ceil(par_chunk_size);

        (0..chunk_count).into_par_iter().for_each(|chunk_index| {
            let ptr = ptr as *mut bool;

            let begin = chunk_index * par_chunk_size;
            let end = (begin + par_chunk_size).min(comparator_count);

            for comparator_index in begin..end {
                // comparator_index は、元コードで
                //
                // chunks_mut(block_size)
                //   .flat_map(|block| left.zip(right))
                //
                // した後の通し番号に相当する。
                let block_index = comparator_index / half;
                let offset = comparator_index % half;

                let x_index = block_index * block_size + offset;

                let y_index = x_index + half;

                unsafe {
                    let x_ptr = ptr.add(x_index);
                    let y_ptr = ptr.add(y_index);

                    let x = *x_ptr;
                    let y = *y_ptr;

                    *x_ptr = x || y;
                    *y_ptr = x && y;
                }
            }
        });

        block_size /= 2;
        // let time = start.elapsed();
        // println!("time: {time:?}");
    }

    input
}
// input.0.par_chunks_mut(block_size).for_each(|input| {
//     let half_length = input.len() / 2;
//     let (left, right) = input.split_at_mut(half_length);

//     for (x, y) in left.iter_mut().zip(right.iter_mut()) {
//         (*x, *y) = comparator(*x, *y);
//     }
// });

// fn bitonic_sorter_single(input: Bitonic) -> Bitonic {
//     // println!("process: {}", input);
//     let half_length = input.0.len() / 2;
//     if half_length < 1 {
//         return input;
//     }
//     let cleaned = half_cleaner(input);
//     // println!("cleaned: {}", cleaned);

//     let (left, right) = cleaned.0.split_at(half_length);
//     let left = Bitonic(left.to_vec());
//     let right = Bitonic(right.to_vec());
//     // println!("left: {}, right: {}", left, right);
//     let out = rayon::join(|| bitonic_sorter(left), || bitonic_sorter(right));

//     // println!("join: {} {}", out.0, out.1);

//     let bitonic_out = out.0.0.into_iter().chain(out.1.0).collect();
//     let bitonic_out = Bitonic(bitonic_out);

//     bitonic_out
// }

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_sorting_network_and_half_cleaner() {
        let input = (true, false, true, false);
        let output = sorting_network_4(input);
        assert_eq!(output, (true, true, false, false));

        let half_cleaner_input =
            Bitonic::new(vec![false, false, false, true, true, true, false, false]).unwrap();
        let half_cleaner_output = half_cleaner(half_cleaner_input);
        assert_eq!(
            half_cleaner_output.0,
            vec![true, true, false, true, false, false, false, false]
        );
    }

    #[test]
    fn test_insertion_and_merge_sort() {
        let input = vec![false, true, false, true, true, false, true];
        let expected = vec![true, true, true, true, false, false, false];

        let mut insertion_out = input.clone();
        insertion_sort(&mut insertion_out);
        assert_eq!(insertion_out, expected);

        let mut merge_out = input;
        merge_sort(&mut merge_out);
        assert_eq!(merge_out, expected);
    }

    #[test]
    fn test_bitonic_sorter() {
        for exponent in 0..=10 {
            let input = generate_bitonic(1 << exponent);
            let true_count = input.iter().filter(|&&value| value).count();
            let output = bitonic_sorter_par(Bitonic::new(input).unwrap()).0;

            assert!(output.is_sorted_by(|a, b| a >= b));
            assert_eq!(output.iter().filter(|&&value| value).count(), true_count);
        }
    }
}
