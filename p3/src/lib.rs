use std::fmt::Display;

use rayon::prelude::*;

// pub const NUM_LENGTH: usize = 1048576;
pub const NUM_LENGTH: usize = 33554432;
// pub const NUM_LENGTH: usize = 1073741824;

pub fn generate_bitonic(len: usize) -> Vec<bool> {
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
pub fn insertion_sort(input: &mut [bool]) {
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
pub fn merge_sort(input: &mut [bool]) {
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
pub struct Bitonic(pub Vec<bool>);

impl Display for Bitonic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // trueを1, falseを0に変換する．
        let transformed: Vec<u8> = self.0.iter().map(|&b| if b { 1 } else { 0 }).collect();
        write!(f, "{:?}", transformed)
    }
}

impl Bitonic {
    pub fn new(input: Vec<bool>) -> Result<Self, String> {
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

#[inline(always)]
fn comparator_mut(x: &mut bool, y: &mut bool) {
    (*x, *y) = (*x || *y, *x && *y);
}

#[inline(always)]
fn clean_task_raw(task: &mut [bool], block_size: usize, half_length: usize) {
    debug_assert!(task.len().is_multiple_of(block_size));

    let ptr = task.as_mut_ptr();
    let mut block_start = 0;

    while block_start < task.len() {
        // SAFETY: block_start is a block boundary, and task is a multiple of
        // block_size. Each block's left and right halves are disjoint and in bounds.
        unsafe {
            let mut left = ptr.add(block_start);
            let mut right = left.add(half_length);

            for _ in 0..half_length {
                let x = *left;
                let y = *right;

                *left = x || y;
                *right = x && y;

                left = left.add(1);
                right = right.add(1);
            }
        }

        block_start += block_size;
    }
}

pub fn bitonic_sorter1(mut input: Bitonic) -> Bitonic {
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

pub fn bitonic_sorter_single2(mut input: Bitonic) -> Bitonic {
    let len = input.0.len();
    if len < 2 {
        return input;
    }

    // 最終段付近を一つのタスクに融合し、段ごとの同期・タスク起動を減らす。
    // タスク数はスレッド数の4倍を目標にする。ブロックサイズは2の冪に保つ。
    let target_task_count = rayon::current_num_threads().saturating_mul(4);
    let task_size = (len / target_task_count.max(1)).max(1);
    let next_power = task_size.next_power_of_two();
    let local_sort_size = if next_power > task_size {
        next_power / 2
    } else {
        next_power
    };

    let mut block_size = len;
    while block_size > local_sort_size {
        let half_length = block_size / 2;
        input
            .0
            .par_chunks_mut(block_size)
            .for_each(|task| clean_task_raw(task, block_size, half_length));
        block_size /= 2;
    }

    input.0.par_chunks_mut(local_sort_size).for_each(|task| {
        let mut block_size = local_sort_size;
        while block_size >= 2 {
            clean_task_raw(task, block_size, block_size / 2);
            block_size /= 2;
        }
    });

    input
}

pub fn bitonic_sorter(mut input: Bitonic) -> Bitonic {
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
            let output = bitonic_sorter_single2(Bitonic::new(input).unwrap()).0;

            assert!(output.is_sorted_by(|a, b| a >= b));
            assert_eq!(output.iter().filter(|&&value| value).count(), true_count);
        }
    }

    #[test]
    fn test_bitonic_sorter_for_all_small_bitonic_inputs() {
        for exponent in 0..=4 {
            let len = 1 << exponent;
            for mask in 0..(1usize << len) {
                let input: Vec<bool> = (0..len).map(|index| mask & (1 << index) != 0).collect();
                if !is_bitonic(&input) {
                    continue;
                }

                let mut expected = input.clone();
                expected.sort_by(|left, right| right.cmp(left));
                let output = bitonic_sorter_single2(Bitonic::new(input).unwrap()).0;
                assert_eq!(output, expected);
            }
        }
    }

    fn is_bitonic(input: &[bool]) -> bool {
        let mut decreasing = false;
        for pair in input.windows(2) {
            if !pair[0] && pair[1] {
                if decreasing {
                    return false;
                }
            } else if pair[0] && !pair[1] {
                decreasing = true;
            }
        }
        true
    }
}
