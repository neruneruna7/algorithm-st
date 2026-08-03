use p3::{Bitonic, NUM_LENGTH, generate_bitonic};
use rayon::prelude::*;

fn main() {
    let input = generate_bitonic(NUM_LENGTH);

    rayon::ThreadPoolBuilder::new().build_global().unwrap();
    let start = std::time::Instant::now();
    let bitonic_out = bitonic_sorter(Bitonic::new(input.clone()).unwrap());
    let bitonic_time = start.elapsed();
    assert!(bitonic_out.0.is_sorted_by(|a, b| a >= b));
    println!("bitonic time: {bitonic_time:?}");
}

fn bitonic_sorter(mut input: Bitonic) -> Bitonic {
    let mut block_size = input.0.len();
    let comparator_count = block_size / 2;

    let num_threads = rayon::current_num_threads();
    let par_chunk_size = comparator_count.div_ceil(num_threads);

    // let mut c = 0;

    while block_size >= 2 {
        let mut comparators = input
            .0
            // 1段進むごとに，前段の2倍の個数で分割
            .chunks_mut(block_size)
            // それぞれを半分で分割し，コンパレータの処理単位で分ける
            .map(|input| {
                let half_length = input.len() / 2;
                let (left, right) = input.split_at_mut(half_length);
                left.iter_mut().zip(right.iter_mut())
            })
            // フラットにして扱いやすく
            .flatten()
            .collect::<Vec<_>>();

        comparators
            .par_chunks_mut(par_chunk_size)
            .for_each(|chunk| {
                for (x, y) in chunk {
                    (**x, **y) = (**x || **y, **x && **y);
                }
            });

        block_size /= 2;
        // c += 1;
    }
    // println!("{c}");

    input
}
