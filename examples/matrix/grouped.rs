use std::{
    mem::size_of,
    ops,
    sync::atomic::{self, AtomicUsize},
    thread::{self},
};

use crate::Matrix;

const GROUP_WIDTH: usize = 16;
const GROUP_SIZE: usize = GROUP_WIDTH * GROUP_WIDTH;

const THREAD_COUNT: usize = 5;

struct Group {
    data: [f32; GROUP_SIZE],
}

impl Group {
    fn zero() -> Self {
        Self {
            data: [0.0; GROUP_SIZE],
        }
    }

    fn get(&self, y: usize, x: usize) -> f32 {
        self.data[y * GROUP_WIDTH + x]
    }

    fn set(&mut self, y: usize, x: usize, val: f32) {
        self.data[y * GROUP_WIDTH + x] = val;
    }

    fn add(&mut self, y: usize, x: usize, val: f32) {
        self.data[y * GROUP_WIDTH + x] += val;
    }

    fn matmul(&self, other: &Self) -> Self {
        let mut result_group: Group = Self::zero();
        for y in 0..GROUP_WIDTH {
            for x in 0..GROUP_WIDTH {
                for i in 0..GROUP_WIDTH {
                    result_group.add(y, x, self.get(y, i) * other.get(i, x));
                }
            }
        }

        result_group
    }
}

impl Clone for Group {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl ops::AddAssign for Group {
    fn add_assign(&mut self, rhs: Self) {
        for i in 0..GROUP_SIZE {
            self.data[i] += rhs.data[i];
        }
    }
}

pub struct MatrixGrouped {
    size: (usize, usize),
    group_size: (usize, usize),
    groups: Vec<Group>,
}

impl MatrixGrouped {
    fn new(height: usize, width: usize, groups: Vec<Group>) -> Self {
        let group_width = Self::get_group_count(width);
        let group_height = Self::get_group_count(height);
        Self {
            size: (height, width),
            group_size: (group_height, group_width),
            groups,
        }
    }

    fn get_group(&self, y: usize, x: usize) -> &Group {
        &self.groups[y * self.group_size.1 + x]
    }

    fn get_group_mut(&mut self, y: usize, x: usize) -> &mut Group {
        &mut self.groups[y * self.group_size.1 + x]
    }

    fn get_group_count(value_count: usize) -> usize {
        (value_count / GROUP_WIDTH) + if value_count % GROUP_WIDTH > 0 { 1 } else { 0 }
    }

    fn matmul_simple(&self, other: &Self) -> Self {
        let mut result = MatrixGrouped::zero(self.size.0, other.size.1);
        for new_group_y in 0..self.group_size.0 {
            for new_group_x in 0..other.group_size.1 {
                let group_new = result.get_group_mut(new_group_y, new_group_x);
                for i_group in 0..self.group_size.1 {
                    let group_left = self.get_group(new_group_y, i_group);
                    let group_right = other.get_group(i_group, new_group_x);
                    *group_new += group_left.matmul(group_right);
                }
            }
        }

        result
    }

    fn matmul_parallel(&self, other: &Self) -> Self {
        let work_index = AtomicUsize::new(0);
        thread::scope(|scope| {
            let mut groups = (0..THREAD_COUNT)
                .map(|_| {
                    scope.spawn(|| {
                        let mut resulting_groups = Vec::<(usize, Vec<Group>)>::new();
                        loop {
                            let new_group_y = work_index.fetch_add(1, atomic::Ordering::Relaxed);
                            if new_group_y < self.group_size.0 {
                                let new_row_groups = (0..other.group_size.0)
                                    .map(|new_group_x| {
                                        let mut group_new = Group::zero();
                                        for i in 0..self.group_size.1 {
                                            let group_left = self.get_group(new_group_y, i);
                                            let group_right = other.get_group(i, new_group_x);

                                            group_new += group_left.matmul(group_right);
                                        }

                                        group_new
                                    })
                                    .collect::<Vec<_>>();

                                resulting_groups.push((new_group_y, new_row_groups));
                            } else {
                                break;
                            }
                        }

                        resulting_groups
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .flatten()
                .collect::<Vec<(usize, Vec<Group>)>>();

            groups.sort_by(|a, b| a.0.cmp(&b.0));

            MatrixGrouped::new(
                self.size.0,
                other.size.1,
                groups.into_iter().flat_map(|g| g.1).collect(),
            )
        })
    }
}

impl Matrix for MatrixGrouped {
    fn zero(height: usize, width: usize) -> Self {
        let group_height = Self::get_group_count(height);
        let group_width = Self::get_group_count(width);

        Self {
            size: (height, width),
            group_size: (group_height, group_width),
            groups: (0..(group_height * group_width))
                .map(|_| Group::zero())
                .collect(),
        }
    }

    fn sequential(height: usize, width: usize) -> Self {
        let mut matrix = Self::zero(height, width);
        let mut val: u32 = 0;
        for y in 0..matrix.size.0 {
            for x in 0..matrix.size.1 {
                matrix.set(y, x, (val % 10) as f32);
                val += 1
            }
        }

        matrix
    }

    fn add(&self, other: &Self) -> Self {
        assert_eq!(self.size.0, other.size.0);
        assert_eq!(self.size.1, other.size.1);

        Self::new(
            self.size.0,
            self.size.1,
            self.groups
                .iter()
                .zip(other.groups.iter())
                .map(|val| {
                    let mut new_group = Group::zero();
                    for i in 0..GROUP_SIZE {
                        new_group.data[i] = val.0.data[i] + val.1.data[i];
                    }

                    new_group
                })
                .collect(),
        )
    }

    fn matmul(&self, other: &Self) -> Self {
        assert_eq!(self.size.1, other.size.0);

        self.matmul_parallel(other)
    }

    fn get(&self, y: usize, x: usize) -> f32 {
        let group_y = y / GROUP_WIDTH;
        let group_x = x / GROUP_WIDTH;

        self.get_group(group_y, group_x)
            .get(y % GROUP_WIDTH, x % GROUP_WIDTH)
    }

    fn set(&mut self, y: usize, x: usize, val: f32) {
        let group_y = y / GROUP_WIDTH;
        let group_x = x / GROUP_WIDTH;

        self.get_group_mut(group_y, group_x)
            .set(y % GROUP_WIDTH, x % GROUP_WIDTH, val);
    }

    fn get_memory_size(&self) -> usize {
        return size_of::<Self>() + self.groups.capacity() * size_of::<Group>();
    }
}
