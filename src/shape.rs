#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub g: f64,
    pub b: f64
}

impl Point {
    pub fn new(
        x: f64,
        y: f64,
        (r, g, b): (f64, f64, f64)
    ) -> Self {
        Self {
            x,
            y,
            r: r * r,
            g: g * g,
            b: b * b
        }
    }

    pub fn color(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }
}

pub type Shape = Vec<Point>;

pub fn polygon(n: usize) -> Shape {
    let mut res = Vec::with_capacity(n);

    for i in 0..n {
        let phase = i as f64 / n as f64 * std::f64::consts::TAU;
        res.push(Point::new(
            phase.cos(),
            phase.sin(),
            (
                0.5 + 0.5 * (phase * 0.6 + 0.7).cos(),
                0.5,
                0.5 + 0.5 * (phase * 0.6 + 0.7).sin()
            )
        ));
    }

    Shape::from(res)
}
