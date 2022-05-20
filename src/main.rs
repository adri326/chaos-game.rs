use pixels::{Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

pub mod shape;
use shape::*;

pub mod world;
use world::*;

pub mod rules;
use rules::*;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 400;

pub const BG_R: f64 = 0.0001;
pub const BG_G: f64 = 0.0001;
pub const BG_B: f64 = 0.0001;

#[allow(dead_code)]
const PHI: f64 = 1.61803398874989484820458683436563811772030;

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

    let choice = AvoidChoice::new(rand::thread_rng(), 1);
    let rule = DefaultRule::new(choice.clone(), 0.5, 2.0 / 3.0);
    let rule = OrRule::new(
        rand::thread_rng(),
        rule,
        SpiralRule::new(
            rand::thread_rng(),
            DefaultRule::new(choice, 1.5, 1.0 / 3.0),
            (0.0, 0.05),
            (1.0, 0.90),
        ),
        0.95,
    );

    let shape = polygon(
        std::env::args()
            .last()
            .map(|a| a.parse::<usize>().ok())
            .flatten()
            .unwrap_or(3),
    );
    let color_a = from_srgb(163, 55, 191);
    let color_b = from_srgb(104, 106, 113);
    let shape = colorize(shape, color_a, color_b, 3);

    let mut world = World::new(WIDTH, HEIGHT, 1.1, 0.3, shape, rule);

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
