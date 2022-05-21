use super::*;

#[derive(Clone)]
pub struct TensorRule<C: Choice = TensorChoice> {
    choice: C,
    pub move_ratio: f64,
    pub jump_ratio: f64,
    pub color_ratio: f64,
    pub scale: f64,
    pub jump_center: bool,
    pub color_small: bool,
}

impl<C: Choice> TensorRule<C> {
    pub fn new(choice: C) -> Self {
        Self {
            choice,
            move_ratio: 0.5,
            jump_ratio: 0.5,
            color_ratio: 1.0,
            scale: 0.2,
            jump_center: false,
            color_small: false,
        }
    }

    pub fn move_ratio(mut self, move_ratio: f64) -> Self {
        self.move_ratio = move_ratio;
        self
    }

    pub fn jump_ratio(mut self, jump_ratio: f64) -> Self {
        self.jump_ratio = jump_ratio;
        self
    }

    pub fn color_ratio(mut self, color_ratio: f64) -> Self {
        self.color_ratio = color_ratio;
        self
    }

    pub fn scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }

    pub fn jump_center(mut self, jump_center: bool) -> Self {
        self.jump_center = jump_center;
        self
    }

    pub fn color_small(mut self, color_small: bool) -> Self {
        self.color_small = color_small;
        self
    }
}

impl Default for TensorRule<TensorChoice> {
    fn default() -> Self {
        Self {
            choice: TensorChoice::default(),
            move_ratio: 0.5,
            jump_ratio: 0.5,
            color_ratio: 1.0,
            scale: 0.2,
            jump_center: false,
            color_small: false,
        }
    }
}
impl<C: Choice> Rule for TensorRule<C> {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        _scatter: bool,
    ) -> (Point, usize) {
        let index = self.choice.choose_point(history, shape);
        let point_big = shape[index / shape.len()];
        let point_small = shape[index % shape.len()];

        let jumped = index / shape.len() != history[0] / shape.len();
        let ratio = if jumped {self.jump_ratio} else {self.move_ratio};

        let dx = point_big.x + if self.jump_center && jumped {0.0} else {point_small.x} * self.scale - previous.x;
        let dy = point_big.y + if self.jump_center && jumped {0.0} else {point_small.y} * self.scale - previous.y;

        let dr = if self.color_small {point_small.r} else {point_big.r} - previous.r;
        let dg = if self.color_small {point_small.g} else {point_big.g} - previous.g;
        let db = if self.color_small {point_small.b} else {point_big.b} - previous.b;

        (
            Point::new(
                previous.x + dx * ratio,
                previous.y + dy * ratio,
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
pub struct TensorChoice<CBig: Choice = DefaultChoice, CSmall: Choice = DefaultChoice> {
    choice_big: CBig,
    choice_small: CSmall,
    rng: RuleRng,
    jump_prob: f64,
    jump_any: bool
}

impl<CBig: Choice, CSmall: Choice> TensorChoice<CBig, CSmall> {
    pub fn new(choice_big: CBig, choice_small: CSmall, jump_prob: f64, jump_any: bool) -> Self {
        Self {
            choice_big,
            choice_small,
            rng: RuleRng::from_entropy(),
            jump_prob,
            jump_any,
        }
    }
}

impl Default for TensorChoice<DefaultChoice, DefaultChoice> {
    fn default() -> Self {
        Self {
            choice_big: DefaultChoice::default(),
            choice_small: DefaultChoice::default(),
            rng: RuleRng::from_entropy(),
            jump_prob: 0.5,
            jump_any: false
        }
    }
}

impl<CBig: Choice, CSmall: Choice> Choice for TensorChoice<CBig, CSmall> {
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize {
        let len = shape.len();

        if self.rng.gen::<f64>() < self.jump_prob {
            let history2 = history.iter().map(|x| *x / len).collect::<Vec<_>>();

            let choice_big = self.choice_big.choose_point(&history2, shape);

            let choice_small = if self.jump_any {
                let history3 = history.iter().map(|x| *x % len).collect::<Vec<_>>();
                self.choice_small.choose_point(&history3, shape)
            } else {
                history[0] % len
            };

            choice_small + len * choice_big
        } else {
            let history2 = history.iter().map(|x| *x % len).collect::<Vec<_>>();
            let choice = self.choice_small.choose_point(&history2, shape);
            len * (history[0] / len) + choice
        }
    }
}
