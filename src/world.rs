use super::shape::*;
use super::rules::*;

#[derive(Clone)]
pub struct World<R: Rule> {
    pixels: Vec<(f64, f64, f64, u64)>,
    pub zoom: f64,
    width: usize,
    height: usize,
    rule: R,
    steps: usize,
    pub shape: Shape
}

impl<R: Rule> World<R> {
    pub fn new(width: u32, height: u32, zoom: f64, shape: Shape, rule: R) -> Self {
        let width = width as usize;
        let height = height as usize;
        Self {
            width,
            height,
            zoom,
            pixels: vec![(0.0, 0.0, 0.0, 0); width * height],
            rule,
            steps: 0,
            shape
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as usize;
        self.height = height as usize;
        self.steps = 0;

        self.pixels = vec![(0.0, 0.0, 0.0, 0); self.width * self.height];
    }

    pub fn update(&mut self) {
        use rand::Rng;

        let mut point = (Point::new(0.0, 0.0, (0.0, 0.0, 0.0)), 0);
        const N_STEPS: usize = 1000000;

        for _n in 0..N_STEPS {
            point = self.rule.next(point, &self.shape);

            // let next = self.choose_point(point);
            // let adv = if self.rng.gen() {
            //     self.rng.gen::<f64>().powf(3.0) * 0.166 + 2.0/3.0
            // } else {
            //     1.5
            // };

            // // let adv = 1.0 / 3.0;
            // x = x * (1.0 - adv) + self.shape[next].x * adv;
            // y = y * (1.0 - adv) + self.shape[next].y * adv;

            // let center_pull = self.rng.gen::<f64>().powf(2.0) * (1.0 - ((x * x + y * y).sqrt() * -0.2).exp());
            // x *= 1.0 - center_pull;
            // y *= 1.0 - center_pull;

            // if let Some((vx, vy)) = self.get_coord((point.0).x, (point.0).y) {
            self.draw_pixel(point.0)
            // }
        }

        self.steps += N_STEPS;
    }

    // #[allow(unused_variables)]
    // fn choose_point(&mut self, prev: usize) -> usize {
    //     use rand::Rng;

    //     (prev + self.rng.gen_range(1..(self.shape.len() - 1))) % self.shape.len()
    // }

    #[inline]
    pub fn get_coord(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let ratio = self.width.min(self.height) as f64 / self.zoom / 2.0;
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;

        let x = (x * ratio + cx).floor();
        let y = (y * ratio + cy).floor();

        if x < 0.0 || y < 0.0 {
            return None
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

            pixel.0 += point.r;
            pixel.1 += point.g;
            pixel.2 += point.b;
            pixel.3 += 1;
        }
    }

    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        use std::ops::Neg;
        if self.steps == 0 {
            return;
        }

        let ratio = self.width as f64 * self.height as f64 / self.steps as f64;
        let ratio = ratio * 0.1;

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if i > self.width * self.height {
                break
            }

            let (r, g, b, n) = self.pixels[i];
            let a = 1.0 - (n as f64 * ratio).neg().exp();
            let r = ((r / n as f64 * a).powf(1.0/2.2) * 255.0) as u8;
            let g = ((g / n as f64 * a).powf(1.0/2.2) * 255.0) as u8;
            let b = ((b / n as f64 * a).powf(1.0/2.2) * 255.0) as u8;

            if n > 0 {
                pixel[0] = r;
                pixel[1] = g;
                pixel[2] = b;
                pixel[3] = 255;
            } else {
                pixel[0] = 0;
                pixel[1] = 0;
                pixel[2] = 0;
                pixel[3] = 0;
            }
        }
    }
}
