use raylib::{ffi::KeyboardKey, prelude::*};

mod events;

use events::{EventHandler, EventMetadata};

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
struct Cell {
    x: i32,
    y: i32,
}

struct Line {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
enum CellStatus {
    Set,
    Unset,
}

struct GameContext {
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,
    screen_height: i32,
    live_cells: Vec<Cell>,
    screen_width: i32,
    pause_play: bool,
    grid_size: i32,
    jump_x: i32,
    jump_y: i32,
    slices: [i32; 2],
    fps: u32,
}
pub struct Game {
    ctx: GameContext,
    event_handler: EventHandler,
    grid_cache: Vec<Line>,
}

impl Game {
    fn set_grid(&mut self) {
        self.ctx.jump_x = self.ctx.grid_size / self.ctx.slices[0];
        self.ctx.jump_y = self.ctx.grid_size / self.ctx.slices[1];

        let mut pos = 0;
        for _ in 0..=self.ctx.slices[0] {
            self.grid_cache.push(Line {
                x1: pos,
                y1: 0,
                x2: pos,
                y2: self.ctx.grid_size,
            });
            pos += self.ctx.jump_x;
        }

        pos = 0;
        for _ in 0..=self.ctx.slices[1] {
            self.grid_cache.push(Line {
                x1: 0,
                y1: pos,
                x2: self.ctx.grid_size,
                y2: pos,
            });
            pos += self.ctx.jump_y;
        }

        self.event_handler.add_mouse_event(
            "set-cell",
            MouseButton::MOUSE_BUTTON_LEFT,
            move |ctx: &mut GameContext, event_metadata: EventMetadata| {
                if event_metadata.mouse_pos_y > ctx.grid_size || event_metadata.mouse_pos_y < 0 {
                    return;
                }
                let rectx = (event_metadata.mouse_pos_x / ctx.jump_x) * ctx.jump_x;
                let recty = (event_metadata.mouse_pos_y / ctx.jump_y) * ctx.jump_y;
                let cell = Cell {
                    x: rectx / ctx.jump_x,
                    y: recty / ctx.jump_y,
                };
                if !ctx.live_cells.contains(&cell) {
                    ctx.live_cells.push(cell);
                } else {
                    if let Some(position) = ctx.live_cells.iter().position(|x| *x == cell) {
                        ctx.live_cells.remove(position);
                    }
                }
            },
        );
    }

    fn check_neighbours(&self, cell: &Cell, board: &Vec<bool>) -> Option<CellStatus> {
        let is_set = |dx: i32, dy: i32| {
            let mut c = *cell;
            c.x = (c.x + dx) % self.ctx.slices[0];
            c.y = (c.y + dy) % self.ctx.slices[1];

            if c.x < 0 {
                c.x = self.ctx.slices[0] + c.x;
            }
            if c.y < 0 {
                c.y = self.ctx.slices[1] + c.y;
            }
            board[(c.x + c.y * self.ctx.slices[0]) as usize]
        };

        let cell_count = [
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
            (-1, -1),
            (-1, 0),
            (-1, 1),
        ]
        .iter()
        .filter(|&&(dx, dy)| is_set(dx, dy))
        .count() as u8;

        if !board[(cell.x + cell.y * self.ctx.slices[0]) as usize] {
            if cell_count == 3 {
                Some(CellStatus::Set)
            } else {
                None
            }
        } else {
            if cell_count == 3 || cell_count == 2 {
                Some(CellStatus::Set)
            } else {
                Some(CellStatus::Unset)
            }
        }
    }

