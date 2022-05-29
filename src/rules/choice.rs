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
