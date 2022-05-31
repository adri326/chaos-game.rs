use pixels::{Pixels, SurfaceTexture};
use clap::{arg, command};
use winit::{
    dpi::LogicalSize,
    event::{Event, VirtualKeyCode},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder
};
use winit_input_helper::WinitInputHelper;

use chaos_game::{
    shape::*,
    world::*,
    rules::*,
    script::*
};

// Default width and height for the window; the window can be resized if RESIZE = true, so these values can be ignored
const WIDTH: u32 = 1024;
const HEIGHT: u32 = 1024;
const RESIZE: bool = true;

fn main() -> Result<(), pixels::Error> {
    let (world, headless, max_steps) = handle_args();

    if headless {
        main_headless(world, max_steps);

        Ok(())
    } else {
        main_interactive(world, max_steps)
    }
}

fn main_headless<R: Rule + 'static>(mut world: World<R>, max_steps: Option<usize>) {
    let (tx, rx) = std::sync::mpsc::channel();

    ctrlc::set_handler(move || tx.send(()).expect("Couldn't notify the main thread of ctrl-c")).expect("Error listening for ctrl-c");

    loop {
        world.update(true);
        if let Ok(_) = rx.try_recv() {
            break
        }
        if let Some(max_steps) = max_steps {
            if world.steps() >= max_steps {
                break
            }
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
}

fn main_interactive<R: Rule + 'static>(mut world: World<R>, max_steps: Option<usize>) -> Result<(), pixels::Error> {
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
            if
                input.key_pressed(VirtualKeyCode::Escape)
                || input.quit()
                || max_steps.map(|m| world.steps() >= m).unwrap_or(false)
            {
                world.stop();
                world.update(false);
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

            world.update(false);
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

fn parse_int(raw: &str) -> Result<usize, std::num::ParseIntError> {
    let (raw, mult) = match raw.chars().last() {
        Some('k') | Some('K') => (&raw[0..(raw.len() - 1)], 1000),
        Some('m') | Some('M') => (&raw[0..(raw.len() - 1)], 1000_000),
        Some('b') | Some('B') => (&raw[0..(raw.len() - 1)], 1000_000_000),
        Some('t') | Some('T') => (&raw[0..(raw.len() - 1)], 1000_000_000_000),
        _ => (raw, 1)
    };

    raw.parse::<usize>().map(|x| x * mult)
}

fn parse_dim(raw: &str) -> Result<(u32, u32), String> {
    let mut iter = raw.split("x");
    let first = iter.next().ok_or(String::from("Expected a non-empty value"))?;
    let second = iter.next().ok_or(String::from("Expected value in format WIDTHxHEIGHT"))?;

    Ok((
        first.parse::<u32>().map_err(|e| format!("{:?}", e))?,
        second.parse::<u32>().map_err(|e| format!("{:?}", e))?
    ))
}

fn handle_args() -> (World<BoxedRule>, bool, Option<usize>) {
    let matches = command!()
        .arg(arg!([input] "The input script to run"))
        .arg(arg!(--headless "Whether to run in headless mode").required(false))
        .arg(
            arg!(--polygon <VALUE> "Number of sides that the default shape polygon will have, ignored if set by the input script")
            .default_value("3")
            .validator(|s| s.parse::<usize>())
        )
        .arg(arg!(--scale <VALUE> "The default scale factor, ignored if set by the input script").default_value("1.25").validator(|s| s.parse::<f64>()))
        .arg(arg!(--steps <VALUE> "Number of steps between an update").default_value("500000").validator(|s| parse_int(s)))
        .arg(arg!(--"scatter-steps" <VALUE> "Number of substeps that work as 'scatter' for each step").default_value("7").validator(|s| parse_int(s)))
        .arg(arg!(--"max-steps" <VALUE> "Stop the program if max-steps is reached").required(false))
        .arg(
            arg!(--queue-length <VALUE> "Maximum number of results that can sit in the queue; decrease if the program runs out of memory, increase if the queue becomes a bottleneck")
            .default_value(&format!("{}", 2 * num_cpus::get()))
            .validator(|s| parse_int(s))
        )
        .arg(
            arg!(--threads <VALUE> "Number of threads; defaults to the number of CPU logical cores")
            .default_value(&format!("{}", num_cpus::get()))
            .validator(|s| parse_int(s))
        )
        .arg(
            arg!(--dim <VALUE> "Dimension of the image, only valid in headless mode")
            .default_value(&format!("{}x{}", WIDTH, HEIGHT))
            .validator(|s| parse_dim(s))
        )
        .get_matches();

    // Execute input script
    let script = std::fs::read_to_string(
        matches.value_of("input").unwrap_or("rule.lisp")
    ).unwrap();
    let (rule, shape, scale, center) = eval_rule(&script).unwrap();

    // Extract rule
    let rule = rule.unwrap_or(BoxedRule::new(DefaultRule::default()));

    // Extract shape
    let shape = if let Some(shape) = shape {
        shape
    } else {
        let color_a = from_srgb(160, 147, 242);
        let color_b = from_srgb(186, 190, 220);
        let n_sides: usize = matches.value_of("polygon").unwrap().parse::<usize>().unwrap();
        colorize(polygon(n_sides), color_a, color_b, (n_sides / 2).max(1))
    };

    // Extract scale
    let scale = scale.unwrap_or(matches.value_of("scale").unwrap().parse::<f64>().unwrap());

    // Extract center
    let center = center.unwrap_or((0.0, 0.0));

    // TODO: rename zoom to scale
    let params = WorldParams {
        zoom: scale,
        center,
        rule: RuleBox::new(rule),
        shape,
        steps: parse_int(matches.value_of("steps").unwrap()).unwrap(),
        scatter_steps: parse_int(matches.value_of("scatter-steps").unwrap()).unwrap(),
    };

    let n_threads = parse_int(matches.value_of("threads").unwrap()).unwrap();

    let headless = matches.occurrences_of("headless") > 0;

    let max_steps = matches.value_of("max-steps").map(|s| parse_int(s).expect("Invalid value for max-steps"));

    let (width, height) = parse_dim(matches.value_of("dim").unwrap()).unwrap();

    // TODO: factor out gain (0.1)
    (World::new(width, height, 0.1, params, n_threads, 10), headless, max_steps)
}
