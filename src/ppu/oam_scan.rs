use crate::{Ppu, mmu::memmap::OAM_START};
// OAM scan takes two dots/t-cycles per object, scanning 40 objects in total.

const BYTES_PER_OBJECT: u32 = 4;

impl Ppu {
    pub fn tick_oam_scan(&mut self) {
        if (self.mode_dots % 2) != 0 {
            return;
        }

        let object_number = (self.mode_dots / 2) - 1;
        let (y_position, x_position, tiles_index, flags, addr) = self.get_oam_bytes(&object_number);

        println!(
            "{}: {:0x}-{:0x}| x: {}, y: {}, idx: {}, flags: {}",
            object_number,
            addr,
            addr + BYTES_PER_OBJECT as u16 - 1,
            x_position,
            y_position,
            tiles_index,
            flags
        );
    }

    fn get_oam_bytes(&mut self, object_number: &u32) -> (u8, u8, u8, u8, u16) {
        let addr = OAM_START + ((object_number) * BYTES_PER_OBJECT) as u16;
        let y_position = self.read_byte(addr);
        let x_position = self.read_byte(addr);
        let tiles_index = self.read_byte(addr);
        let flags = self.read_byte(addr);

        (y_position, x_position, tiles_index, flags, addr)
    }
}
