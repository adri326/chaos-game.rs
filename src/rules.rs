use super::shape::*;
use rand::{Rng, SeedableRng};
use rand_distr::Distribution;

#[cfg(feature = "box")]
use dyn_clone::{DynClone, clone_box};

#[cfg(feature = "box")]
pub mod boxed;
#[cfg(feature = "box")]
pub use boxed::*;

pub mod branch;
pub use branch::*;

pub mod color;
pub use color::*;

pub mod tensor;
pub use tensor::*;

pub mod transform;
pub use transform::*;

pub mod choice;
pub use choice::*;

type RuleRng = rand_xoshiro::Xoshiro256Plus;

#[cfg(feature = "box")]
pub trait Rule: Send + DynClone {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize);
}

#[cfg(not(feature = "box"))]
pub trait Rule: Sized + Send + Clone {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize);
}

pub trait RuleHelper: Sized + Rule {
    fn tensored(self) -> TensoredRule<Self> {
        TensoredRule::new(self)
    }

    fn spiral(self, delta: (f64, f64), epsilon: (f64, f64)) -> SpiralRule<Self> {
        SpiralRule::new(
            self,
            delta,
            epsilon
        )
    }

    fn discrete_spiral(self, (p, p_scatter): (f64, f64), delta: f64, epsilon: f64, darken: f64) -> DiscreteSpiralRule<Self> {
        if p < 0.0 || p > 1.0 || p_scatter < 0.0 || p_scatter > 1.0 {
            DiscreteSpiralRule::new(
                self,
                (1.0, 1.0),
                delta,
                epsilon,
                darken
            ).unwrap()
        } else {
            DiscreteSpiralRule::new(
                self,
                (p, p_scatter),
                delta,
                epsilon,
                darken
            ).unwrap()
        }
    }

    fn darken(self, amount: f64) -> DarkenRule<Self> {
        DarkenRule::new(
            self,
            amount
        )
    }
}

impl<R: Rule + Sized> RuleHelper for R {}

#[cfg(feature = "box")]
dyn_clone::clone_trait_object!(Rule);

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

#[cfg(feature = "box")]
mod rule_box {
    use super::*;

    pub struct RuleBox<R: Rule>(Box<R>);

    impl<R: Rule> RuleBox<R> {
        pub fn new(rule: R) -> Self {
            Self(Box::new(rule))
        }
    }

    impl<R: Rule> Clone for RuleBox<R> {
        fn clone(&self) -> Self {
            Self(clone_box(self.0.as_ref()))
        }
    }

    impl<R: Rule> std::ops::Deref for RuleBox<R> {
        type Target = R;

        fn deref(&self) -> &Self::Target {
            self.0.as_ref()
        }
    }

    impl<R: Rule> std::ops::DerefMut for RuleBox<R> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.as_mut()
        }
    }
}

#[cfg(not(feature = "box"))]
mod rule_box {
    use super::*;

    pub struct RuleBox<R: Rule>(R);

    impl<R: Rule> RuleBox<R> {
        pub fn new(rule: R) -> Self {
            Self(rule)
        }
    }

    impl<R: Rule> Clone for RuleBox<R> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<R: Rule> std::ops::Deref for RuleBox<R> {
        type Target = R;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<R: Rule> std::ops::DerefMut for RuleBox<R> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

pub use rule_box::RuleBox;
