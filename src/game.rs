use raylib::{ffi::KeyboardKey, prelude::*};
use std::collections::{HashMap, HashSet};

mod events;

use events::{EventMetadata, KeyBoardEventHandler};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
struct Rectangle {
    x: i32,
    y: i32,
    jump_x: i32,
    jump_y: i32,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
enum CellStatus {
    Set,
    Unset,
    Markforset,
    Markforremove,
}

#[derive(PartialEq, Eq, Hash)]
enum Objecttypes {
    Rectangle,
    Line,
}

#[derive(PartialEq, Eq, Hash)]
struct Renderobject {
    pos: [i32; 4],
    object_type: Objecttypes,
}

#[allow(unused)]
struct Gamecontext {
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,
    screen_height: i32,
    rectangles: HashMap<Rectangle, CellStatus>,
    screen_width: i32,
    pause_play: bool,
    grid_size: i32,
    fps: u32,
}
pub struct Game {
    ctx: Gamecontext,
    keyboard_event_handler: KeyBoardEventHandler,
    render_objects: HashSet<Renderobject>,
}

impl Game {
    fn set_grid(&mut self, slices: [i32; 2]) {
        let mut pos = 0;
        let jump_x = self.ctx.screen_width / slices[0];
        let jump_y = self.ctx.grid_size / slices[1];
        self.render_objects.insert(Renderobject {
            pos: [pos, 0, pos, self.ctx.grid_size],
            object_type: Objecttypes::Line,
        });
        for _ in 0..slices[0] {
            pos += jump_x;
            self.render_objects.insert(Renderobject {
                pos: [pos, 0, pos, self.ctx.grid_size],
                object_type: Objecttypes::Line,
            });
        }
        pos = 0;
        self.render_objects.insert(Renderobject {
            pos: [0, pos, self.ctx.screen_width, pos],
            object_type: Objecttypes::Line,
        });
        for _ in 0..slices[1] {
            pos += jump_y;
            self.render_objects.insert(Renderobject {
                pos: [0, pos, self.ctx.screen_width, pos],
                object_type: Objecttypes::Line,
            });
        }

        self.keyboard_event_handler.add_mouse_event(
            "set-cell",
            MouseButton::MOUSE_BUTTON_LEFT,
            move |ctx: &mut Gamecontext, event_metadata: EventMetadata| {
                if event_metadata.mouse_pos_y > ctx.grid_size || event_metadata.mouse_pos_y < 0 {
                    return;
                }
                let rectx = (event_metadata.mouse_pos_x / jump_x) * jump_x;
                let recty = (event_metadata.mouse_pos_y / jump_y) * jump_y;
                let rectangle = Rectangle {
                    x: rectx,
                    y: recty,
                    jump_x: jump_x,
                    jump_y: jump_y,
                };
                let Some(cell_status) = ctx.rectangles.get(&rectangle) else {
                    ctx.rectangles.insert(
                        Rectangle {
                            x: rectx,
                            y: recty,
                            jump_x: jump_x,
                            jump_y: jump_y,
                        },
                        CellStatus::Set,
                    );
                    return;
                };
                if matches!(cell_status, CellStatus::Unset) {
                    ctx.rectangles.insert(
                        Rectangle {
                            x: rectx,
                            y: recty,
                            jump_x: jump_x,
                            jump_y: jump_y,
                        },
                        CellStatus::Set,
                    );
                } else if matches!(cell_status, CellStatus::Set) {
                    ctx.rectangles.insert(
                        Rectangle {
                            x: rectx,
                            y: recty,
                            jump_x: jump_x,
                            jump_y: jump_y,
                        },
                        CellStatus::Unset,
                    );
                }
            },
        );
    }

