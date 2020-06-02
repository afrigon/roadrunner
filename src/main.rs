mod game;
mod input;
mod network;
mod ops;
mod render;
mod utils;

extern crate bincode;
extern crate core;
extern crate gl;
extern crate glutin;
extern crate image;
extern crate math;
extern crate serde;
extern crate serde_json;

use crate::game::{Game, GameType};
use crate::input::InputHandler;
use crate::render::display::Display;

use core::utils::sleep;
use core::utils::{
    logging,
    logging::{info, Level, Logger},
};
use glutin::event::{DeviceEvent, Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use std::io;
use std::time::{Duration, Instant};

const FPS_REFRESH_TIMEOUT: u64 = 1;
const FRAME_RATE_CAP: u32 = 60;
const PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
const PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() -> io::Result<()> {
    logging::init(vec![
        Box::new(Logger::to_file("logs", Level::Debug)),
        Box::new(Logger::to_stdout(Level::Info)),
    ]);
    info!("Welcome to {} v{}", PKG_NAME, PKG_VERSION);

    let event_loop = EventLoop::new();
    let display = Display::new(PKG_NAME, &event_loop);
    let (width, height) = display.size();

    let mut input_handler = InputHandler::default();

    #[cfg(not(feature = "remote"))]
    let game_type = GameType::Local;
    #[cfg(feature = "remote")]
    let game_type = GameType::Remote {
        info: crate::network::RemoteInfo::new(String::from("localhost"), 25565),
    };

    let mut game = Game::new(game_type)?;
    game.resize(width, height);

    let mut fps: u32 = 0;
    let mut last_time = Instant::now();
    let mut last_fps_update = Instant::now();

    let expected_tick_duration = Duration::new(1, 0) / FRAME_RATE_CAP;

    event_loop.run(move |event, _, control_flow| match event {
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::MouseMotion { delta } => input_handler.process_cursor(delta),
            _ => (),
        },
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::KeyboardInput { input, .. } => input_handler.process_keyboard(input),
            WindowEvent::Resized(size) => {
                display.resize(size);
                game.resize(size.width as usize, size.height as usize);
            }
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                display.resize(*new_inner_size);
                game.resize(
                    new_inner_size.width as usize,
                    new_inner_size.height as usize,
                );
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => (),
        },
        Event::MainEventsCleared => {
            let time_delta = last_time.elapsed().as_secs_f64();
            last_time = Instant::now();

            if last_fps_update.elapsed().as_secs() >= FPS_REFRESH_TIMEOUT {
                fps = (1.0 / time_delta) as u32;
                info!("FPS: {}", fps);
                last_fps_update = Instant::now();
            }

            game.update(time_delta, &input_handler);

            input_handler.clear();

            display.request_redraw();
        }
        Event::RedrawRequested(_) => {
            game.render();
            display.swap_buffers();

            // makeshift fps limiting
            if let Some(cooldown) = expected_tick_duration.checked_sub(last_time.elapsed()) {
                sleep(cooldown);
            }
        }
        Event::LoopDestroyed => return,
        _ => (),
    });
}
