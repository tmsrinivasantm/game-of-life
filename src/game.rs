use raylib::{ffi::KeyboardKey, prelude::*};
use std::collections::{HashMap, HashSet, VecDeque};

mod events;

use events::{EventMetadata, KeyBoardEventHandler};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
struct Cell {
    x: i32,
    y: i32,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
enum CellStatus {
    Set,
    Unset,
}

#[derive(PartialEq, Eq)]
enum ObjectTypes {
    Rectangle,
    Line,
}

#[derive(PartialEq, Eq)]
struct RenderObject {
    pos: [i32; 4],
    object_type: ObjectTypes,
}

struct GameContext {
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,
    screen_height: i32,
    live_cells: HashSet<Cell>,
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
    keyboard_event_handler: KeyBoardEventHandler,
    render_objects: VecDeque<RenderObject>,
}

impl Game {
    fn set_grid(&mut self) {
        self.ctx.jump_x = self.ctx.grid_size / self.ctx.slices[0];
        self.ctx.jump_y = self.ctx.grid_size / self.ctx.slices[1];

        let mut pos = 0;
        for _ in 0..=self.ctx.slices[0] {
            self.render_objects.push_back(RenderObject {
                pos: [pos, 0, pos, self.ctx.grid_size],
                object_type: ObjectTypes::Line,
            });
            pos += self.ctx.jump_x;
        }

        pos = 0;
        for _ in 0..=self.ctx.slices[1] {
            self.render_objects.push_back(RenderObject {
                pos: [0, pos, self.ctx.grid_size, pos],
                object_type: ObjectTypes::Line,
            });
            pos += self.ctx.jump_y;
        }

        self.keyboard_event_handler.add_mouse_event(
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
                    ctx.live_cells.insert(cell);
                } else {
                    ctx.live_cells.remove(&cell);
                }
            },
        );
    }

    fn check_neighbours(&self, cell: &Cell, board: &Vec<CellStatus>) -> Option<CellStatus> {
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
            board[(c.x + c.y * self.ctx.slices[0]) as usize] == CellStatus::Set
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

        let status = board[(cell.x + cell.y * self.ctx.slices[0]) as usize];
        if status != CellStatus::Set {
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

    fn calculate(&mut self) {
        // mark for operation
        if !self.ctx.pause_play {
            return;
        }
        let mut board_vec: Vec<CellStatus> =
            vec![CellStatus::Unset; (self.ctx.slices[0] * self.ctx.slices[1]) as usize];
        for cell in &self.ctx.live_cells {
            board_vec[(cell.x + cell.y * self.ctx.slices[0]) as usize] = CellStatus::Set;
        }
        let mut visited: HashMap<Cell, CellStatus> = HashMap::new();
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
            if !visited.contains_key(&c) {
                if let Some(cell_status) = self.check_neighbours(&c, &board_vec) {
                    visited.insert(c, cell_status);
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
            .for_each(|&(dx, dy)| cover_neighbourhood(dx, dy, &cell));
        }
        // apply operation
        for (cell, cell_status) in visited {
            match cell_status {
                CellStatus::Set => {
                    self.ctx.live_cells.insert(cell);
                }
                CellStatus::Unset => {
                    self.ctx.live_cells.remove(&cell);
                }
            }
        }
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
                live_cells: HashSet::new(),
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
            keyboard_event_handler: KeyBoardEventHandler::new(),
            render_objects: VecDeque::with_capacity((slices[0] * slices[1]) as usize),
        }
    }
    fn update(&mut self) {
        for rectangle in &self.ctx.live_cells {
            self.render_objects.push_front(RenderObject {
                pos: [
                    rectangle.x * self.ctx.jump_x,
                    rectangle.y * self.ctx.jump_y,
                    self.ctx.jump_x,
                    self.ctx.jump_y,
                ],
                object_type: ObjectTypes::Rectangle,
            });
        }
    }
    fn render(&mut self) {
        let mut draw_handle = self
            .ctx
            .raylib_handle
            .begin_drawing(&self.ctx.raylib_thread);
        draw_handle.clear_background(Color::BLACK);

        loop {
            let Some(object) = self.render_objects.front() else {
                return;
            };
            match object.object_type {
                ObjectTypes::Rectangle => {
                    draw_handle.draw_rectangle(
                        object.pos[0],
                        object.pos[1],
                        object.pos[2], // jump_x
                        object.pos[3], // jump_y
                        Color::RED,
                    );
                    self.render_objects.pop_front();
                }
                ObjectTypes::Line => {
                    break;
                }
            }
        }
        for object in &self.render_objects {
            if matches!(object.object_type, ObjectTypes::Line) {
                draw_handle.draw_line(
                    object.pos[0],
                    object.pos[1],
                    object.pos[2],
                    object.pos[3], // screen_height
                    Color::WHITE,
                );
            }
        }
    }
    pub fn run(mut self) {
        self.keyboard_event_handler.add_keyboard_event(
            "pause-play",
            KeyboardKey::KEY_SPACE,
            move |ctx: &mut GameContext, _: EventMetadata| {
                ctx.pause_play = !ctx.pause_play;
            },
        );

        let mut frame_count = 0;

        self.set_grid();

        while !self.ctx.raylib_handle.window_should_close() {
            self.keyboard_event_handler
                .capture_keyboard_events(&mut self.ctx);
            self.keyboard_event_handler
                .capture_mouse_events(&mut self.ctx);

            if frame_count == self.ctx.fps / 6 {
                self.calculate();
                frame_count = 0;
            }
            // println!(
            //     "{:?}x{:?}",
            //     self.ctx.raylib_handle.get_screen_width(),
            //     self.ctx.raylib_handle.get_screen_height()
            // );

            self.update();
            self.render();
            frame_count += 1;
        }
    }
}
