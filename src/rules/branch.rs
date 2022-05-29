use super::*;

pub struct OrRule<Left: Rule, Right: Rule> {
    rng: RuleRng,
    left: RuleBox<Left>,
    right: RuleBox<Right>,
    p: f64,
    p_scatter: f64,
}

impl<Left: Rule, Right: Rule> OrRule<Left, Right> {
    pub fn new(left: Left, right: Right, p: f64, p_scatter: f64) -> Self {
        Self {
            rng: RuleRng::from_entropy(),
            left: RuleBox::new(left),
            right: RuleBox::new(right),
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

impl<Left: Rule, Right: Rule> Clone for OrRule<Left, Right> {
    fn clone(&self) -> Self {
        Self {
            rng: self.rng.clone(),
            left: self.left.clone(),
            right: self.right.clone(),
            p: self.p,
            p_scatter: self.p_scatter,
        }
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

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
        self.left.reseed(seed);
        self.right.reseed(seed);
    }
}
