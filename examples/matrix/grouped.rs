use std::{
    mem::size_of,
    sync::{mpsc, Arc},
    thread::{self, ScopedJoinHandle},
};

use threadpool::atomic::spinlock::SpinLock;

use crate::Matrix;

const GROUP_WIDTH: usize = 16;
const GROUP_SIZE: usize = GROUP_WIDTH * GROUP_WIDTH;

const THREAD_COUNT: usize = 1;

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

                    for y in 0..GROUP_WIDTH {
                        for x in 0..GROUP_WIDTH {
                            for i in 0..GROUP_WIDTH {
                                group_new.add(y, x, group_left.get(y, i) * group_right.get(i, x));
                            }
                        }
                    }
                }
            }
        }

        result
    }

    fn matmul_parallel(&self, other: &Self) -> Self {
        thread::scope(|scope| {
            let work_queue = Arc::new(SpinLock::new({
                let mut work_queue =
                    Vec::<(usize, usize)>::with_capacity(self.group_size.0 * self.group_size.1);
                for new_group_y in 0..self.group_size.0 {
                    for new_group_x in 0..other.group_size.1 {
                        work_queue.push((new_group_y, new_group_x));
                    }
                }

                work_queue
            }));

            let thread_handles = (0..THREAD_COUNT)
                .map(|_| {
                    let local_work_queue = work_queue.clone();
                    {
                        scope.spawn(move || {
                            let mut resulting_groups = Vec::<(usize, usize, Group)>::new();
                            loop {
                                let work = { local_work_queue.lock().pop() };
                                match work {
                                    Some((new_group_y, new_group_x)) => {
                                        resulting_groups.push((
                                            new_group_y,
                                            new_group_x,
                                            Group::zero(),
                                        ));
                                        let group_new = resulting_groups.last_mut().unwrap();
                                        for i_group in 0..self.group_size.1 {
                                            let group_left = self.get_group(new_group_y, i_group);
                                            let group_right = other.get_group(i_group, new_group_x);

                                            for y in 0..GROUP_WIDTH {
                                                for x in 0..GROUP_WIDTH {
                                                    for i in 0..GROUP_WIDTH {
                                                        group_new.2.add(
                                                            y,
                                                            x,
                                                            group_left.get(y, i)
                                                                * group_right.get(i, x),
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    None => break,
                                }
                            }

                            resulting_groups
                        })
                    }
                })
                .collect::<Vec<ScopedJoinHandle<Vec<(usize, usize, Group)>>>>();

            let mut groups = thread_handles
                .into_iter()
                .map(|handle| handle.join().unwrap())
                .flatten()
                .collect::<Vec<(usize, usize, Group)>>();

            groups.sort_by(|a, b| {
                if a.0 == b.0 {
                    a.1.cmp(&b.1)
                } else {
                    a.0.cmp(&b.0)
                }
            });

            MatrixGrouped::new(
                self.size.0,
                other.size.1,
                groups.into_iter().map(|g| g.2).collect(),
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
