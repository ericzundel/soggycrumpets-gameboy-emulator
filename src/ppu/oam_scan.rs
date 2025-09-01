use crate::ppu::tiles::{TILE_HEIGHT_IN_PIXELS, TILE_WIDTH_IN_PIXELS, get_tile_row};
use crate::ppu::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use crate::util::get_bit;
use crate::{Ppu, mmu::memmap::OAM_START};

// OAM scan takes two dots/t-cycles per object, scanning 40 objects in total.

const OBJECT_SIZE_BYTES: u32 = 4;
const SCREEN_BUFFER_X: u32 = 8;
const SCREEN_BUFFER_Y: u32 = 16;

type ObjectDisplay = [[Option<u8>; DISPLAY_WIDTH]; DISPLAY_HEIGHT];
pub struct OamData {
    pub object_display: ObjectDisplay,
}

impl OamData {
    pub fn new() -> Self {
        OamData {
            object_display: [[None; DISPLAY_WIDTH]; DISPLAY_HEIGHT],
        }
    }
}

struct ObjectFlags {
    priority: bool,
    xflip: bool,
    yflip: bool,
}

impl Ppu {
    pub fn tick_oam_scan(&mut self) {
        if (self.mode_dots % 2) != 0 {
            return;
        }

        let object_number = (self.mode_dots / 2) - 1;
        let object_addr = self.get_oam_addr(&object_number);
        let (y_position, x_position, tile_index, flags) = self.get_oam_bytes(&object_addr);
        let tile_start_addr = self.get_tile_start_addr(tile_index, true);

        let object_height = 8;

        // Display the object if it is on the current scanline
        if (self.ly as i32) >= (y_position as i32 - SCREEN_BUFFER_Y as i32)
            && (self.ly as i32) < (y_position as i32 - SCREEN_BUFFER_Y as i32 + object_height)
        {
            let mut tile_row_index = self.ly - (y_position - SCREEN_BUFFER_Y as u8);
            if flags.yflip {
                tile_row_index = (TILE_HEIGHT_IN_PIXELS - 1) as u8 - tile_row_index;
            }

            let tile_row_high_byte = self.get_tile_row_high_byte(tile_start_addr, tile_row_index);
            let tile_row_low_byte = self.get_tile_row_low_byte(tile_start_addr, tile_row_index);
            let mut object_row = get_tile_row(tile_row_low_byte, tile_row_high_byte);

            if flags.xflip {
                object_row.reverse();
            }

            self.write_row_to_display(&object_row, x_position);
        }

        // println!(
        //     "{}: {:0x}-{:0x} | x: {}, y: {}, idx: {}, tile addr: 0x{:0x} priority: {}, xflip: {}, yflip: {}",
        //     object_number,
        //     object_addr,
        //     object_addr + OBJECT_SIZE_BYTES as u16 - 1,
        //     x_position,
        //     y_position,
        //     tile_index,
        //     tile_start_addr,
        //     flags.priority,
        //     flags.xflip,
        //     flags.yflip,
        // );
    }

    fn get_oam_bytes(&mut self, addr: &u16) -> (u8, u8, u8, ObjectFlags) {
        let y_position = self.read_byte(*addr);
        let x_position = self.read_byte(*addr + 1);
        let tile_index = self.read_byte(*addr + 2);
        let flags_byte = self.read_byte(*addr + 3);
        let flags = ObjectFlags {
            priority: get_bit(flags_byte, 7),
            yflip: get_bit(flags_byte, 6),
            xflip: get_bit(flags_byte, 5),
        };

        (y_position, x_position, tile_index, flags)
    }

    fn get_oam_addr(&mut self, object_number: &u32) -> u16 {
        OAM_START + ((object_number) * OBJECT_SIZE_BYTES) as u16
    }

    fn write_row_to_display(&mut self, row: &[u8; TILE_WIDTH_IN_PIXELS], x_position: u8) {
        let mut pixel_x = x_position as i32 - SCREEN_BUFFER_X as i32;
        let pixel_y = self.ly;
        for pixel in row {
            if pixel_x >= 0 && pixel_x < (DISPLAY_HEIGHT as i32) {
                self.oam_data.object_display[pixel_y as usize][pixel_x as usize] = Some(*pixel);
            }
            pixel_x += 1;
        }
    }
}
