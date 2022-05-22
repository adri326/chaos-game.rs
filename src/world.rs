use super::rules::*;
use super::shape::*;
use super::*;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::sync::Arc;
use std_semaphore::Semaphore;

#[derive(Clone)]
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

#[derive(Clone)]
pub struct WorldParams<R: Rule + Clone> {
    pub zoom: f64,
    pub rule: R,
    pub steps: usize,
    pub scatter_steps: usize,
    pub shape: Shape,
}

pub struct World<R: Rule + Clone> {
    pub gain: f64,
    width: usize,
    height: usize,
    total_steps: usize,

    pub params: WorldParams<R>,
    workers: Workers,
}

struct Workers {
    pixels: Vec<Pixel>,
    receiver: Receiver<(Vec<Pixel>, usize)>,
    transmit: Sender<(Vec<Pixel>, usize)>,
    threads: Vec<(JoinHandle<()>, Sender<()>)>,
    n_threads: usize,
    queue_sem: Arc<Semaphore>,
    queue_length: isize,
}

struct Worker<R: Rule + Clone + 'static> {
    rx: Receiver<()>,
    tx: Sender<(Vec<Pixel>, usize)>,
    tx_sem: Arc<Semaphore>,
    pixels: Vec<Pixel>,

    width: usize,
    height: usize,
    ratio: f64,

    params: WorldParams<R>,
}

impl<R: Rule + Clone + 'static> World<R> {
    pub fn new(
        width: u32,
        height: u32,
        gain: f64,
        params: WorldParams<R>,
        n_threads: usize,
        queue_length: isize
    ) -> Self {
        let width = width as usize;
        let height = height as usize;

        let workers = Workers::new(params.clone(), width, height, n_threads, queue_length);

        Self {
            width,
            height,
            gain,
            total_steps: 0,

            params,
            workers,
        }
    }

    pub fn width(&self) -> u32 {
        self.width as u32
    }

    pub fn height(&self) -> u32 {
        self.height as u32
    }

    pub fn stop(&mut self) {
        self.workers.stop();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width as usize;
        self.height = height as usize;
        self.total_steps = 0;

        // self.pixels = vec![Pixel::default(); self.width * self.height];
        self.workers.stop();
        self.workers
            .start(self.params.clone(), self.width, self.height);
    }

    pub fn update(&mut self) {
        self.total_steps += self.workers.recv();
    }

    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    pub fn draw(&self, frame: &mut [u8]) {
        use std::ops::Neg;
        if self.steps() == 0 {
            return;
        }

        let ratio = self.width as f64 * self.height as f64 / self.steps() as f64;
        let ratio = ratio * self.gain;

        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            if i > self.width * self.height {
                break;
            }

            let p = &self.workers.pixels[i];
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
        self.total_steps
    }

    pub fn mse(&self) -> f64 {
        if cfg!(feature = "sigma") {
            let mut res = 0.0;

            for pixel in self.workers.pixels.iter() {
                res += pixel.error_squared();
            }

            res / self.workers.pixels.len() as f64
        } else {
            0.0
        }
    }
}

impl Workers {
    pub fn new<R: Rule + 'static>(
        params: WorldParams<R>,
        width: usize,
        height: usize,
        n_threads: usize,
        queue_length: isize,
    ) -> Self {
        let (main_tx, main_rx) = mpsc::channel();

        let mut res = Self {
            pixels: vec![Pixel::default(); width * height],
            receiver: main_rx,
            transmit: main_tx,
            threads: Vec::with_capacity(n_threads),
            n_threads,
            queue_sem: Arc::new(Semaphore::new(queue_length)),
            queue_length
        };

        res.start(params, width, height);

        res
    }

    pub fn stop(&mut self) {
        let threads = std::mem::replace(&mut self.threads, Vec::with_capacity(self.n_threads));

        for worker in threads.iter() {
            (worker.1).send(()).expect("Error while stopping worker!");
        }

        // Note: prevent deadlock by allowing every thread to get past the semaphore condition
        while let Ok(x) = self.receiver.try_recv() {
            std::mem::drop(x);
            self.queue_sem.release();
        }

        for worker in threads.into_iter() {
            (worker.0)
                .join()
                .expect("Error while waiting for worker to stop!");
        }

        let (main_tx, main_rx) = mpsc::channel();
        self.receiver = main_rx;
        self.transmit = main_tx;
        self.queue_sem = Arc::new(Semaphore::new(self.queue_length));
    }

    pub fn start<R: Rule + 'static>(
        &mut self,
        params: WorldParams<R>,
        width: usize,
        height: usize,
    ) {

        for _n in 0..self.n_threads {
            let params = params.clone();
            let (thread_tx, thread_rx) = mpsc::channel();
            let main_tx = self.transmit.clone();
            let tx_sem = Arc::clone(&self.queue_sem);

            let handle = thread::spawn(move || {
                let worker = Worker {
                    rx: thread_rx,
                    tx: main_tx,
                    tx_sem,
                    pixels: vec![Pixel::default(); width * height],
                    width,
                    height,
                    params,
                    ratio: 0.0,
                };

                worker.run();
            });
            self.threads.push((handle, thread_tx));
        }

        self.pixels = vec![Pixel::default(); width * height];
    }

    pub fn recv(&mut self) -> usize {
        let mut total_steps = 0;

        while let Ok((pixels, steps)) = self.receiver.try_recv() {
            total_steps += steps;
            for (from, to) in pixels.into_iter().zip(self.pixels.iter_mut()) {
                to.add_pixel(from);
            }
            self.queue_sem.release();
        }

        total_steps
    }
}

impl<R: Rule + Clone> Worker<R> {
    pub fn run(mut self) {
        self.ratio = self.width.min(self.height) as f64 / self.params.zoom / 2.0;
        loop {
            let mut point = Point::new(0.0, 0.0, (0.0, 0.0, 0.0));
            let mut history = vec![0; 4];

            for n in 0..self.params.steps {
                if n % 1000 == 0 {
                    if let Ok(()) = self.rx.try_recv() {
                        return;
                    }
                }
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

            let steps = self.params.steps * (1 + self.params.scatter_steps);

            self.tx_sem.acquire();
            self.tx
                .send((self.pixels, steps))
                .expect("Error while transmitting results from worker thread!");

            if let Ok(()) = self.rx.try_recv() {
                return;
            }

            self.pixels = vec![Pixel::default(); self.width * self.height];
        }
    }

    #[inline]
    pub fn get_coord(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        let cx = self.width as f64 / 2.0;
        let cy = self.height as f64 / 2.0;

        let x = (x * self.ratio + cx).floor();
        let y = (y * self.ratio + cy).floor();

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
