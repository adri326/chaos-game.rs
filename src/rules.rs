use rand::Rng;
use rand::rngs::ThreadRng;
use super::shape::*;

pub trait Rule {
    fn next(&mut self, previous: (Point, usize), shape: &Shape) -> (Point, usize);

    // fn choose_point(&mut self, previous: usize, shape: &Shape) -> usize;
}

pub struct DefaultRule<R: Rng> {
    rng: R,
    move_ratio: f64,
    color_ratio: f64
}

impl<R: Rng> DefaultRule<R> {
    pub fn new(rng: R, move_ratio: f64, color_ratio: f64) -> Self {
        Self {
            rng,
            move_ratio,
            color_ratio
        }
    }

    fn choose_point(&mut self, previous: usize, shape: &Shape) -> usize {
        self.rng.gen_range(0..shape.len())
    }
}

impl Default for DefaultRule<ThreadRng> {
    fn default() -> Self {
        Self {
            rng: rand::thread_rng(),
            move_ratio: 0.5,
            color_ratio: 1.0
        }
    }
}

impl<R: Rng> Rule for DefaultRule<R> {
    fn next(&mut self, (previous, index): (Point, usize), shape: &Shape) -> (Point, usize) {
        let index = self.choose_point(index, shape);
        let point = shape[index];
        let dx = point.x - previous.x;
        let dy = point.y - previous.y;

        let dr = point.r - previous.r;
        let dg = point.g - previous.g;
        let db = point.b - previous.b;

        (Point::new(
            previous.x + dx * self.move_ratio,
            previous.y + dy * self.move_ratio,
            // point.color()
            (
                previous.r + dr * self.color_ratio,
                previous.g + dg * self.color_ratio,
                previous.b + db * self.color_ratio,
            )
        ), index)
    }
}
