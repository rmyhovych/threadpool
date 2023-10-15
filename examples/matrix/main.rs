use std::time::Instant;

use classic::MatrixClassic;
use grouped::MatrixGrouped;

mod classic;
mod grouped;

trait Matrix {
    fn zero(height: usize, width: usize) -> Self;

    fn sequential(height: usize, width: usize) -> Self;

    fn add(&self, other: &Self) -> Self;

    fn matmul(&self, other: &Self) -> Self;

    fn get(&self, y: usize, x: usize) -> f32;

    fn set(&mut self, y: usize, x: usize, val: f32);

    fn get_memory_size(&self) -> usize;
}

type MatrixType = MatrixGrouped;

const MATRIX_SIZE: usize = 1 << 12;

fn main() {
    let matrix0 = MatrixType::sequential(MATRIX_SIZE, MATRIX_SIZE);
    let mut res = MatrixType::sequential(MATRIX_SIZE, 32);

    let instant_start = Instant::now();
    for _ in 0..100 {
        res = matrix0.matmul(&res);
    }
    let duration = instant_start.elapsed();
    println!("Duration[{:.2?}] [{:.2}]", duration, res.get(12, 1));
}
