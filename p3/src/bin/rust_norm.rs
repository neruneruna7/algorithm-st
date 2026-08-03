use p3::{Bitonic, NUM_LENGTH, bitonic_sorter, generate_bitonic, merge_sort};

fn main() {
    let input = generate_bitonic(NUM_LENGTH);

    let mut normal_out = input.clone();
    let start = std::time::Instant::now();
    normal_out.sort_by(|a, b| b.cmp(a));
    let normal_time = start.elapsed();
    assert!(normal_out.is_sorted_by(|a, b| a >= b));
    println!("normal time: {normal_time:?}");
}
