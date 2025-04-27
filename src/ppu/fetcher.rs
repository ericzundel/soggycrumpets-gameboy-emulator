use super::{
    tiles::{TILE_HEIGHT_IN_PIXELS, TILE_WIDTH_IN_PIXELS, get_tile_row},
    *,
};

const TILEMAP_1_ADDR: u16 = 0x9800;
const TILEMAP_2_ADDR: u16 = 0x9C00;
const TILEMAP_WIDTH: u16 = 32;

#[derive(Debug)]
pub enum FetcherState {
    GetTile,
    GetTileDataHigh,
    GetTileDataLow,
    Sleep,
    Push,
}

/// The fetcher is a state machine within the PPU state machine.
/// It retrieves pixel data from memory to be drawn to the screen, one tile row (8 pixels) at a time.
/// It operates over the duration of the pixel-draw state, retrieving one scanline worth of pixels.
#[derive(Debug)]
pub struct Fetcher {
    pub state: FetcherState,

    x: u8,
    y: u8,
    tile_x: u8,
    tile_y: u8,

    pub drawing_window: bool,

    tile_addr: u16,
    tile_data_low: u8,
    tile_data_high: u8,
}

impl Fetcher {
    pub fn new() -> Self {
        Fetcher {
            state: FetcherState::GetTile,

            x: 0,
            y: 0,
            tile_x: 0,
            tile_y: 0,

            drawing_window: false,

            tile_addr: 0x0000,
            tile_data_low: 0,
            tile_data_high: 0,
        }
    }
}

impl Ppu {
    /// This implementation of the fetcher, at least for now, is greatly simplified.
    /// Each state is completing within a constant time of 2 dots/t-cycles.
    /// The fetcher normally works with a pixel FIFO (first-in, first-out) to render
    /// to the screen, but this implementation does not use the FIFO.
    /// The way it works now: 8 t-cycles to draw 8 pixels, 160 t-cycles to draw a scanline.
    /// The fetcher progresses one state every other t-cycle.
    pub fn tick_fetcher(&mut self) {
        if self.mode_dots % 2 != 0 {
            return;
        }

        if self.mode_dots > WINDOW_WIDTH as u32 {
            return;
        }

        match self.fetcher.state {
            FetcherState::GetTile => {
                self.fetcher_get_tile();
                self.fetcher.state = FetcherState::GetTileDataLow;
            }
            FetcherState::GetTileDataLow => {
                self.fetcher_get_tile_data(false);
                self.fetcher.state = FetcherState::GetTileDataHigh;
            }
            FetcherState::GetTileDataHigh => {
                self.fetcher_get_tile_data(true);
                self.fetcher.state = FetcherState::Push;
            }
            FetcherState::Sleep => self.fetcher_sleep(),
            FetcherState::Push => {
                self.fetcher_push();
                self.fetcher.state = FetcherState::GetTile;
                self.lx += TILE_WIDTH_IN_PIXELS as u8;
            }
        }
    }

    fn fetcher_get_tile(&mut self) {
        let bg_tile_map = self.get_lcdc_flag(BG_TILE_MAP_BIT);
        let window_tile_map = self.get_lcdc_flag(WINDOW_TILE_MAP_BIT);

        let window_enable = self.get_lcdc_flag(WINDOW_ENABLE_BIT);

        self.update_wx();
        self.fetcher.drawing_window = self.wx_triggered && self.wy_triggered && window_enable;
        if self.fetcher.drawing_window {
            self.window_drawn_this_scanline = true;
        }

        // The two tilemap addresses can be accessed both in background mode and in window mode.
        // Window and background mode each have a bit that determines which tilemap they will use.
        let tilemap_base_addr = if !self.fetcher.drawing_window && bg_tile_map
            || self.fetcher.drawing_window && window_tile_map
        {
            TILEMAP_2_ADDR
        } else {
            TILEMAP_1_ADDR
        };

        // WX is subtracted from LX because LX = WX (accounting for the offset of 7) should grab
        // the leftmost window tile from memory. LX = WX + 1 should grab the next, etc.
        // WY works in the same way, as the WY counter is externally keeping track of the window
        // tile's current y-position in memory. wy_counter = 0 will render the topmost window tile,
        // wy_counter = 1 will render the next, etc.
        (self.fetcher.x, self.fetcher.y) = if self.fetcher.drawing_window {
            let wx = self.read_byte(WX_ADDR).wrapping_sub(7);
            let wy = self.wy_counter;
            (self.lx.wrapping_sub(wx), wy)
        } else {
            let scx = self.read_byte(SCX_ADDR);
            let scy = self.read_byte(SCY_ADDR);
            (self.lx.wrapping_add(scx), self.ly.wrapping_add(scy))
        };

        self.fetcher.tile_x = self.fetcher.x / 8;
        self.fetcher.tile_y = self.fetcher.y / 8;

        let tilemap_addr = tilemap_base_addr
            + (self.fetcher.tile_y as u16 * TILEMAP_WIDTH)
            + (self.fetcher.tile_x) as u16;

        let tile_index = self.read_byte(tilemap_addr);

        self.fetcher.tile_addr = self.get_tile_start_addr(tile_index);
    }

    fn fetcher_get_tile_data(&mut self, high: bool) {
        let tile_start_addr = self.fetcher.tile_addr;
        let row_index = self.fetcher.y % TILE_HEIGHT_IN_PIXELS as u8;

        if high {
            self.fetcher.tile_data_high =
                self.read_byte(tile_start_addr + (row_index as u16 * 2) + 1);
        } else {
            self.fetcher.tile_data_low = self.read_byte(tile_start_addr + (row_index as u16 * 2));
        }
    }

    fn fetcher_sleep(&self) {}

    fn fetcher_push(&mut self) {
        let mut tile_row = get_tile_row(self.fetcher.tile_data_low, self.fetcher.tile_data_high);
        let row = self.ly as usize;
        let col = self.lx as usize;

        // If bg and window isn't enabled, the pixels are replaced with all 0
        let bg_and_window_enable = self.get_lcdc_flag(BG_AND_WINDOW_ENABLE_BIT);
        tile_row = if bg_and_window_enable {
            tile_row
        } else {
            [0; 8]
        };

        for (i, pixel) in tile_row.iter().enumerate() {
            self.display[row][col + i] = *pixel;
        }
    }

    /// WX = 7 starts rendering the window at the-left of the screen, so WX = 0 is one tile
    /// offscreen to the left. LX = 0, on the other hand, starts at the left of the screen as you
    /// would expect. This means that any time you compare the two, you need to either add 7 to LX
    /// or subtract 7 from WX to ensure that they are both measured from the same point.
    fn update_wx(&mut self) {
        let wx = self.read_byte(WX_ADDR);
        if (self.lx) == wx.wrapping_sub(7) {
            self.wx_triggered = true;
        }
    }
}