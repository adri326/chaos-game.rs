use super::rules::*;
use super::shape::*;
use super::*;

#[derive(Clone)]
pub struct Pixel {
    pub r_sum: f64,
    pub g_sum: f64,
    pub b_sum: f64,
    pub n: f64,
    pub l_sum: f64,
    pub l_squared: f64
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            r_sum: 0.0,
            g_sum: 0.0,
            b_sum: 0.0,
            n: 0.0,
            l_sum: 0.0,
            l_squared: 0.0
        }
    }
}

impl Pixel {
    pub fn add(&mut self, point: Point) {
        self.r_sum += point.r;
        self.g_sum += point.g;
        self.b_sum += point.b;
        self.n += 1.0;

        let lightness = point.lightness();
        self.l_sum += lightness;
        self.l_squared += lightness * lightness;
    }

    /// σ(Y)² = 𝔼[Y²] - 𝔼[Y]² with Y = ΣX/n
    /// Thus, σ(X)² = n*σ(Y)²/n² = σ(Y)²/n
    pub fn error_squared(&self) -> f64 {
        if self.n == 0.0 {
            0.0
        } else {
            let res = (self.l_squared / self.n) - (self.l_sum / self.n) * (self.l_sum / self.n);
            res / self.n
        }
    }
}

#[derive(Clone)]
pub struct World<R: Rule> {
    pixels: Vec<Pixel>,
    pub zoom: f64,
    pub gain: f64,
    width: usize,
    height: usize,
    rule: R,
    steps: usize,
    pub shape: Shape,
}

impl<R: Rule> World<R> {
    pub fn new(width: u32, height: u32, zoom: f64, gain: f64, shape: Shape, rule: R) -> Self {
        let width = width as usize;
        let height = height as usize;
        Self {
            width,
            height,
            zoom,
            gain,
            pixels: vec![Pixel::default(); width * height],
            rule,
            steps: 0,
            shape,
        }
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as usize;
        self.height = height as usize;
        self.steps = 0;

        self.pixels = vec![Pixel::default(); self.width * self.height];
    }

    pub fn update(&mut self, steps: usize) {
        let mut point = Point::new(0.0, 0.0, (0.0, 0.0, 0.0));
        let mut history = vec![0; 4];

        for _n in 0..steps {
            let (new_point, new_index) = self.rule.next(point, &history, &self.shape);

            history.rotate_right(1);
            history[0] = new_index;

            self.draw_pixel(new_point);
            point = new_point;
        }

        self.steps += steps;
    }

    #[inline]
    pub fn get_coord(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let ratio = self.width.min(self.height) as f64 / self.zoom / 2.0;
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;

        let x = (x * ratio + cx).floor();
        let y = (y * ratio + cy).floor();

        if x < 0.0 || y < 0.0 {
            return None;
        }

        let x = x as usize;
        let y = y as usize;
        if x < self.width && y < self.height {
            Some((x, y))
        } else {
            None
        }
    }

    #[inline]
    pub fn draw_pixel(&mut self, point: Point) {
        if let Some((x, y)) = self.get_coord(point.x, point.y) {
            let mut pixel = &mut self.pixels[x + y * self.width];

            pixel.add(point);
        }
    }

    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        use std::ops::Neg;
        if self.steps == 0 {
            return;
        }

        let ratio = self.width as f64 * self.height as f64 / self.steps as f64;
        let ratio = ratio * self.gain;

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if i > self.width * self.height {
                break;
            }

            let p = &self.pixels[i];
            let a = 1.0 - (p.n * ratio).neg().exp();
            let r = ((p.r_sum / p.n * a + BG_R * (1.0 - a)).powf(1.0 / GAMMA) * 255.0) as u8;
            let g = ((p.g_sum / p.n * a + BG_G * (1.0 - a)).powf(1.0 / GAMMA) * 255.0) as u8;
            let b = ((p.b_sum / p.n * a + BG_B * (1.0 - a)).powf(1.0 / GAMMA) * 255.0) as u8;

            if p.n > 0.0 {
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
                pixel[3] = 255;
            } else {
                pixel[0] = (BG_R.powf(1.0 / GAMMA) * 255.0) as u8;
                pixel[1] = (BG_G.powf(1.0 / GAMMA) * 255.0) as u8;
                pixel[2] = (BG_B.powf(1.0 / GAMMA) * 255.0) as u8;
                pixel[3] = 255;
            }
        }
    }

    pub fn steps(&self) -> usize {
        self.steps
    }

    pub fn mse(&self) -> f64 {
        let mut res = 0.0;

        for pixel in self.pixels.iter() {
            res += pixel.error_squared();
        }

        res / self.pixels.len() as f64
    }
}
