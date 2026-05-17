use crate::ppu::{DISPLAY_HEIGHT, DISPLAY_WIDTH, GbDisplay};

use sdl2::{EventPump, event::Event, render::Canvas, video::Window};

pub const WINDOW_WIDTH: usize = 160;
pub const WINDOW_HEIGHT: usize = 144;
pub const WINDOW_SCALE_FACTOR: usize = 2;
const SCANCODE_MAX_VALUE: usize = 512;
const DISPLAY_WIDTH_BYTES: usize = DISPLAY_WIDTH * 4;
const DISPLAY_BYTES: usize = DISPLAY_WIDTH_BYTES * DISPLAY_HEIGHT;

// These are the four pixel colors that the gameboy can produce. Format: [R, G, B, A]
const COLOR_0: [u8; 4] = [0xE0, 0xF8, 0xD0, 0xFF]; //Color::RGB(224, 248, 208);
const COLOR_1: [u8; 4] = [0x88, 0xC0, 0x70, 0xFF]; //Color::RGB(136, 192, 112);
const COLOR_2: [u8; 4] = [0x34, 0x68, 0x56, 0xFF]; //Color::RGB(52, 104, 86);
const COLOR_3: [u8; 4] = [0x08, 0x18, 0x20, 0xFF]; //Color::RGB(8, 24, 32);

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
        self.canvas.clear();

        // Each pixel is a u32 RGBA value, but SDL wants an array of bytes
        let mut pixels_rgb: [u8; DISPLAY_BYTES] = [0; DISPLAY_BYTES];

        // This loop unpacks our 2d 4-color pixel array into the 1d RGBA byte array format that SDL wants.
        for (y, row) in display.iter().enumerate() {
            for (x, pixel) in row.iter().enumerate() {
                for rgba_segment in 0..4 {
                    pixels_rgb[(y * DISPLAY_WIDTH + x) * 4 + rgba_segment] = match pixel {
                        3 => COLOR_3[rgba_segment],
                        2 => COLOR_2[rgba_segment],
                        1 => COLOR_1[rgba_segment],
                        0 => COLOR_0[rgba_segment],
                        _ => panic!("Invalid pixel color value detected in the display"),
                    };
                }
            }
        }

        let texture_creator = self.canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(
                sdl2::pixels::PixelFormatEnum::RGBA32,
                DISPLAY_WIDTH as u32,
                DISPLAY_HEIGHT as u32,
            )
            .unwrap();
        texture
            .update(None, &pixels_rgb, DISPLAY_WIDTH_BYTES)
            .unwrap();

        let _ = self.canvas.copy(&texture, None, None);
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