    fn calculate(
        &mut self,
        board_vec: &mut Vec<bool>,
        visited: &mut Vec<bool>,
        new_live_cells: &mut Vec<Cell>,
    ) {
        // mark for operation
        if !self.ctx.pause_play {
            return;
        }

        board_vec.fill(false);
        visited.fill(false);
        new_live_cells.clear();

        for cell in &self.ctx.live_cells {
            board_vec[(cell.x + cell.y * self.ctx.slices[0]) as usize] = true;
        }
        let mut cover_neighbourhood = |dx: i32, dy: i32, cell: &Cell| {
            let mut c = *cell;
            c.x = (c.x + dx) % self.ctx.slices[0];
            c.y = (c.y + dy) % self.ctx.slices[1];
            if c.x < 0 {
                c.x = self.ctx.slices[0] + c.x;
            }
            if c.y < 0 {
                c.y = self.ctx.slices[1] + c.y;
            }
            if !visited[(c.x + c.y * self.ctx.slices[0]) as usize] {
                if let Some(cell_status) = self.check_neighbours(&c, &board_vec) {
                    if matches!(cell_status, CellStatus::Set) {
                        new_live_cells.push(c);
                    }
                    visited[(c.x + c.y * self.ctx.slices[0]) as usize] = true;
                }
            }
        };
        for cell in &self.ctx.live_cells {
            [
                (0, -1),
                (0, 1),
                (0, 0),
                (1, -1),
                (1, 0),
                (1, 1),
                (-1, -1),
                (-1, 0),
                (-1, 1),
            ]
            .iter()
            .for_each(|&(dx, dy)| cover_neighbourhood(dx, dy, cell));
        }
        // apply operation

        std::mem::swap(&mut self.ctx.live_cells, new_live_cells);
    }

    pub fn new() -> Game {
        let screen_width: i32 = 800;
        let grid_size: i32 = screen_width;
        let screen_height: i32 = grid_size + 100;
        let fps = 60u32;
        let slices = [40, 40];
        let (mut rl, thread) = raylib::init()
            .size(screen_width, screen_height)
            .msaa_4x()
            .vsync()
            .title("Game Of Life")
            .borderless_windowed_mode()
            .build();
        rl.set_target_fps(fps);
        Game {
            ctx: GameContext {
                live_cells: Vec::new(),
                raylib_handle: rl,
                raylib_thread: thread,
                screen_height: screen_height,
                screen_width: screen_width,
                grid_size: grid_size,
                pause_play: false,
                jump_x: 0,
                jump_y: 0,
                slices: slices,
                fps: fps,
            },
            event_handler: EventHandler::new(),
            grid_cache: Vec::new(),
        }
    }
    fn render(&mut self) {
        let mut draw_handle = self
            .ctx
            .raylib_handle
            .begin_drawing(&self.ctx.raylib_thread);
        draw_handle.clear_background(Color::BLACK);

        for cells in &self.ctx.live_cells {
            draw_handle.draw_rectangle(
                cells.x * self.ctx.jump_x,
                cells.y * self.ctx.jump_y,
                self.ctx.jump_x, // jump_x
                self.ctx.jump_y, // jump_y
                Color::RED,
            );
        }

        for object in &self.grid_cache {
            draw_handle.draw_line(
                object.x1,
                object.y1,
                object.x2,
                object.y2, // screen_height
                Color::WHITE,
            );
        }
    }
    pub fn run(mut self) {
        self.event_handler.add_keyboard_event(
            "pause-play",
            KeyboardKey::KEY_SPACE,
            move |ctx: &mut GameContext, _: EventMetadata| {
                ctx.pause_play = !ctx.pause_play;
            },
        );

        let mut board_vec: Vec<bool> =
            vec![false; (self.ctx.slices[0] * self.ctx.slices[1]) as usize];
        let mut visited: Vec<bool> =
            vec![false; (self.ctx.slices[0] * self.ctx.slices[1]) as usize];
        let mut new_live_cells: Vec<Cell> = Vec::new();

        let mut frame_count = 0;

        self.set_grid();

        while !self.ctx.raylib_handle.window_should_close() {
            self.event_handler.capture_keyboard_events(&mut self.ctx);
            self.event_handler.capture_mouse_events(&mut self.ctx);

            if frame_count == self.ctx.fps / 6 {
                self.calculate(&mut board_vec, &mut visited, &mut new_live_cells);
                frame_count = 0;
            }
            // println!(
            //     "{:?}x{:?}",
            //     self.ctx.raylib_handle.get_screen_width(),
            //     self.ctx.raylib_handle.get_screen_height()
            // );

            self.render();
            frame_count += 1;
        }
    }
}
