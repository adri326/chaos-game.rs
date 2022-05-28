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
const HEADLESS: bool = false;

pub const BG_R: f64 = 0.001;
pub const BG_G: f64 = 0.001;
pub const BG_B: f64 = 0.001;

pub const GAMMA: f64 = 2.2;

#[allow(dead_code)]
const PHI: f64 = 1.61803398874989484820458683436563811772030;

fn main() -> Result<(), pixels::Error> {

    let rule = TensorRule::new(TensorChoice::new(
            NeighborhoodChoice::new(2),
            NeighborChoice::new(1),
            0.5,
            false
        ))
        .scale((PHI - 1.0) / 4.0)
        .jump_center(true)
        .color_small(false)
        .move_ratio(PHI - 1.0)
        .color_ratio(1.0 / 3.0)
        .jump_ratio(PHI - 1.0)
        .discrete_spiral((0.9, 0.5), std::f64::consts::PI / 7.0, 1.0 / 2.0, 0.975);

    let rule = OrRule::new(
        rule,
        DefaultRule::new(DefaultChoice::default(), -0.5, 0.5)
        .tensored()
        .spiral(
            (0.0, 0.0),
            (0.95, 1.05)
        )
        .darken(0.25),
        0.95,
        0.5
    );

    // let choice = AvoidTwoChoice::new(1, -1);
    // let rule = DefaultRule::new(choice.clone(), 0.5, 1.0 / 3.0);
    // let rule = OrRule::new(
    //     rule,
    //     DefaultRule::new(AvoidChoice::new(0), 1.5, 1.0 / 3.0).spiral(
    //         (0.0, 0.05),
    //         (1.0, 0.90),
    //     )
    //     .darken(0.5),
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
    let color_a = from_srgb(160, 147, 242);
    let color_b = from_srgb(186, 190, 220);
    let shape = colorize(shape, color_a, color_b, 4);

    let params = WorldParams {
        zoom: 1.25,
        rule: RuleBox::new(rule),
        shape,
        steps: 1_000_000,
        scatter_steps: 3,
    };

    let mut world = World::new(WIDTH, HEIGHT, 0.1, params, 2, 10);

    if HEADLESS {
        let (tx, rx) = std::sync::mpsc::channel();

        ctrlc::set_handler(move || tx.send(()).expect("Couldn't notify the main thread of ctrl-c")).expect("Error listening for ctrl-c");

        loop {
            world.update();
            if let Ok(_) = rx.try_recv() {
                break
            }
        }

        world.stop();
        println!("{} iterations, MSE: {:.8}", world.steps(), world.mse());

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

        Ok(())
    } else {
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
}