    fn check_neighbours(&self, rectangle: &Rectangle) -> CellStatus {
        let is_set = |dx: i32, dy: i32| {
            let mut rect = *rectangle;
            rect.x = (rectangle.x + (dx * rectangle.jump_x)) % self.ctx.grid_size;
            rect.y = (rectangle.y + (dy * rectangle.jump_y)) % self.ctx.grid_size;

            if rect.x < 0  {
                rect.x = self.ctx.grid_size + rect.x;
            }
            if rect.y < 0  {
                rect.y = self.ctx.grid_size + rect.y;
            }
            self.ctx.rectangles.get(&rect) == Some(&CellStatus::Set)
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

        match self.ctx.rectangles.get(rectangle) {
            Some(&status) => {
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
            None => {
                if cell_count == 3 {
                    CellStatus::Markforset
                } else {
                    CellStatus::Unset
                }
            }
        }
    }

    fn calculate(&mut self) {
        // mark for operation
        if !self.ctx.pause_play {
            return;
        }
        let mut visited: HashMap<Rectangle, CellStatus> = HashMap::new();
        let mut cover_neighbourhood = |dx: i32, dy: i32, rectangle: &Rectangle| {
            let mut r = *rectangle;
            r.x = (rectangle.x + (dx * rectangle.jump_x)) % self.ctx.grid_size;
            r.y = (rectangle.y + (dy * rectangle.jump_y)) % self.ctx.grid_size;
            if r.x < 0  {
                r.x = self.ctx.grid_size + r.x;
            }
            if r.y < 0  {
                r.y = self.ctx.grid_size + r.y;
            }
            if !visited.contains_key(&r) {
                visited.insert(r, self.check_neighbours(&r));
            }
        };
        for (rectangle, cell_status) in &self.ctx.rectangles {
            if matches!(cell_status, CellStatus::Unset) {
                continue;
            }
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
                    self.ctx.rectangles.insert(*rectangle, CellStatus::Set);
                }
                CellStatus::Markforremove => {
                    self.ctx.rectangles.insert(*rectangle, CellStatus::Unset);
                }
                _ => (),
            }
        }
    }

    pub fn new() -> Game {
        let screen_width: i32 = 1200;
        let grid_size: i32 = 1200;
        let screen_height: i32 = 1300;
        let fps = 60u32;
        let (mut rl, thread) = raylib::init()
            .size(screen_width, screen_height)
            .msaa_4x()
            .vsync()
            .title("Game Of Life")
            .build();
        rl.set_target_fps(fps);
        Game {
            ctx: Gamecontext {
                rectangles: HashMap::new(),
                raylib_handle: rl,
                raylib_thread: thread,
                screen_height: screen_height,
                screen_width: screen_width,
                grid_size: grid_size,
                pause_play: false,
                fps: fps,
            },
            keyboard_event_handler: KeyBoardEventHandler::new(),
            render_objects: HashSet::new(),
        }
    }
    fn render(&mut self) {
        let mut draw_handle = self
            .ctx
            .raylib_handle
            .begin_drawing(&self.ctx.raylib_thread);
        draw_handle.clear_background(Color::BLACK);

        for object in &self.render_objects {
            match object.object_type {
                Objecttypes::Rectangle => {
                    draw_handle.draw_rectangle(
                        object.pos[0],
                        object.pos[1],
                        object.pos[2], // jump_x
                        object.pos[3], // jump_y
                        Color::RED,
                    );
                }
                Objecttypes::Line => {
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
        for (rectangle, cell_status) in &self.ctx.rectangles {
            match cell_status {
                CellStatus::Unset => {
                    self.render_objects.remove(&Renderobject {
                        pos: [rectangle.x, rectangle.y, rectangle.jump_x, rectangle.jump_y],
                        object_type: Objecttypes::Rectangle,
                    });
                }
                CellStatus::Set => {
                    self.render_objects.insert(Renderobject {
                        pos: [rectangle.x, rectangle.y, rectangle.jump_x, rectangle.jump_y],
                        object_type: Objecttypes::Rectangle,
                    });
                }
                _ => (),
            }
        }
    }
    pub fn run(mut self) {
        let slices = [40, 40];

        self.set_grid(slices);

        self.keyboard_event_handler.add_keyboard_event(
            "pause-play",
            KeyboardKey::KEY_SPACE,
            move |ctx: &mut Gamecontext, _: EventMetadata| {
                ctx.pause_play = !ctx.pause_play;
            },
        );

        let mut frame_count = 0;

        while !self.ctx.raylib_handle.window_should_close() {
            self.keyboard_event_handler
                .capture_keyboard_events(&mut self.ctx);
            self.keyboard_event_handler
                .capture_mouse_events(&mut self.ctx);

            if frame_count == self.ctx.fps / 6 {
                self.calculate();
                frame_count = 0;
            }

            self.update();
            self.render();
            frame_count += 1;
        }
    }
}
