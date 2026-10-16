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
    Markforset,
    Markforremove,
}

#[derive(PartialEq, Eq, Hash)]
enum ObjectTypes {
    Rectangle,
    Line,
}

#[derive(PartialEq, Eq, Hash)]
struct RenderObject {
    pos: [i32; 4],
    object_type: ObjectTypes,
}

#[allow(unused)]
struct GameContext {
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,
    screen_height: i32,
    board: Vec<CellStatus>,
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
        self.ctx.jump_x = self.ctx.screen_width / self.ctx.slices[0];
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
                pos: [0, pos, self.ctx.screen_width, pos],
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
                let cell_status = ctx.board
                    [((rectx / ctx.jump_x) + (recty / ctx.jump_y) * ctx.slices[0]) as usize];
                if matches!(cell_status, CellStatus::Unset) {
                    ctx.board
                        [((rectx / ctx.jump_x) + (recty / ctx.jump_y) * ctx.slices[0]) as usize] =
                        CellStatus::Set;
                    ctx.live_cells.insert(Cell { x: rectx, y: recty });
                } else if matches!(cell_status, CellStatus::Set) {
                    ctx.board
                        [((rectx / ctx.jump_x) + (recty / ctx.jump_y) * ctx.slices[0]) as usize] =
                        CellStatus::Unset;
                    ctx.live_cells.remove(&Cell { x: rectx, y: recty });
                }
            },
        );
    }

    fn check_neighbours(&self, rectangle: &Cell) -> CellStatus {
        let is_set = |dx: i32, dy: i32| {
            let mut rect = *rectangle;
            rect.x = (rectangle.x + (dx * self.ctx.jump_x)) % self.ctx.grid_size;
            rect.y = (rectangle.y + (dy * self.ctx.jump_y)) % self.ctx.grid_size;

            if rect.x < 0 {
                rect.x = self.ctx.grid_size + rect.x;
            }
            if rect.y < 0 {
                rect.y = self.ctx.grid_size + rect.y;
            }
            self.ctx.board[(rect.x / self.ctx.jump_x
                + rect.y / self.ctx.jump_y * self.ctx.slices[0])
                as usize]
                == CellStatus::Set
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

        let status = self.ctx.board[(rectangle.x / self.ctx.jump_x
            + rectangle.y / self.ctx.jump_y * self.ctx.slices[0])
            as usize];
        if status != CellStatus::Set {
            if cell_count == 3 {
                CellStatus::Markforset
            } else {
                CellStatus::Unset
            }
        } else {
            if cell_count == 3 || cell_count == 2 {
                CellStatus::Set
            } else {
                if status != CellStatus::Set {
                    CellStatus::Unset
                } else {
                    CellStatus::Markforremove
                }
            }
        }
    }

    fn calculate(&mut self) {
        // mark for operation
        if !self.ctx.pause_play {
            return;
        }
        let mut visited: HashMap<Cell, CellStatus> = HashMap::new();
        let mut cover_neighbourhood = |dx: i32, dy: i32, rectangle: &Cell| {
            let mut r = *rectangle;
            r.x = (rectangle.x + (dx * self.ctx.jump_x)) % self.ctx.grid_size;
            r.y = (rectangle.y + (dy * self.ctx.jump_y)) % self.ctx.grid_size;
            if r.x < 0 {
                r.x = self.ctx.grid_size + r.x;
            }
            if r.y < 0 {
                r.y = self.ctx.grid_size + r.y;
            }
            if !visited.contains_key(&r) {
                visited.insert(r, self.check_neighbours(&r));
            }
        };
        for rectangle in &self.ctx.live_cells {
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
            .for_each(|&(dx, dy)| cover_neighbourhood(dx, dy, &rectangle));
        }
        // apply operation
        for rectangle in visited.keys() {
            let cell_status = visited.get(rectangle).unwrap_or(&CellStatus::Unset);
            match cell_status {
                CellStatus::Markforset => {
                    self.ctx.board[(rectangle.x / self.ctx.jump_x
                        + rectangle.y / self.ctx.jump_y * self.ctx.slices[0])
                        as usize] = CellStatus::Set;
                    self.ctx.live_cells.insert(*rectangle);
                }
                CellStatus::Markforremove => {
                    self.ctx.board[(rectangle.x / self.ctx.jump_x
                        + rectangle.y / self.ctx.jump_y * self.ctx.slices[0])
                        as usize] = CellStatus::Unset;
                    self.ctx.live_cells.remove(rectangle);
                }
                _ => (),
            }
        }
    }

    pub fn new() -> Game {
        let screen_width: i32 = 800;
        let grid_size: i32 = screen_width;
        let screen_height: i32 = grid_size + 100;
        let fps = 60u32;
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
                board: vec![CellStatus::Unset; 1600],
                live_cells: HashSet::new(),
                raylib_handle: rl,
                raylib_thread: thread,
                screen_height: screen_height,
                screen_width: screen_width,
                grid_size: grid_size,
                pause_play: false,
                jump_x: 0,
                jump_y: 0,
                slices: [40, 40],
                fps: fps,
            },
            keyboard_event_handler: KeyBoardEventHandler::new(),
            render_objects: VecDeque::with_capacity(1600),
        }
    }
    fn render(&mut self) {
        // println!("{:?}", self.render_objects.capacity());
        let mut draw_handle = self
            .ctx
            .raylib_handle
            .begin_drawing(&self.ctx.raylib_thread);
        draw_handle.clear_background(Color::BLACK);

        loop {
            let Some(object) = self.render_objects.pop_front() else {
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
                }
                ObjectTypes::Line => {
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
    }
    fn update(&mut self) {
        for rectangle in &self.ctx.live_cells {
            self.render_objects.push_back(RenderObject {
                pos: [rectangle.x, rectangle.y, self.ctx.jump_x, self.ctx.jump_y],
                object_type: ObjectTypes::Rectangle,
            });
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

        while !self.ctx.raylib_handle.window_should_close() {
            self.set_grid();

            self.keyboard_event_handler
                .capture_keyboard_events(&mut self.ctx);
            self.keyboard_event_handler
                .capture_mouse_events(&mut self.ctx);

            if frame_count == self.ctx.fps / 6 {
                self.calculate();
                frame_count = 0;
            }

            // println!("{:?}x{:?}", self.ctx.raylib_handle.get_screen_width(), self.ctx.raylib_handle.get_screen_height());

            self.update();
            self.render();
            frame_count += 1;
        }
    }
}
