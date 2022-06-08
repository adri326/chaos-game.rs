use super::rules::*;
use super::shape::*;
use super::*;
use std::sync::mpsc::{TrySendError, Receiver};
use std::sync::{Arc, Mutex};
use worker_pool::{DownMsg, WorkerPool, WorkerSender};

#[derive(Clone, Copy)]
pub struct Pixel {
    pub r_sum: f64,
    pub g_sum: f64,
    pub b_sum: f64,
    pub n: f64,
    #[cfg(feature = "sigma")]
    pub l_sum: f64,
    #[cfg(feature = "sigma")]
    pub l_squared: f64,
}

impl Default for Pixel {
    fn default() -> Self {
        Self {
            r_sum: 0.0,
            g_sum: 0.0,
            b_sum: 0.0,
            n: 0.0,
            #[cfg(feature = "sigma")]
            l_sum: 0.0,
            #[cfg(feature = "sigma")]
            l_squared: 0.0,
        }
    }
}

impl Pixel {
    pub fn add(&mut self, point: Point) {
        self.r_sum += point.r * point.weight;
        self.g_sum += point.g * point.weight;
        self.b_sum += point.b * point.weight;
        self.n += point.weight;

        #[cfg(feature = "sigma")]
        if true {
            let lightness = point.lightness();
            self.l_sum += lightness * point.weight;
            self.l_squared += lightness * lightness * point.weight;
        }
    }

    pub fn add_pixel(&mut self, other: Pixel) {
        self.r_sum += other.r_sum;
        self.g_sum += other.g_sum;
        self.b_sum += other.b_sum;
        self.n += other.n;

        #[cfg(feature = "sigma")]
        if true {
            self.l_sum += other.l_sum;
            self.l_squared += other.l_squared;
        }
    }

    /// σ(Y)² = 𝔼[Y²] - 𝔼[Y]² with Y = ΣX/n
    /// Thus, σ(X)² = n*σ(Y)²/n² = σ(Y)²/n
    #[inline]
    #[cfg(feature = "sigma")]
    pub fn error_squared(&self) -> f64 {
        if self.n == 0.0 {
            0.0
        } else {
            let res = (self.l_squared / self.n) - (self.l_sum / self.n) * (self.l_sum / self.n);
            res / self.n
        }
    }

    #[inline]
    #[cfg(not(feature = "sigma"))]
    pub fn error_squared(&self) -> f64 {
        0.0
    }
}

pub struct WorldParams<R: Rule> {
    pub zoom: f64,
    pub center: (f64, f64),
    pub rule: RuleBox<R>,
    pub steps: usize,
    pub scatter_steps: usize,
    pub burnin_steps: usize,
    pub shape: Shape,
    pub gain: f64,
}

pub struct World {
    width: usize,
    height: usize,

    pub state: Arc<Mutex<Image>>,
    manager: WorkerPool<(), ManagerMsg>,
}

pub struct State {
    pub pixels: Vec<Pixel>,
    pub steps: usize,
    pub width: usize,
    pub height: usize
}

pub struct Image {
    pub pixels: Vec<u8>,
    pub steps: usize,
    pub width: usize,
    pub height: usize
}

#[derive(Clone, Debug)]
enum ManagerMsg {
    Resize(usize, usize),
}

// Acts as a middle-man between World and Worker; takes all the data sent by the workers and puts it into result_buffer
struct Manager<R: Rule> {
    state: State,
    tmp_buffer: Image,
    result_buffer: Arc<Mutex<Image>>,

    params: WorldParams<R>,
    workers: WorkerPool<State, ManagerMsg>,
    n_threads: usize,
}

struct Worker<R: Rule + 'static> {
    pixels: Vec<Pixel>,

    width: usize,
    height: usize,
    ratio: f64,
    steps: usize,

    params: WorldParams<R>,
}

