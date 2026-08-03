use p3::{Bitonic, NUM_LENGTH, bitonic_sorter, bitonic_sorter1, generate_bitonic};

fn main() {
    let input = generate_bitonic(NUM_LENGTH);

    rayon::ThreadPoolBuilder::new().build_global().unwrap();
    let start = std::time::Instant::now();
    let bitonic_out = bitonic_sorter1(Bitonic::new(input.clone()).unwrap());
    let bitonic_time = start.elapsed();
    assert!(bitonic_out.0.is_sorted_by(|a, b| a >= b));
    println!("bitonic time: {bitonic_time:?}");
}
