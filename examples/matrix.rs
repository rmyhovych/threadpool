use std::{
    fmt::Display,
    ops::{Index, IndexMut},
    sync::Arc,
    time::Instant,
};

use rand::{distributions::Normal, Rng};

use threadpool::ThreadPool;

extern crate threadpool;

struct MatrixChunk {
    data: Matrix,
    x: usize,
    y: usize,
}

#[derive(Clone)]
struct Matrix {
    height: usize,
    width: usize,
    data: Vec<f64>,
}

impl Matrix {
    fn zeros(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            data: (0..(height * width)).map(|_| 0.0).collect(),
        }
    }

    fn randn(height: usize, width: usize, mean: f64, std_dev: f64) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            height,
            width,
            data: rng
                .sample_iter(&Normal::new(mean, std_dev))
                .take(height * width)
                .collect(),
        }
    }

    fn mm(m0: Matrix, m1: Matrix) -> Matrix {
        assert_eq!(m0.width, m1.height);

        let mut result = Matrix::zeros(m0.height, m1.width);
        for y in 0..m0.height {
            for x in 0..m1.width {
                let mut val: f64 = 0.0;
                for i in 0..m0.width {
                    val += m0[y][i] * m1[i][x];
                }
                result[y][x] = val;
            }
        }

        result
    }

    fn parallel_mm(tp: &mut ThreadPool<MatrixChunk>, m0: Matrix, m1: Matrix) -> Matrix {
        assert_eq!(m0.width, m1.height);

        let min_chunk_size: usize = 64;

        let chunk_height = Self::get_chunk_size(m0.height, min_chunk_size);
        let chunk_width = Self::get_chunk_size(m1.width, min_chunk_size);
        if chunk_width == m0.height && chunk_height == m1.width {
            Self::mm(m0, m1)
        } else {
            let m0_shared = Arc::new(m0);
            let m1_shared = Arc::new(m1);

            let mut result = Self::zeros(m0_shared.height, m1_shared.width);
            for y in (0..result.height).step_by(chunk_height) {
                for x in (0..result.width).step_by(chunk_width) {
                    let height = result.height.min(y + chunk_height) - y;
                    let width = result.width.min(x + chunk_width) - x;

                    let m0_local = m0_shared.clone();
                    let m1_local = m1_shared.clone();

                    tp.run(move || {
                        let mut chunk = MatrixChunk {
                            data: Matrix::zeros(height, width),
                            x,
                            y,
                        };

                        for j in y..(y + height) {
                            for i in x..(x + width) {
                                let mut val = 0.0;
                                for k in 0..m0_local.height {
                                    val += m0_local[j][k] * m1_local[k][i];
                                }
                                chunk.data[j - y][i - x] = val;
                            }
                        }

                        chunk
                    });
                }
            }

            let chunks = tp.collect_results();
            for chunk in chunks {
                for j in 0..chunk.data.height {
                    for i in 0..chunk.data.width {
                        result[j + chunk.y][i + chunk.x] = chunk.data[j][i];
                    }
                }
            }

            result
        }
    }

    fn get_chunk_size(start_size: usize, min_size: usize) -> usize {
        let mut multiplier = 1;
        while start_size / multiplier > min_size {
            multiplier += 1;
        }

        start_size / multiplier
    }
}

impl Index<usize> for Matrix {
    type Output = [f64];

    fn index<'a>(&'a self, i: usize) -> &'a [f64] {
        let start = i * self.width;
        &self.data[start..start + self.width]
    }
}

impl IndexMut<usize> for Matrix {
    fn index_mut<'a>(&'a mut self, i: usize) -> &'a mut [f64] {
        let start = i * self.width;
        &mut self.data[start..start + self.width]
    }
}

impl Display for Matrix {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        for y in 0..self.height {
            for x in 0..self.width {
                if let Err(e) = fmt.write_str(format!("| {:.1$}\t", self[y][x], 2).as_str()) {
                    return Err(e);
                }
            }
            if let Err(e) = fmt.write_str("|\n") {
                return Err(e);
            }
        }
        Ok(())
    }
}

fn main() {
    let mut tp = ThreadPool::<MatrixChunk>::new(8);

    let m0 = Matrix::randn(1024, 1024, 0.0, 1.0);
    let m1 = Matrix::randn(1024, 1024, 0.0, 1.0);

    let now = Instant::now();
    Matrix::mm(m0.clone(), m1.clone());
    println!("Single-thread: Elapsed {:.2}s", now.elapsed().as_millis() as f32 / 1000.0);

    let now = Instant::now();
    Matrix::parallel_mm(&mut tp, m0.clone(), m1.clone());
    println!("Multi-thread: Elapsed {:.2}s", now.elapsed().as_millis() as f32 / 1000.0);
}
