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

type RuleInnerRng = rand_xoshiro::Xoshiro256Plus;

#[derive(Debug, PartialEq, Eq)]
pub struct RuleRng {
    pub instance_seed: [u8; 32],
    pub rng: RuleInnerRng,
}

#[cfg(feature = "box")]
pub trait Rule: Send + DynClone {
    fn next(
        &mut self,
        previous: Point,
        history: &[usize],
        shape: &Shape,
        scatter: bool,
    ) -> (Point, usize);

    fn reseed(&mut self, seed: &[u8; 32]);
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

    fn reseed(&mut self, seed: &[u8; 32]);
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

pub trait Choice: DynClone + Send {
    fn choose_point(&mut self, history: &[usize], shape: &Shape) -> usize;

    fn reseed(&mut self, seed: &[u8; 32]);
}

impl RuleRng {
    #[inline]
    pub fn reseed(&mut self, seed: &[u8; 32]) {
        for (instance_byte, seed_byte) in self.instance_seed.iter_mut().zip(seed.iter().copied()) {
            *instance_byte ^= seed_byte;
        }

        self.rng = RuleInnerRng::from_seed(self.instance_seed.clone());
    }
}

impl Clone for RuleRng {
    fn clone(&self) -> Self {
        Self {
            instance_seed: self.instance_seed.clone(),
            rng: RuleInnerRng::from_seed(self.instance_seed.clone())
        }
    }
}

impl rand::RngCore for RuleRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    #[inline]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.rng.fill_bytes(dest)
    }

    #[inline]
    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.rng.try_fill_bytes(dest)
    }
}

impl SeedableRng for RuleRng {
    type Seed = <RuleInnerRng as SeedableRng>::Seed;

    fn from_seed(seed: [u8; 32]) -> Self {
        Self {
            instance_seed: seed,
            rng: RuleInnerRng::from_seed(seed),
        }
    }

    fn seed_from_u64(seed: u64) -> Self {
        let mut rng = rand_xoshiro::SplitMix64::seed_from_u64(seed);
        Self::from_rng(&mut rng).unwrap()
    }
}

// === Rules ===

pub struct DefaultRule<C: Choice = DefaultChoice> {
    choice: RuleBox<C>,
    move_ratio: f64,
    color_ratio: f64,
}

impl<C: Choice> DefaultRule<C> {
    pub fn new(choice: C, move_ratio: f64, color_ratio: f64) -> Self {
        Self {
            choice: RuleBox::new(choice),
            move_ratio,
            color_ratio,
        }
    }
}

impl Default for DefaultRule<DefaultChoice> {
    fn default() -> Self {
        Self {
            choice: RuleBox::new(DefaultChoice::default()),
            move_ratio: 0.5,
            color_ratio: 1.0,
        }
    }
}

impl<C: Choice> Clone for DefaultRule<C> {
    fn clone(&self) -> Self {
        Self {
            choice: self.choice.clone(),
            move_ratio: self.move_ratio,
            color_ratio: self.color_ratio
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

    fn reseed(&mut self, seed: &[u8; 32]) {
        self.choice.reseed(seed);
    }
}

#[cfg(feature = "box")]
mod rule_box {
    use super::*;

    #[derive(Debug)]
    pub struct RuleBox<R: DynClone>(Box<R>);

    impl<R: DynClone> RuleBox<R> {
        pub fn new(rule: R) -> Self {
            Self(Box::new(rule))
        }
    }

    impl<R: DynClone> Clone for RuleBox<R> {
        fn clone(&self) -> Self {
            Self(clone_box(self.0.as_ref()))
        }
    }

    impl<R: DynClone> std::ops::Deref for RuleBox<R> {
        type Target = R;

        fn deref(&self) -> &Self::Target {
            self.0.as_ref()
        }
    }

    impl<R: DynClone> std::ops::DerefMut for RuleBox<R> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            self.0.as_mut()
        }
    }
}

#[cfg(not(feature = "box"))]
mod rule_box {
    use super::*;

    #[derive(Debug)]
    pub struct RuleBox<R: DynClone>(R);

    impl<R: DynClone> RuleBox<R> {
        pub fn new(rule: R) -> Self {
            Self(rule)
        }
    }

    impl<R: DynClone> Clone for RuleBox<R> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<R: DynClone> std::ops::Deref for RuleBox<R> {
        type Target = R;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl<R: DynClone> std::ops::DerefMut for RuleBox<R> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
}

pub use rule_box::RuleBox;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rand_reseed() {
        // The probability that this gives a false negative is 2^(-100*64), assuming xoshiro is a good PRNG algorithm
        for _ in 0..100 {
            let mut rng_base = RuleRng::from_entropy();
            let mut rng_clone = rng_base.clone();

            assert!(rng_base.gen::<u64>() == rng_clone.gen::<u64>());

            let mut rng_base = RuleRng::from_entropy();
            let mut rng_reseed = rng_base.clone();
            rng_reseed.reseed(&rand::thread_rng().gen());

            assert!(rng_base.gen::<u64>() != rng_reseed.gen::<u64>());

            let rng_base = RuleRng::from_entropy();
            let mut rng_reseed = rng_base.clone();
            let mut rng_reseed2 = rng_base.clone();
            let seed = rand::thread_rng().gen();
            rng_reseed.reseed(&seed);
            rng_reseed2.reseed(&seed);

            assert!(rng_reseed.gen::<u64>() == rng_reseed2.gen::<u64>());
        }
    }
}
