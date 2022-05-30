use super::*;

mod crate_macro {
    macro_rules! simple_choice {
        ($name:tt) => {
            #[derive(Clone)]
            pub struct $name {
                rng: RuleRng,
            }

            impl $name {
                pub fn new() -> Self {
                    Self {
                        rng: RuleRng::from_entropy(),
                    }
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        rng: RuleRng::from_entropy(),
                    }
                }
            }
        };

        ($name:tt, $param:tt : $type:tt = $default:tt) => {
            #[derive(Clone)]
            pub struct $name {
                rng: RuleRng,
                $param: $type,
            }

            impl $name {
                pub fn new($param: $type) -> Self {
                    Self {
                        rng: RuleRng::from_entropy(),
                        $param,
                    }
                }
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        rng: RuleRng::from_entropy(),
                        $param: $default,
                    }
                }
            }
        };
    }

    pub(crate) use simple_choice;
}

crate_macro::simple_choice!(DefaultChoice);

impl Choice for DefaultChoice {
    fn choose_point(&mut self, _history: &[usize], shape: &Shape) -> usize {
        self.rng.gen_range(0..shape.len())
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
    }
}

crate_macro::simple_choice!(AvoidChoice, diff: isize = 0);

impl Choice for AvoidChoice {
    #[inline]
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize {
        let diff = self.diff.rem_euclid(shape.len() as isize) as usize;

        let mut inc = self.rng.gen_range(0..shape.len() - 1);
        if inc >= diff {
            inc += 1;
        }

        (history[0] + inc) % shape.len()
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
    }
}

#[derive(Clone)]
pub struct AvoidTwoChoice {
    rng: RuleRng,
    diff: isize,
    diff2: isize,
}

impl AvoidTwoChoice {
    pub fn new(diff: isize, diff2: isize) -> Self {
        Self {
            rng: RuleRng::from_entropy(),
            diff,
            diff2,
        }
    }
}

impl Default for AvoidTwoChoice {
    fn default() -> Self {
        Self {
            rng: RuleRng::from_entropy(),
            diff: 0,
            diff2: 0,
        }
    }
}

impl Choice for AvoidTwoChoice {
    #[inline]
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize {
        let len = shape.len();
        let diff = self.diff.rem_euclid(len as isize) as usize;
        let diff2 = self.diff2.rem_euclid(len as isize) as usize;

        let current = history[0];
        let last = history.get(1).copied().unwrap_or(current);

        let inc = loop {
            let mut inc = self.rng.gen_range(0..len - 1);
            if inc >= diff {
                inc += 1;
            }
            if (current + inc) % len != (last + diff2) % len {
                break inc;
            }
        };

        let res = (current + inc) % len;
        res
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
    }
}

crate_macro::simple_choice!(NeighborChoice, dist: usize = 1);

impl Choice for NeighborChoice {
    #[inline]
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize {
        if self.rng.gen() {
            (history[0] + self.dist) % shape.len()
        } else {
            (history[0] + shape.len() - self.dist) % shape.len()
        }
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
    }
}

crate_macro::simple_choice!(NeighborhoodChoice, max_dist: usize = 1);

impl Choice for NeighborhoodChoice {
    #[inline]
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize {
        let choice = self.rng.gen_range(-(self.max_dist as isize)..=(self.max_dist as isize));
        (history[0] as isize + choice).rem_euclid(shape.len() as isize) as usize
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
    }
}

/// One of the most generic choice functions: `∀ (j, k), ℙ(Xᵢ₊₁ = k | Xᵢ = j) = Mⱼₖ`
#[derive(Clone, Debug)]
pub struct MatrixChoice {
    rng: RuleRng,
    n_points: usize,
    matrix: Vec<f64>,
}

impl MatrixChoice {
    pub fn new(n_points: usize, mut matrix: Vec<f64>) -> Option<Self> {
        if matrix.len() != n_points * n_points && matrix.len() != n_points {
            return None
        }

        for y in 0..(matrix.len() / n_points) {
            let mut sum = 0.0;
            for x in 0..n_points {
                sum += matrix[x + y * n_points];
                matrix[x + y * n_points] = sum;
            }
        }

        let matrix = if matrix.len() == n_points {
            let mut new_matrix = Vec::with_capacity(n_points * n_points);
            for _y in 0..n_points {
                new_matrix.extend_from_slice(&matrix);
            }
            new_matrix
        } else {
            matrix
        };

        Some(Self {
            rng: RuleRng::from_entropy(),
            n_points,
            matrix
        })
    }
}

impl Choice for MatrixChoice {
    #[inline]
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize {
        let last = history[0];
        let max = self.matrix[(last + 1) * self.n_points - 1];
        if max == 0.0 {
            return last;
        }

        let num = self.rng.gen_range(0.0..max);

        // TODO: binary search
        for x in 0..self.n_points.min(shape.len()) {
            if self.matrix[x + last * self.n_points] >= num {
                return x;
            }
        }

        return last;
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
    }
}
