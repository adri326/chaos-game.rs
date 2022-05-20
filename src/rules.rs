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

// === Choices ===

macro_rules! simple_choice {
    ($name:tt) => {
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