impl World {
    pub fn new<R: Rule + 'static>(
        width: u32,
        height: u32,
        params: WorldParams<R>,
        n_threads: usize,
        queue_length: usize
    ) -> Self {
        let width = width as usize;
        let height = height as usize;

        let result_buffer = Arc::new(Mutex::new(
            Image::empty(width, height)
        ));
        let mut manager = WorkerPool::new(1);

        {
            let result_buffer = Arc::clone(&result_buffer);
            manager.execute(move |tx, rx| {
                let instance = Manager {
                    params,
                    workers: WorkerPool::new(queue_length),
                    n_threads,
                    state: State::empty(width, height),
                    tmp_buffer: Image::empty(width, height),
                    result_buffer
                };

                instance.run(tx, rx);
            });
        }

        Self {
            width,
            height,

            manager,
            state: result_buffer
        }
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }

    pub fn stop(&mut self) {
        for _ in self.manager.stop() {}
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as usize;
        self.height = height as usize;

        self.manager.broadcast(DownMsg::Other(ManagerMsg::Resize(self.width, self.height)));
    }

    pub fn draw(&self, frame: &mut [u8]) {
        if let Ok(state) = self.state.lock() {
            if state.width == self.width && state.height == self.height && frame.len() == state.pixels.len() {
                for (target, src) in frame.iter_mut().zip(state.pixels.iter()) {
                    *target = *src;
                }
            } else {
                let bg_r = (BG_R.powf(1.0 / GAMMA) * 255.0) as u8;
                let bg_g = (BG_G.powf(1.0 / GAMMA) * 255.0) as u8;
                let bg_b = (BG_B.powf(1.0 / GAMMA) * 255.0) as u8;

                for pixel in frame.chunks_exact_mut(4) {
                    pixel[0] = bg_r;
                    pixel[1] = bg_g;
                    pixel[2] = bg_b;
                    pixel[3] = 255;
                }
            }
        }
    }

    pub fn steps(&self) -> usize {
        self.state.lock().unwrap().steps
    }
}

impl<R: Rule + 'static> Manager<R> {
    fn run(mut self, _tx: WorkerSender<()>, rx: Receiver<DownMsg<ManagerMsg>>) {
        self.spawn_threads();

        loop {
            if let Some(msg) = worker_pool::try_recv_break!(rx) {
                match msg {
                    ManagerMsg::Resize(width, height) => {
                        self.resize(width, height);
                        continue;
                    }
                }
            }

            self.update();
            std::thread::sleep(std::time::Duration::new(0, 10_000_000));
        }

        self.stop();
        self.draw();
    }

    fn stop(&mut self) {
        for msg in self.workers.stop() {
            self.state.combine(msg);
        }
    }

    fn update(&mut self) {
        let mut received_msg = false;
        for msg in self.workers.recv_burst() {
            self.state.combine(msg);
            received_msg = true;
        }

        if received_msg {
            self.draw();
        }
    }

    fn draw(&mut self) {
        debug_assert!(self.tmp_buffer.width == self.state.width && self.tmp_buffer.height == self.state.height);
        self.state.draw(&mut self.tmp_buffer.pixels, self.params.gain);
        self.tmp_buffer.steps = self.state.steps;

        if let Ok(mut result_buffer) = self.result_buffer.lock() {
            std::mem::swap(&mut *result_buffer, &mut self.tmp_buffer);
        }
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.state.reset(width, height);

        self.workers.broadcast(DownMsg::Other(ManagerMsg::Resize(width, height)));

        self.tmp_buffer = Image::empty(width, height);
        *self.result_buffer.lock().unwrap() = Image::empty(width, height);
        // for _ in self.workers.stop() {}
        // self.spawn_threads();
    }

    fn spawn_threads(&mut self) {
        let params = self.params.clone();
        let pixels = vec![Pixel::default(); self.state.width * self.state.height];
        let width = self.state.width;
        let height = self.state.height;
        self.workers.execute_many(self.n_threads, move |tx, rx| {
            let worker = Worker {
                pixels,
                width,
                height,
                params,
                steps: 0,
                ratio: 0.0,
            };

            worker.run(tx, rx);
        });
    }
}

impl<R: Rule> Worker<R> {
    pub fn run(mut self, tx: WorkerSender<State>, rx: Receiver<DownMsg<ManagerMsg>>) {
        use rand::Rng;

        self.params.rule.reseed(&rand::thread_rng().gen());
        self.ratio = self.width.min(self.height) as f64 / self.params.zoom / 2.0;

        let mut first_iteration = true;

        loop {
            if let Some(msg) = worker_pool::try_recv_break!(rx) {
                match msg {
                    ManagerMsg::Resize(width, height) => {
                        self.width = width;
                        self.height = height;
                        self.steps = 0;
                        self.pixels = vec![Pixel::default(); width * height];
                        self.ratio = self.width.min(self.height) as f64 / self.params.zoom / 2.0;
                        first_iteration = true;
                        continue;
                    }
                }
            }

            let mut point = Point::new(0.0, 0.0, (0.0, 0.0, 0.0));
            let mut history = vec![0; 4];

            let n_steps = if first_iteration {
                first_iteration = false;
                (self.params.steps / 10).max(1)
            } else {
                self.params.steps
            };

            for _n in 0..self.params.burnin_steps {
                let (new_point, new_index) = self.params.rule.next(point, &history, &self.params.shape, false);

                point = new_point;

                history.rotate_right(1);
                history[0] = new_index;
            }

            for _n in 0..n_steps {
                for _nscatter in 0..self.params.scatter_steps {
                    let (new_point, _) =
                        self.params
                            .rule
                            .next(point, &history, &self.params.shape, true);
                    self.draw_pixel(new_point);
                }

                let (new_point, new_index) =
                    self.params
                        .rule
                        .next(point, &history, &self.params.shape, false);

                history.rotate_right(1);
                history[0] = new_index;

                self.draw_pixel(new_point);
                point = new_point;
            }

            self.steps += n_steps * (1 + self.params.scatter_steps);

            match tx.try_send(State::new(self.pixels, self.steps, self.width, self.height)) {
                Ok(_) => {
                    self.pixels = vec![Pixel::default(); self.width * self.height];
                    self.steps = 0;
                }
                Err(TrySendError::Full(msg)) => self.pixels = msg.pixels,
                Err(TrySendError::Disconnected(_)) => panic!("Manager disconnected!"),
            }
        }

        tx.send(State::new(self.pixels, self.steps, self.width, self.height)).unwrap();
    }

