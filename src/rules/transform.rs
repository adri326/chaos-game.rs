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

#[derive(Clone, Debug)]
pub enum RandAdvanceDistr {
    SkewNormal(rand_distr::SkewNormal<f64>),
    Uniform(rand::distributions::Uniform<f64>)
}

impl Distribution<f64> for RandAdvanceDistr {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match self {
            Self::SkewNormal(distr) => distr.sample(rng),
            Self::Uniform(distr) => distr.sample(rng)
        }
    }
}

// TODO: maybe move to another module
#[derive(Debug)]
pub struct RandAdvanceRule<C: Choice> {
    choice: RuleBox<C>,
    rng: RuleRng,
    distribution: RandAdvanceDistr,
    color_ratio: f64,
}

impl<C: Choice> RandAdvanceRule<C> {
    /// Creates a new RandAdvanceRule, with either the SkewNormal or the Uniform distribution:
    /// If omega > 0, SkewNormal(position = zeta, scale = omega, shape = alpha) is used
    /// Otherwise, Uniform(low = zeta, high = alpha) is used
    pub fn new(choice: C, zeta: f64, omega: f64, alpha: f64, color_ratio: f64) -> Self {
        Self {
            choice: RuleBox::new(choice),
            rng: RuleRng::from_entropy(),
            distribution: if omega > 0.0 {
                RandAdvanceDistr::SkewNormal(rand_distr::SkewNormal::new(zeta, omega, alpha).unwrap())
            } else {
                RandAdvanceDistr::Uniform(rand::distributions::Uniform::new(zeta, alpha))
            },
            color_ratio
        }
    }
}

impl<C: Choice> Clone for RandAdvanceRule<C> {
    fn clone(&self) -> Self {
        Self {
            choice: self.choice.clone(),
            rng: self.rng.clone(),
            distribution: self.distribution.clone(),
            color_ratio: self.color_ratio
        }
    }
}

impl<C: Choice> Rule for RandAdvanceRule<C> {
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

        let move_ratio = self.rng.sample(&self.distribution);

        (
            Point::new(
                previous.x + dx * move_ratio,
                previous.y + dy * move_ratio,
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

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.choice.reseed(seed);
        self.rng.reseed(seed);
    }
}

#[derive(Debug)]
pub struct AdvanceTwoRule<C1: Choice, C2: Choice> {
    choices: (RuleBox<C1>, RuleBox<C2>),
    move_ratio: (f64, f64),
    color_ratio: f64,
    skew: f64,
}

impl<C1: Choice, C2: Choice> AdvanceTwoRule<C1, C2> {
    pub fn new(choice1: C1, choice2: C2, move_ratio1: f64, move_ratio2: f64, color_ratio: f64, skew: f64) -> Self {
        Self {
            choices: (RuleBox::new(choice1), RuleBox::new(choice2)),
            move_ratio: (move_ratio1, move_ratio2),
            color_ratio,
            skew
        }
    }

    #[inline]
    fn advance(&self, previous: Point, next: Point, amount: f64) -> Point {
        let dx = next.x - previous.x;
        let dy = next.y - previous.y;

        let dr = next.r - previous.r;
        let dg = next.g - previous.g;
        let db = next.b - previous.b;

        Point::new(
            previous.x + dx * amount,
            previous.y + dy * amount,
            (
                previous.r + dr * self.color_ratio,
                previous.g + dg * self.color_ratio,
                previous.b + db * self.color_ratio,
            ),
        )
    }
}

impl<C1: Choice, C2: Choice> Clone for AdvanceTwoRule<C1, C2> {
    fn clone(&self) -> Self {
        Self {
            choices: self.choices.clone(),
            move_ratio: self.move_ratio,
            color_ratio: self.color_ratio,
            skew: self.skew
        }
    }
}

impl<C1: Choice, C2: Choice> Rule for AdvanceTwoRule<C1, C2> {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        _scatter: bool,
    ) -> (Point, usize) {
        let index1 = self.choices.0.choose_point(history, shape);
        let point1 = self.advance(previous, shape[index1], self.move_ratio.0);

        let index2 = self.choices.1.choose_point(history, shape);
        let point2 = self.advance(previous, shape[index2], self.move_ratio.1);

        (
            Point::new(
                point1.x * (1.0 - self.skew) + point2.x * self.skew,
                point1.y * (1.0 - self.skew) + point2.y * self.skew,
                (
                    point1.r * (1.0 - self.skew) + point2.r * self.skew,
                    point1.g * (1.0 - self.skew) + point2.g * self.skew,
                    point1.b * (1.0 - self.skew) + point2.b * self.skew,
                )
            ),
            index1,
        )
    }

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.choices.0.reseed(seed);
        self.choices.1.reseed(seed);
    }
}
