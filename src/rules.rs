use super::shape::*;
// use rand::rngs::RuleRng;
use rand::{Rng, SeedableRng};

pub mod tensor;
pub use tensor::*;

type RuleRng = rand_xoshiro::Xoshiro256Plus;

pub trait Rule: Clone + Send {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize);
}

pub trait Choice: Clone + Send {
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize;
}

// === Rules ===

#[derive(Clone)]
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
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        _scatter: bool,
    ) -> (Point, usize) {
        let index = self.choice.choose_point(history, shape);
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

#[derive(Clone)]
pub struct DarkenRule<R: Rule> {
    rule: R,
    amount: f64,
}

impl<R: Rule> DarkenRule<R> {
    pub fn new(rule: R, amount: f64) -> Self {
        Self { rule, amount }
    }
}

impl<R: Rule> Rule for DarkenRule<R> {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize) {
        let (mut next, index) = self.rule.next(previous, history, shape, scatter);

        next.r *= self.amount;
        next.g *= self.amount;
        next.b *= self.amount;

        (next, index)
    }
}

#[derive(Clone)]
pub struct SpiralRule<R: Rule> {
    rng: RuleRng,
    rule: R,
    delta_low: f64,
    delta_high: f64,
    epsilon_low: f64,
    epsilon_high: f64,
}

impl<R: Rule> SpiralRule<R> {
    pub fn new(
        rule: R,
        (delta_low, delta_high): (f64, f64),
        (epsilon_low, epsilon_high): (f64, f64),
    ) -> Self {
        Self {
            rule,
            rng: RuleRng::from_entropy(),
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

impl<R: Rule> Rule for SpiralRule<R> {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize) {
        let (mut next, index) = self.rule.next(previous, history, shape, scatter);

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

#[derive(Clone)]
pub struct OrRule<Left: Rule, Right: Rule> {
    rng: RuleRng,
    left: Left,
    right: Right,
    p: f64,
    p_scatter: f64,
}

impl<Left: Rule, Right: Rule> OrRule<Left, Right> {
    pub fn new(left: Left, right: Right, p: f64, p_scatter: f64) -> Self {
        Self {
            rng: RuleRng::from_entropy(),
            left,
            right,
            p,
            p_scatter,
        }
    }

    pub fn left(&self) -> &Left {
        &self.left
    }

    pub fn right(&self) -> &Right {
        &self.right
    }
}

impl<Left: Rule, Right: Rule> Rule for OrRule<Left, Right> {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize) {
        let p = if scatter { self.p_scatter } else { self.p };
        let (mut res, prob) = if self.rng.gen_range((0.0)..(1.0)) < p {
            (
                self.left.next(previous, history, shape, scatter),
                self.p / p,
            )
        } else {
            (
                self.right.next(previous, history, shape, scatter),
                (1.0 - self.p) / (1.0 - p),
            )
        };

        if scatter {
            res.0.mul_weight(prob);
        }
        res
    }
}

// === Choices ===

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
}
