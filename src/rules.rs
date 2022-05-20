use super::shape::*;
use rand::rngs::ThreadRng;
use rand::Rng;

pub trait Rule {
    fn next(&mut self, previous: (Point, usize), shape: &Shape) -> (Point, usize);
}

pub trait Choice {
    fn choose_point(&mut self, previous: usize, shape: &Shape) -> usize;
}

// === Rules ===

pub struct DefaultRule<C: Choice = DefaultChoice> {
    choice: C,
    move_ratio: f64,
    color_ratio: f64,
}

impl<C: Choice> DefaultRule<C> {
    pub fn new(choice: C, move_ratio: f64, color_ratio: f64) -> Self {
        Self {
            choice,
            move_ratio,
            color_ratio,
        }
    }
}

impl Default for DefaultRule<DefaultChoice> {
    fn default() -> Self {
        Self {
            choice: DefaultChoice::default(),
            move_ratio: 0.5,
            color_ratio: 1.0,
        }
    }
}

impl<C: Choice> Rule for DefaultRule<C> {
    fn next(&mut self, (previous, index): (Point, usize), shape: &Shape) -> (Point, usize) {
        let index = self.choice.choose_point(index, shape);
        let point = shape[index];
        let dx = point.x - previous.x;
        let dy = point.y - previous.y;

        let dr = point.r - previous.r;
        let dg = point.g - previous.g;
        let db = point.b - previous.b;

        (
            Point::new(
                previous.x + dx * self.move_ratio,
                previous.y + dy * self.move_ratio,
                // point.color()
                (
                    previous.r + dr * self.color_ratio,
                    previous.g + dg * self.color_ratio,
                    previous.b + db * self.color_ratio,
                ),
            ),
            index,
        )
    }
}

pub struct SpiralRule<R: Rule, S: Rng> {
    rng: S,
    rule: R,
    delta_low: f64,
    delta_high: f64,
    epsilon_low: f64,
    epsilon_high: f64,
}

impl<R: Rule, S: Rng> SpiralRule<R, S> {
    pub fn new(
        rng: S,
        rule: R,
        (delta_low, delta_high): (f64, f64),
        (epsilon_low, epsilon_high): (f64, f64),
    ) -> Self {
        Self {
            rule,
            rng,
            delta_low,
            delta_high,
            epsilon_low,
            epsilon_high,
        }
    }

    pub fn inner(&self) -> &R {
        &self.rule
    }
}

impl<R: Rule, S: Rng> Rule for SpiralRule<R, S> {
    fn next(&mut self, previous: (Point, usize), shape: &Shape) -> (Point, usize) {
        let (mut next, index) = self.rule.next(previous, shape);

        let amount: f64 = self.rng.gen();
        // Cov(δ, ε) = 0.0
        let delta = self.delta_low + (self.delta_high - self.delta_low) * amount;
        let epsilon = self.epsilon_low + (self.epsilon_high - self.epsilon_low) * amount;

        let angle = next.y.atan2(next.x);
        let radius = (next.x * next.x + next.y * next.y).sqrt();

        next.x = (angle + delta).cos() * radius * epsilon;
        next.y = (angle + delta).sin() * radius * epsilon;

        (next, index)
    }
}

pub struct OrRule<Left: Rule, Right: Rule, S: Rng> {
    rng: S,
    left: Left,
    right: Right,
    p: f64,
}

impl<Left: Rule, Right: Rule, S: Rng> OrRule<Left, Right, S> {
    pub fn new(rng: S, left: Left, right: Right, p: f64) -> Self {
        Self {
            rng,
            left,
            right,
            p,
        }
    }

    pub fn left(&self) -> &Left {
        &self.left
    }

    pub fn right(&self) -> &Right {
        &self.right
    }
}

impl<Left: Rule, Right: Rule, S: Rng> Rule for OrRule<Left, Right, S> {
    fn next(&mut self, previous: (Point, usize), shape: &Shape) -> (Point, usize) {
        if self.rng.gen_range((0.0)..(1.0)) < self.p {
            self.left.next(previous, shape)
        } else {
            self.right.next(previous, shape)
        }
    }
}

// === Choices ===

macro_rules! simple_choice {
    ($name:tt) => {
        #[derive(Clone)]
        pub struct $name<R: Rng = ThreadRng> {
            rng: R,
        }

        impl<R: Rng> $name<R> {
            pub fn new(rng: R) -> Self {
                Self { rng }
            }
        }

        impl Default for $name<ThreadRng> {
            fn default() -> Self {
                Self {
                    rng: rand::thread_rng(),
                }
            }
        }
    };

    ($name:tt, $param:tt : $type:tt = $default:tt) => {
        #[derive(Clone)]
        pub struct $name<R: Rng = ThreadRng> {
            rng: R,
            $param: $type,
        }

        impl<R: Rng> $name<R> {
            pub fn new(rng: R, $param: $type) -> Self {
                Self { rng, $param }
            }
        }

        impl Default for $name<ThreadRng> {
            fn default() -> Self {
                Self {
                    rng: rand::thread_rng(),
                    $param: $default,
                }
            }
        }
    };
}

simple_choice!(DefaultChoice);

impl<R: Rng> Choice for DefaultChoice<R> {
    fn choose_point(&mut self, _previous: usize, shape: &Shape) -> usize {
        self.rng.gen_range(0..shape.len())
    }
}

simple_choice!(AvoidChoice, diff: isize = 0);

impl<R: Rng> Choice for AvoidChoice<R> {
    #[inline]
    fn choose_point(&mut self, previous: usize, shape: &Shape) -> usize {
        let diff = self.diff.rem_euclid(shape.len() as isize) as usize;

        let mut inc = self.rng.gen_range(0..shape.len() - 1);
        if inc >= diff {
            inc += 1;
        }

        (previous + inc) % shape.len()
    }
}

#[derive(Clone)]
pub struct AvoidTwoChoice<R: Rng = ThreadRng> {
    rng: R,
    diff: isize,
    diff2: isize,
    last: usize,
}

impl<R: Rng> AvoidTwoChoice<R> {
    pub fn new(rng: R, diff: isize, diff2: isize) -> Self {
        Self {
            rng,
            diff,
            diff2,
            last: 0,
        }
    }
}

impl Default for AvoidTwoChoice<ThreadRng> {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
            diff: 0,
            diff2: 0,
            last: 0,
        }
    }
}

impl<R: Rng> Choice for AvoidTwoChoice<R> {
    #[inline]
    fn choose_point(&mut self, previous: usize, shape: &Shape) -> usize {
        let len = shape.len();
        let diff = self.diff.rem_euclid(len as isize) as usize;
        let diff2 = self.diff2.rem_euclid(len as isize) as usize;

        let inc = loop {
            let mut inc = self.rng.gen_range(0..len - 1);
            if inc >= diff {
                inc += 1;
            }
            if (previous + inc) % len != (self.last + diff2) % len {
                break inc;
            }
        };

        let res = (previous + inc) % len;
        self.last = previous;
        res
    }
}
