use std::{collections::HashMap, hash::Hash, iter::Scan};

use crate::ppu::GbDisplay;

use sdl2::{
    EventPump, event::Event, keyboard::Scancode, pixels::Color, rect::Rect, render::Canvas,
    video::Window,
};

pub const WINDOW_WIDTH: usize = 160;
pub const WINDOW_HEIGHT: usize = 144;
pub const WINDOW_SCALE_FACTOR: usize = 2;

const SCANCODE_MAX_VALUE: usize = 512;

#[derive(Clone, Debug)]
pub struct Inputs {
    pub key_down: [bool; SCANCODE_MAX_VALUE],
    pub key_was_down: [bool; SCANCODE_MAX_VALUE],
    pub keypress_unique: [bool; SCANCODE_MAX_VALUE],
}

impl Inputs {
    fn new() -> Self {
        Inputs {
            key_down: [false; SCANCODE_MAX_VALUE],
            key_was_down: [false; SCANCODE_MAX_VALUE],
            keypress_unique: [false; SCANCODE_MAX_VALUE],
        }
    }
}

pub struct UserInterface {
    pub inputs: Inputs,

    canvas: Canvas<Window>,
    event_pump: EventPump,
    pub running: bool,
}

impl UserInterface {
    pub fn new() -> Self {
        let (canvas, event_pump) = UserInterface::init_window();
        UserInterface {
            canvas,
            event_pump,
            inputs: Inputs::new(),
            running: true,
        }
    }

    fn init_window() -> (Canvas<Window>, EventPump) {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window(
                "Gameboy",
                (WINDOW_WIDTH * WINDOW_SCALE_FACTOR) as u32,
                (WINDOW_HEIGHT * WINDOW_SCALE_FACTOR) as u32,
            )
            .position_centered()
            .build()
            .unwrap();

        // Rendering
        let canvas = window.into_canvas().build().unwrap();

        // Window events
        let event_pump = sdl_context.event_pump().unwrap();

        (canvas, event_pump)
    }

    pub fn render_display(&mut self, display: &GbDisplay) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();
        for (y, row) in display.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                let color = match pixel {
                    3 => Color::RGB(8, 24, 32),
                    2 => Color::RGB(52, 104, 86),
                    1 => Color::RGB(136, 192, 112),
                    0 => Color::RGB(224, 248, 208),
                    _ => panic!("Invalid pixel color value detected in the display"),
                };
                self.canvas.set_draw_color(color);

                self.canvas
                    .fill_rect(Rect::new(
                        (x as i32) * (WINDOW_SCALE_FACTOR as i32),
                        (y as i32) * (WINDOW_SCALE_FACTOR as i32),
                        WINDOW_SCALE_FACTOR as u32,
                        WINDOW_SCALE_FACTOR as u32,
                    ))
                    .unwrap();
            }
        }

        self.canvas.present();
    }

    pub fn process_inputs(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => self.running = false,
                Event::KeyDown {
                    scancode: Some(scancode),
                    ..
                } => self.inputs.key_down[scancode as usize] = true,
                Event::KeyUp {
                    scancode: Some(scancode),
                    ..
                } => self.inputs.key_down[scancode as usize] = false,
                _ => {}
            }
        }

        for i in 0..self.inputs.key_down.len() {
            self.inputs.keypress_unique[i] =
                self.inputs.key_down[i] && !self.inputs.key_was_down[i];

            self.inputs.key_was_down[i] = self.inputs.key_down[i];
        }
    }
}
