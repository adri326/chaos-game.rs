use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 400;

#[derive(Clone)]
struct Point {
    x: f64,
    y: f64,
    r: f64,
    g: f64,
    b: f64
}

type WorldRng = rand::rngs::ThreadRng;

#[derive(Clone)]
struct World {
    pixels: Vec<(f64, f64, f64, u64)>,
    zoom: f64,
    width: usize,
    height: usize,
    rng: WorldRng,
    steps: usize,
    shape: Vec<Point>
}

fn polygon(n: usize) -> Vec<Point> {
    let mut res = Vec::with_capacity(n);

    for i in 0..n {
        let phase = i as f64 / n as f64 * std::f64::consts::TAU;
        res.push(Point::new(
            phase.cos(),
            phase.sin(),
            (
                0.5 + 0.5 * (phase * 0.6 + 0.7).cos(),
                0.3,
                0.5 + 0.5 * (phase * 0.6 + 0.7).sin()
            )
        ));
    }

    res
}

fn main() -> Result<(), pixels::Error> {
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Chaos Game")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    let mut world = World::new(WIDTH, HEIGHT, 1.5, polygon(5));

    event_loop.run(move |event, _, control_flow| {
        if let Event::RedrawRequested(_) = event {
            world.draw(pixels.get_frame());

            if pixels
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        if input.update(&event) {
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                pixels.resize_buffer(size.width, size.height);
                world.resize(size.width, size.height);
            }

            world.update();
            world.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| eprintln!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }
    })
}

impl Point {
    fn new(
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
}

impl World {
    fn new(width: u32, height: u32, zoom: f64, shape: Vec<Point>) -> Self {
        let width = width as usize;
        let height = height as usize;
        Self {
            width,
            height,
            zoom,
            pixels: vec![(0.0, 0.0, 0.0, 0); width * height],
            rng: rand::thread_rng(),
            steps: 0,
            shape
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width as usize;
        self.height = height as usize;
        self.steps = 0;

        self.pixels = vec![(0.0, 0.0, 0.0, 0); self.width * self.height];
    }

    fn update(&mut self) {
        use rand::Rng;

        let mut x = 0.0;
        let mut y = 0.0;
        let mut point = 0;
        const N_STEPS: usize = 1000000;

        for _n in 0..N_STEPS {
            let next = self.choose_point(point);
            let adv = if self.rng.gen() {
                self.rng.gen::<f64>().powf(3.0) * 0.166 + 2.0/3.0
            } else {
                1.5
            };

            // let adv = 1.0 / 3.0;
            x = x * (1.0 - adv) + self.shape[next].x * adv;
            y = y * (1.0 - adv) + self.shape[next].y * adv;

            let center_pull = self.rng.gen::<f64>().powf(2.0) * (1.0 - ((x * x + y * y).sqrt() * -0.2).exp());
            x *= 1.0 - center_pull;
            y *= 1.0 - center_pull;

            if let Some((vx, vy)) = self.get_coord(x, y) {
                self.draw_pixel(vx, vy, point)
            }
            point = next;
        }

        self.steps += N_STEPS;
    }

    #[allow(unused_variables)]
    fn choose_point(&mut self, prev: usize) -> usize {
        use rand::Rng;

        (prev + self.rng.gen_range(1..(self.shape.len() - 1))) % self.shape.len()
    }

    #[inline]
    fn get_coord(&self, x: f64, y: f64) -> Option<(usize, usize)> {
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

    fn draw_pixel(&mut self, x: usize, y: usize, point: usize) {
        let mut pixel = &mut self.pixels[x + y * self.width];
        let point = &self.shape[point];

        pixel.0 += point.r;
        pixel.1 += point.g;
        pixel.2 += point.b;
        pixel.3 += 1;
    }

    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
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
            let r = ((r / n as f64 * a).sqrt() * 255.0) as u8;
            let g = ((g / n as f64 * a).sqrt() * 255.0) as u8;
            let b = ((b / n as f64 * a).sqrt() * 255.0) as u8;

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
