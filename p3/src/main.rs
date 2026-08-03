use std::fmt::Display;

use p3::{
    Bitonic, bitonic_sorter, bitonic_sorter_single2, bitonic_sorter1, generate_bitonic, merge_sort,
};
use rayon::prelude::*;

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
    let bitonic_out = bitonic_sorter1(Bitonic::new(input.clone()).unwrap());
    let bitonic_time = start.elapsed();
    assert!(bitonic_out.0.is_sorted_by(|a, b| a >= b));
    println!("bitonic single time: {bitonic_time:?}");

    let start = std::time::Instant::now();
    let bitonic_out = bitonic_sorter_single2(Bitonic::new(input.clone()).unwrap());
    let bitonic_time = start.elapsed();
    assert!(bitonic_out.0.is_sorted_by(|a, b| a >= b));
    println!("bitonic single2 time: {bitonic_time:?}");

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

    let start = std::time::Instant::now();
    let mut merge_out = input.clone();
    merge_sort(&mut merge_out);
    let merge_time = start.elapsed();
    assert!(merge_out.is_sorted_by(|a, b| a >= b));
    println!("merge sort time: {merge_time:?}");
}
