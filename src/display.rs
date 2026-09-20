use embassy_time::{Duration, Instant};
use embedded_graphics::{
    geometry::Size,
    pixelcolor::BinaryColor,
    prelude::{DrawTarget, Drawable, Point, Primitive},
    primitives::{PrimitiveStyle, Rectangle},
};
use rmk::display::{DisplayRenderer, RenderContext};

const GRID_WIDTH: usize = 25;
const GRID_HEIGHT: usize = 8;
const CELL_PITCH_X: u32 = 5;
const CELL_PITCH_Y: u32 = 4;
const CELL_WIDTH: u32 = 4;
const CELL_HEIGHT: u32 = 3;
const GENERATION_INTERVAL: Duration = Duration::from_millis(100);

/// Conway's Game of Life for the Klor's 128x32 monochrome OLED.
///
/// The simulation uses a 25x8 grid of native 32-bit rows. Each live cell is
/// drawn as a 4x3 block to compensate for the panel's apparent pixel aspect
/// ratio, and each key press adds a new R-pentomino.
pub struct GameOfLifeRenderer {
    cells: [u32; GRID_HEIGHT],
    rng: u32,
    next_generation: Instant,
    pending_clusters: u16,
    first_frame: bool,
    sleeping: bool,
}

impl GameOfLifeRenderer {
    fn next_random(&mut self) -> u32 {
        // Xorshift32 is small, deterministic, and more than sufficient for
        // spreading new clusters around the display.
        let mut value = self.rng;
        value ^= value << 13;
        value ^= value >> 17;
        value ^= value << 5;
        self.rng = value;
        value
    }

    fn set_cell(&mut self, x: usize, y: usize) {
        self.cells[y % GRID_HEIGHT] |= 1u32 << (x % GRID_WIDTH);
    }

    fn add_cluster(&mut self) {
        // Keep the seed contiguous so a key press only dirties a 3x3 area of
        // the OLED framebuffer instead of wrapping across a screen boundary.
        let x = self.next_random() as usize % (GRID_WIDTH - 2);
        let y = self.next_random() as usize % (GRID_HEIGHT - 2);

        // R-pentomino: a compact seed with a long, chaotic evolution.
        self.set_cell(x + 1, y);
        self.set_cell(x + 2, y);
        self.set_cell(x, y + 1);
        self.set_cell(x + 1, y + 1);
        self.set_cell(x + 1, y + 2);
    }

    fn step(&mut self) {
        let current = self.cells;
        let mut next = [0u32; GRID_HEIGHT];

        for y in 0..GRID_HEIGHT {
            for x in 0..GRID_WIDTH {
                let mut neighbors = 0u8;

                for dy in [-1isize, 0, 1] {
                    for dx in [-1isize, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }

                        let nx = (x as isize + dx).rem_euclid(GRID_WIDTH as isize) as usize;
                        let ny = (y as isize + dy).rem_euclid(GRID_HEIGHT as isize) as usize;
                        neighbors += ((current[ny] >> nx) & 1) as u8;
                    }
                }

                let alive = ((current[y] >> x) & 1) != 0;
                if neighbors == 3 || (alive && neighbors == 2) {
                    next[y] |= 1u32 << x;
                }
            }
        }

        self.cells = next;
    }

    fn draw_frame<D: DrawTarget<Color = BinaryColor>>(&self, display: &mut D) {
        display.clear(BinaryColor::Off).ok();

        for y in 0..GRID_HEIGHT {
            let mut live = self.cells[y];

            while live != 0 {
                let x = live.trailing_zeros() as usize;

                Rectangle::new(
                    Point::new(
                        (x as u32 * CELL_PITCH_X) as i32,
                        (y as u32 * CELL_PITCH_Y) as i32,
                    ),
                    Size::new(CELL_WIDTH, CELL_HEIGHT),
                )
                .into_styled(PrimitiveStyle::with_fill(BinaryColor::On))
                .draw(display)
                .ok();
                live &= live - 1;
            }
        }
    }
}

impl Default for GameOfLifeRenderer {
    fn default() -> Self {
        let now = Instant::now();
        let mut renderer = Self {
            cells: [0; GRID_HEIGHT],
            rng: 0x4b4c_4f52,
            next_generation: now + GENERATION_INTERVAL,
            pending_clusters: 0,
            first_frame: true,
            sleeping: false,
        };

        // Start with a live board rather than waiting for the first key press.
        renderer.set_cell(GRID_WIDTH / 2, GRID_HEIGHT / 2 - 1);
        renderer.set_cell(GRID_WIDTH / 2 + 1, GRID_HEIGHT / 2);
        renderer.set_cell(GRID_WIDTH / 2 - 1, GRID_HEIGHT / 2 + 1);
        renderer.set_cell(GRID_WIDTH / 2, GRID_HEIGHT / 2 + 1);
        renderer.set_cell(GRID_WIDTH / 2 + 1, GRID_HEIGHT / 2 + 1);
        renderer
    }
}

impl DisplayRenderer<BinaryColor> for GameOfLifeRenderer {
    fn render<D: DrawTarget<Color = BinaryColor>>(
        &mut self,
        context: &RenderContext,
        display: &mut D,
    ) {
        if context.sleeping {
            if !self.sleeping {
                display.clear(BinaryColor::Off).ok();
                self.sleeping = true;
            }
            return;
        }

        let waking = self.sleeping;
        self.sleeping = false;

        if context.key_press_latch {
            self.pending_clusters = self.pending_clusters.saturating_add(1);
        }

        // Follow RMK's built-in renderer pattern: clear and draw a complete
        // frame, then return so DisplayProcessor can flush it. Key events after
        // the first frame only update `pending_clusters` and touch no pixels.
        if self.first_frame || waking {
            self.first_frame = false;
            self.draw_frame(display);
            return;
        }

        let now = Instant::now();
        if now < self.next_generation {
            return;
        }

        for _ in 0..self.pending_clusters {
            self.add_cluster();
        }
        self.pending_clusters = 0;

        self.step();
        // Keep a fixed cadence even if a keyboard event happens to trigger
        // this frame just before RMK's periodic display tick. Skip missed
        // deadlines rather than producing a burst of catch-up frames.
        while self.next_generation <= now {
            self.next_generation += GENERATION_INTERVAL;
        }

        self.draw_frame(display);
    }
}
