use super::GAMMA;

#[derive(Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Point {
    pub fn new(x: f64, y: f64, (r, g, b): (f64, f64, f64)) -> Self {
        Self {
            x,
            y,
            r: r * r,
            g: g * g,
            b: b * b,
        }
    }

    pub fn color(&self) -> (f64, f64, f64) {
        (self.r, self.g, self.b)
    }

    pub fn set_color(&mut self, color: (f64, f64, f64)) {
        self.r = color.0;
        self.g = color.1;
        self.b = color.2;
    }

    pub fn lightness(&self) -> f64 {
        // Since we're in linear color space, we can just use the L = 0.2126 * r + 0.7152 * g + 0.0722 * b formula:
        0.2126 * self.r + 0.7152 * self.g + 0.0722 * self.b
    }
}

pub type Shape = Vec<Point>;

pub fn polygon(n: usize) -> Shape {
    let mut res = Vec::with_capacity(n);

    for i in 0..n {
        let phase = i as f64 / n as f64 * std::f64::consts::TAU - std::f64::consts::PI / 2.0;
        res.push(Point::new(
            phase.cos(),
            phase.sin(),
            (
                0.5 + 0.5 * (phase * 0.6 + 0.7).cos(),
                0.5,
                0.5 + 0.5 * (phase * 0.6 + 0.7).sin(),
            ),
        ));
    }

    Shape::from(res)
}

pub fn from_srgb(r: u8, g: u8, b: u8) -> (f64, f64, f64) {
    (
        (r as f64 / 255.0).powf(GAMMA),
        (g as f64 / 255.0).powf(GAMMA),
        (b as f64 / 255.0).powf(GAMMA),
    )
}

pub fn colorize(shape: Shape, from: (f64, f64, f64), to: (f64, f64, f64), modulus: usize) -> Shape {
    let len = shape.len();
    let mut res = Vec::with_capacity(len);

    for (i, mut point) in shape.into_iter().enumerate() {
        let ratio = (i % modulus) as f64 / (modulus - 1) as f64;

        point.r = from.0 + (to.0 - from.0) * ratio;
        point.g = from.1 + (to.1 - from.1) * ratio;
        point.b = from.2 + (to.2 - from.2) * ratio;

        res.push(point);
    }

    res
}
