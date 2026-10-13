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

struct Gamecontext {
    render_objects: HashSet<Renderobject>,
    raylib_handle: RaylibHandle,
    raylib_thread: RaylibThread,
    screen_height: i32,
    rectanges: HashMap<Rectangle, CellStatus>,
    screen_width: i32,
    pause_play: bool,
    grid_size: i32,
}
pub struct Game {
    ctx: Gamecontext,
    keyboard_event_handler: KeyBoardEventHandler,
}

impl Game {
    fn set_grid(&mut self, slices: [i32; 2], screen_width: i32, grid_size: i32) {
        let mut pos = 0;
        let jump_x = screen_width / slices[0];
        let jump_y = grid_size / slices[1];
        self.ctx.render_objects.insert(Renderobject {
            pos: [pos, 0, pos, grid_size],
            object_type: Objecttypes::Line,
        });
        for _ in 0..slices[0] {
            pos += jump_x;
            self.ctx.render_objects.insert(Renderobject {
                pos: [pos, 0, pos, grid_size],
                object_type: Objecttypes::Line,
            });
        }
        pos = 0;
        self.ctx.render_objects.insert(Renderobject {
            pos: [0, pos, screen_width, pos],
            object_type: Objecttypes::Line,
        });
        for _ in 0..slices[1] {
            pos += jump_y;
            self.ctx.render_objects.insert(Renderobject {
                pos: [0, pos, screen_width, pos],
                object_type: Objecttypes::Line,
            });
        }

        self.keyboard_event_handler.add_mouse_event(
            "set-cell",
            MouseButton::MOUSE_BUTTON_LEFT,
            move |ctx: &mut Gamecontext, event_metadata: EventMetadata| {
                if event_metadata.mouse_pos_y > grid_size || event_metadata.mouse_pos_y < 0 {
                    return;
                }
                let rectx = (event_metadata.mouse_pos_x / jump_x) * jump_x;
                let recty = (event_metadata.mouse_pos_y / jump_y) * jump_y;
                let render_object = Renderobject {
                    pos: [rectx, recty, jump_x, jump_y],
                    object_type: Objecttypes::Rectangle,
                };
                if !ctx.render_objects.contains(&render_object) {
                    ctx.render_objects.insert(render_object);
                    ctx.rectanges.insert(
                        Rectangle {
                            x: rectx,
                            y: recty,
                            jump_x: jump_x,
                            jump_y: jump_y,
                        },
                        CellStatus::Set,
                    );
                } else {
                    ctx.render_objects.remove(&render_object);
                    ctx.rectanges.remove(&Rectangle {
                        x: rectx,
                        y: recty,
                        jump_x: jump_x,
                        jump_y: jump_y,
                    });
                }
            },
        );
    }

    fn check_neighbours(
        &self,
        rectangle: &Rectangle,
    ) -> CellStatus {
        let is_set = |dx: i32, dy: i32| {
            let mut rect = *rectangle;
            rect.x = rectangle.x + (dx * rectangle.jump_x);
            rect.y = rectangle.y + (dy * rectangle.jump_y);

            if rect.x >= 0 && rect.y >= 0 {
                self.ctx.rectanges.get(&rect) == Some(&CellStatus::Set)
            } else {
                false
            }
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

        match self.ctx.rectanges.get(rectangle) {
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
            r.x = rectangle.x + (dx * rectangle.jump_x);
            r.y = rectangle.y + (dy * rectangle.jump_y);
            if !visited.contains_key(&r) && r.x >= 0 && r.y >= 0 {
                visited.insert(r, self.check_neighbours(&r));
            }
        };
        for (rectangle, _) in &self.ctx.rectanges {
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
                    self.ctx.rectanges.insert(*rectangle, CellStatus::Set);
                }
                CellStatus::Markforremove => {
                    self.ctx.rectanges.insert(*rectangle, CellStatus::Unset);
                }
                _ => (),
            }
        }
    }

    pub fn new() -> Game {
        let screen_width: i32 = 1200;
        let grid_size: i32 = 1200;
        let screen_height: i32 = 1300;
        let (rl, thread) = raylib::init()
            .size(screen_width, screen_height)
            .msaa_4x()
            .vsync()
            .title("Game Of Life")
            .build();
        Game {
            ctx: Gamecontext {
                render_objects: HashSet::new(),
                rectanges: HashMap::new(),
                raylib_handle: rl,
                raylib_thread: thread,
                screen_height: screen_height,
                screen_width: screen_width,
                grid_size: grid_size,
                pause_play: false,
            },
            keyboard_event_handler: KeyBoardEventHandler::new(),
        }
    }
    fn render(render_objects: &HashSet<Renderobject>, draw_handle: &mut RaylibDrawHandle) {
        for object in render_objects {
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
    pub fn update(&mut self) {
        if !self.ctx.pause_play {
            return;
        }
        for (rectangle, cell_status) in &self.ctx.rectanges {
            match cell_status {
                CellStatus::Unset => {
                    self.ctx.render_objects.remove(&Renderobject {
                        pos: [rectangle.x, rectangle.y, rectangle.jump_x, rectangle.jump_y],
                        object_type: Objecttypes::Rectangle,
                    });
                }
                CellStatus::Set => {
                    self.ctx.render_objects.insert(Renderobject {
                        pos: [rectangle.x, rectangle.y, rectangle.jump_x, rectangle.jump_y],
                        object_type: Objecttypes::Rectangle,
                    });
                }
                _ => (),
            }
        }
    }
    pub fn run(mut self) {
        let screen_width: i32 = self.ctx.screen_width;
        let grid_size: i32 = self.ctx.grid_size;
        let fps = 60;
        let slices = [40, 40];
        self.ctx.raylib_handle.set_target_fps(fps);

        self.set_grid(slices, screen_width, grid_size);

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

            if frame_count == 30 {
                self.calculate();
                frame_count = 0;
            }

            self.update();
            let mut d = self
                .ctx
                .raylib_handle
                .begin_drawing(&self.ctx.raylib_thread);
            d.clear_background(Color::BLACK);
            Game::render(&self.ctx.render_objects, &mut d);
            frame_count += 1;
        }
    }
}
