use std::mem::size_of;

use crate::Matrix;

pub struct MatrixClassic {
    height: usize,
    width: usize,
    data: Vec<f32>,
}

impl MatrixClassic {
    fn new(height: usize, width: usize, data: Vec<f32>) -> Self {
        Self {
            height,
            width,
            data,
        }
    }
}

impl Matrix for MatrixClassic {
    fn zero(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            data: (0..(height * width)).map(|_| 0.0).collect(),
        }
    }

    fn sequential(height: usize, width: usize) -> Self {
        Self {
            height,
            width,
            data: (0..(height * width)).map(|v| (v % 10) as f32).collect(),
        }
    }

    fn add(&self, other: &Self) -> Self {
        assert_eq!(self.height, other.height);
        assert_eq!(self.width, other.width);

        Self::new(
            self.height,
            self.width,
            self.data
                .iter()
                .zip(other.data.iter())
                .map(|val| val.0 + val.1)
                .collect(),
        )
    }

    fn matmul(&self, other: &Self) -> Self {
        assert_eq!(self.width, other.height);

        let mut result: MatrixClassic = MatrixClassic::zero(self.height, other.width);
        for new_y in 0..self.height {
            for new_x in 0..other.width {
                let mut local_result: f32 = 0.0;
                for i in 0..self.width {
                    local_result += self.get(new_y, i) * other.get(i, new_x);
                }

                result.set(new_y, new_x, local_result);
            }
        }

        result
    }

    fn get(&self, y: usize, x: usize) -> f32 {
        self.data[y * self.width + x]
    }

    fn set(&mut self, y: usize, x: usize, val: f32) {
        self.data[y * self.width + x] = val;
    }

    fn get_memory_size(&self) -> usize {
        return size_of::<Self>() + self.data.capacity() * size_of::<f32>();
    }
}
