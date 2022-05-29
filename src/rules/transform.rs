use super::*;

pub struct SpiralRule<R: Rule> {
    rng: RuleRng,
    rule: RuleBox<R>,
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
            rule: RuleBox::new(rule),
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

impl<R: Rule> Clone for SpiralRule<R> {
    fn clone(&self) -> Self {
        Self {
            rng: self.rng.clone(),
            rule: self.rule.clone(),
            delta_low: self.delta_low,
            delta_high: self.delta_high,
            epsilon_low: self.epsilon_low,
            epsilon_high: self.epsilon_high
        }
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

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
        self.rule.reseed(seed);
    }
}

pub struct DiscreteSpiralRule<R: Rule> {
    rule: RuleBox<R>,
    rng: RuleRng,
    distribution: rand_distr::Geometric,
    distribution_scatter: rand_distr::Geometric,
    p: f64,
    p_scatter: f64,
    delta: f64,
    epsilon: f64,
    darken: f64,
}

impl<R: Rule> DiscreteSpiralRule<R> {
    pub fn new(rule: R, (p, p_scatter): (f64, f64), delta: f64, epsilon: f64, darken: f64) -> Result<Self, rand_distr::GeoError> {
        Ok(Self {
            rule: RuleBox::new(rule),
            rng: RuleRng::from_entropy(),
            distribution: rand_distr::Geometric::new(p)?,
            distribution_scatter: rand_distr::Geometric::new(p_scatter)?,
            p,
            p_scatter,
            delta,
            epsilon,
            darken
        })
    }
}

impl<R: Rule> Clone for DiscreteSpiralRule<R> {
    fn clone(&self) -> Self {
        Self {
            rule: self.rule.clone(),
            rng: self.rng.clone(),
            distribution: self.distribution.clone(),
            distribution_scatter: self.distribution_scatter.clone(),
            p: self.p,
            p_scatter: self.p_scatter,
            delta: self.delta,
            epsilon: self.epsilon,
            darken: self.darken,
        }
    }
}

impl<R: Rule> Rule for DiscreteSpiralRule<R> {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize) {
        let (mut next, index) = self.rule.next(previous, history, shape, scatter);

        let num = if scatter {
            self.distribution_scatter.sample(&mut self.rng)
        } else {
            self.distribution.sample(&mut self.rng)
        }.try_into().unwrap_or(i32::MAX);

        if scatter {
            let weight = ((1.0 - self.p) / (1.0 - self.p_scatter)).powi(num) * self.p / self.p_scatter;
            next.mul_weight(weight);
        }

        if num > 0 {
            let delta = self.delta * num as f64;
            let epsilon = self.epsilon.powi(num);
            let darken = self.darken.powi(num);

            let angle = next.y.atan2(next.x);
            let radius = (next.x * next.x + next.y * next.y).sqrt();

            next.x = (angle + delta).cos() * radius * epsilon;
            next.y = (angle + delta).sin() * radius * epsilon;

            next.r *= darken;
            next.g *= darken;
            next.b *= darken;
        }

        (next, index)
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.rng.reseed(seed);
        self.rule.reseed(seed);
    }
}