    #[inline]
    pub fn get_coord(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;

        let x = ((x - self.params.center.0) * self.ratio + cx).floor();
        let y = ((y - self.params.center.1) * self.ratio + cy).floor();

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
            self.pixels[x + y * self.width].add(point);
        }
    }
}

impl<R: Rule> Clone for WorldParams<R> {
    fn clone(&self) -> Self {
        Self {
            zoom: self.zoom,
            center: self.center,
            rule: self.rule.clone(),
            steps: self.steps,
            scatter_steps: self.scatter_steps,
            burnin_steps: self.burnin_steps,
            shape: self.shape.clone(),
            gain: self.gain,
        }
    }
}

impl State {
    pub fn empty(width: usize, height: usize) -> Self {
        Self {
            pixels: vec![Pixel::default(); width * height],
            steps: 0,
            width,
            height
        }
    }

    pub fn new(pixels: Vec<Pixel>, steps: usize, width: usize, height: usize) -> Self {
        assert!(pixels.len() == width * height);
        Self {
            pixels,
            steps,
            width,
            height
        }
    }

    pub fn combine(&mut self, other: State) -> bool {
        if other.width != self.width || other.height != self.height {
            return false
        }

        self.steps += other.steps;
        for (from, to) in other.pixels.into_iter().zip(self.pixels.iter_mut()) {
            to.add_pixel(from);
        }
        true
    }

    pub fn reset(&mut self, width: usize, height: usize) {
        if width * height == self.pixels.len() {
            for p in self.pixels.iter_mut() {
                *p = Pixel::default();
            }
        } else {
            self.pixels = vec![Pixel::default(); width * height];
        }
        self.width = width;
        self.height = height;
        self.steps = 0;
    }

    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8], gain: f64) {
        use std::ops::Neg;

        let bg_r = (BG_R.powf(1.0 / GAMMA) * 255.0) as u8;
        let bg_g = (BG_G.powf(1.0 / GAMMA) * 255.0) as u8;
        let bg_b = (BG_B.powf(1.0 / GAMMA) * 255.0) as u8;

        // Nothing to draw, simply fill the buffer with the background color
        if self.steps == 0 || self.pixels.len() * 4 != frame.len() {
            for pixel in frame.chunks_exact_mut(4) {
                pixel[0] = bg_r;
                pixel[1] = bg_g;
                pixel[2] = bg_b;
                pixel[3] = 255;
            }
            return;
        }

        let ratio = self.width as f64 * self.height as f64 / self.steps as f64 * gain;

        // Draw all the pixels
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if i > self.width * self.height {
                break;
            }

            let p = self.pixels[i];
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
                pixel[0] = bg_r;
                pixel[1] = bg_g;
                pixel[2] = bg_b;
                pixel[3] = 255;
            }
        }
    }

    pub fn mse(&self) -> f64 {
        if cfg!(feature = "sigma") {
            let mut res = 0.0;

            for pixel in self.pixels.iter() {
                res += pixel.error_squared();
            }

            res / self.pixels.len() as f64
        } else {
            0.0
        }
    }
}

impl Image {
    pub fn empty(width: usize, height: usize) -> Self {
        let bg_r = (BG_R.powf(1.0 / GAMMA) * 255.0) as u8;
        let bg_g = (BG_G.powf(1.0 / GAMMA) * 255.0) as u8;
        let bg_b = (BG_B.powf(1.0 / GAMMA) * 255.0) as u8;

        let mut res = vec![0u8; width * height * 4];

        for pixel in res.chunks_exact_mut(4) {
            pixel[0] = bg_r;
            pixel[1] = bg_g;
            pixel[2] = bg_b;
            pixel[3] = 255;
        }

        Self {
            pixels: res,
            steps: 0,
            width,
            height
        }
    }
}
