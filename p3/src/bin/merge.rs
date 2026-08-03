use p3::{Bitonic, NUM_LENGTH, bitonic_sorter, generate_bitonic, merge_sort};

fn main() {
    // let input = generate_bitonic(1048576);
    let input = generate_bitonic(NUM_LENGTH);

    let mut merge_out = input.clone();
    let start = std::time::Instant::now();
    merge_sort(&mut merge_out);
    let merge_time = start.elapsed();
    assert!(merge_out.is_sorted_by(|a, b| a >= b));
    println!("merge sort time: {merge_time:?}");
}
