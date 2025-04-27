use crate::mmu::memmap::VBLANK_INTERRUPT_BIT;

use super::{
    DISPLAY_HEIGHT, DOTS_PER_SCANLINE, HBLANK_MAX_DOTS, OAM_SCAN_DOTS, PIXEL_DRAW_MIN_DOTS, Ppu,
    PpuMode, VBLANK_DOTS, fetcher::FetcherState,
};

impl Ppu {
    pub fn oam_scan(&mut self) {
        // OAMSCAN -> PIXELDRAW
        if OAM_SCAN_DOTS == self.mode_dots {
            self.set_mode(PpuMode::PixelDraw);

            self.mmu.borrow_mut().vram_lock = true;
        }
    }

    pub fn pixel_draw(&mut self) {
        // PIXELDRAW -> HBLANK
        self.tick_fetcher();
        if PIXEL_DRAW_MIN_DOTS == self.mode_dots {
            self.set_mode(PpuMode::HBlank);

            // Reset fetcher
            self.fetcher.state = FetcherState::GetTile;
            self.wx_triggered = false;
            self.fetcher.drawing_window = false;

            let mut mmu = self.mmu.borrow_mut();
            mmu.vram_lock = false;
            mmu.oam_lock = false;
        }
    }

    pub fn hblank(&mut self) {
        if HBLANK_MAX_DOTS == self.mode_dots {
            // HBLANK -> VBLANK
            if self.ly == DISPLAY_HEIGHT - 1 {
                self.set_mode(PpuMode::VBlank);
                self.mmu
                    .borrow_mut()
                    .request_interrupt(VBLANK_INTERRUPT_BIT);
            // HBLANK -> OAMSCAN
            } else {
                self.set_mode(PpuMode::OamScan);
                self.mmu.borrow_mut().oam_lock = true;
            }
            self.inc_ly();
        }
    }

    pub fn vblank(&mut self) {
        if self.scanline_dots == DOTS_PER_SCANLINE {
            self.inc_ly();
            if VBLANK_DOTS == self.mode_dots {
                // println!("VBLANK -> OAM");
                self.set_mode(PpuMode::OamScan);
                self.mmu.borrow_mut().oam_lock = true;
                self.reset_ly();
                self.frame_complete = true;
            }
        }
    }
}
