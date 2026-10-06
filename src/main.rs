use std::collections::HashMap;

use raylib::prelude::*;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
struct Rectangle {
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

fn main() {
    let screen_width: i32 = 1200;
    let screen_height: i32 = 1200;
    let fps = 60;
    let slices = [40, 40];
    let jump_x = screen_width / slices[0];
    let jump_y = screen_height / slices[1];
    let mut rectangles: HashMap<Rectangle, CellStatus> = HashMap::new();
    let (mut rl, thread) = raylib::init()
        .size(screen_width, screen_height)
        .msaa_4x()
        .vsync()
        .title("Game Of Life")
        .build();
    rl.set_target_fps(fps);

    let mut count = 0u8;
    let mut flag = 0;
    let mut initial_rectangles: HashMap<Rectangle, CellStatus> = HashMap::new();
    initial_rectangles.clear();
    while !rl.window_should_close() {
        if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
            let mouse_position = rl.get_mouse_position();
            let rectangle = Rectangle {
                x: (mouse_position.x as i32 / jump_x) * jump_x,
                y: (mouse_position.y as i32 / jump_y) * jump_y,
            };
            match rectangles.get(&rectangle) {
                Some(_) => {
                    rectangles.remove(&rectangle);
                }
                None => {
                    rectangles.insert(rectangle, CellStatus::Set);
                }
            };
        }
        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            if flag == 0 {
                initial_rectangles = rectangles.clone();
                flag = 1;
            } else {
                flag = 0;
            }
        }
        if rl.is_key_pressed(KeyboardKey::KEY_R) {
            rectangles.clear();
            rectangles = initial_rectangles.clone();
        }
        if flag == 1 {
            count += 1;
            if count == 20 && flag == 1 {
                calculate(&mut rectangles, jump_x, jump_y);
                count = 0;
            }
        }
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::get_color(0x181818FF));
        draw_grid(&mut d, slices, screen_width, screen_height);
        for rectangle in rectangles.keys() {
            d.draw_rectangle(rectangle.x, rectangle.y, jump_x, jump_y, Color::RED);
        }
    }
    todo!("Add UI");
}

fn draw_grid(
    draw_handle: &mut RaylibDrawHandle,
    slices: [i32; 2],
    screen_width: i32,
    screen_height: i32,
) {
    let mut pos = 0;
    let jump_x = screen_width / slices[0];
    let jump_y = screen_height / slices[1];
    draw_handle.draw_line(pos, 0, pos, screen_height, Color::WHITE);
    for _ in 0..slices[0] {
        pos += jump_x;
        draw_handle.draw_line(pos, 0, pos, screen_height, Color::WHITE);
    }
    pos = 0;
    draw_handle.draw_line(0, pos, screen_width, pos, Color::WHITE);
    for _ in 0..slices[1] {
        pos += jump_y;
        draw_handle.draw_line(0, pos, screen_width, pos, Color::WHITE);
    }
}

fn check_neighbours(
    rectangle: &Rectangle,
    rectangles: &HashMap<Rectangle, CellStatus>,
    jump_x: i32,
    jump_y: i32,
) -> CellStatus {
    let is_set = |dx: i32, dy: i32| {
        let mut rect = *rectangle;
        rect.x = rectangle.x + (dx * jump_x);
        rect.y = rectangle.y + (dy * jump_y);

        if rect.x >= 0 && rect.y >= 0 {
            rectangles.get(&rect) == Some(&CellStatus::Set)
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

    match rectangles.get(&rectangle) {
        Some(&status) => {
            if status != CellStatus::Set {
                return CellStatus::Unset;
            } else {
                if cell_count == 3 || cell_count == 2 {
                    return CellStatus::Set;
                } else {
                    return CellStatus::Markforremove;
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

fn calculate(rectangles: &mut HashMap<Rectangle, CellStatus>, jump_x: i32, jump_y: i32) {
    // mark for operation
    let mut visited: HashMap<Rectangle, CellStatus> = HashMap::new();
    for rectangle in rectangles.keys() {
        let mut cover_neighbourhood = |dx: i32, dy: i32| {
            let mut r = *rectangle;
            r.x = rectangle.x + (dx * jump_x);
            r.y = rectangle.y + (dy * jump_y);
            if visited.get(&r).is_none() && r.x >= 0 && r.y >= 0 {
                visited.insert(r, check_neighbours(&r, rectangles, jump_x, jump_y));
            }
        };
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
        .for_each(|&(dx, dy)| cover_neighbourhood(dx, dy));
    }
    // apply operation
    for rectangle in visited.keys() {
        let cell_status = visited.get(rectangle).unwrap_or(&CellStatus::Unset);
        match cell_status {
            CellStatus::Markforset => {
                rectangles.insert(*rectangle, CellStatus::Set);
            }
            CellStatus::Markforremove => {
                rectangles.remove(rectangle);
            }
            _ => (),
        }
    }
}
