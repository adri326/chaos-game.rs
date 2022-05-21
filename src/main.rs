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

// const WIDTH: u32 = 1920 * 4;
// const HEIGHT: u32 = 1080 * 4;
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const RESIZE: bool = true;

pub const BG_R: f64 = 0.001;
pub const BG_G: f64 = 0.001;
pub const BG_B: f64 = 0.001;

pub const GAMMA: f64 = 2.2;

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

    let rule = TensorRule::new(TensorChoice::new(
            NeighborChoice::new(2),
            AvoidTwoChoice::new(0, 0),
            0.5,
            false
        ))
        .scale(0.2)
        .jump_center(true)
        .color_small(true)
        .move_ratio(2.0/3.0)
        .jump_ratio(0.5);

    let rule = OrRule::new(
        rule,
        DarkenRule::new(SpiralRule::new(
            TensorRule::new(TensorChoice::new(
                NeighborChoice::new(1),
                NeighborChoice::new(1),
                0.5,
                false
            ))
            .scale(0.2)
            .jump_center(false)
            .color_small(false)
            .move_ratio(-0.25)
            .jump_ratio(PHI - 1.0),
            (-0.1, 0.1),
            (1.0, 1.0)
        ), 0.8),
        0.9,
        0.25
    );

    // let choice = AvoidTwoChoice::new(1, -1);
    // let rule = DefaultRule::new(choice.clone(), 0.5, 1.0 / 3.0);
    // let rule = OrRule::new(
    //     rule,
    //     DarkenRule::new(
    //         SpiralRule::new(
    //             DefaultRule::new(AvoidChoice::new(0), 1.5, 1.0 / 3.0),
    //             (0.0, 0.05),
    //             (1.0, 0.90),
    //         ),
    //         0.5,
    //     ),
    //     0.9,
    //     0.4,
    // );

    let shape = polygon(
        std::env::args()
            .last()
            .map(|a| a.parse::<usize>().ok())
            .flatten()
            .unwrap_or(3),
    );
    let color_a = from_srgb(202, 147, 242);
    let color_b = from_srgb(120, 52, 120);
    let shape = colorize(shape, color_a, color_b, 5);

    let params = WorldParams {
        zoom: 1.2,
        rule,
        shape,
        steps: 1_000_000,
        scatter_steps: 7,
    };

    let mut world = World::new(WIDTH, HEIGHT, 0.1, params, 16);

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
                world.stop();
                world.update();
                println!("{} iterations, MSE: {:.8}", world.steps(), world.mse());
                *control_flow = ControlFlow::Exit;

                let mut buffer = vec![0; world.width() as usize * world.height() as usize * 4];
                world.draw(&mut buffer);
                image::save_buffer(
                    "./output.png",
                    &buffer,
                    world.width(),
                    world.height(),
                    image::ColorType::Rgba8,
                )
                .expect("Couldn't save result to disk!");

                return;
            }

            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
                if RESIZE {
                    pixels.resize_buffer(size.width, size.height);
                    world.resize(size.width, size.height);
                }
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
