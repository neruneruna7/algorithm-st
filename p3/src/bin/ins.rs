use p3::{Bitonic, NUM_LENGTH, bitonic_sorter, generate_bitonic, insertion_sort};

fn main() {
    let input = generate_bitonic(NUM_LENGTH);

    let mut insertion_out = input.clone();
    let start = std::time::Instant::now();
    insertion_sort(&mut insertion_out);
    let insertion_time = start.elapsed();
    assert!(insertion_out.is_sorted_by(|a, b| a >= b));
    println!("insertion sort time: {insertion_time:?}");
}
