use std::collections::HashMap;

use raylib::ffi::{KeyboardKey, MouseButton};

use super::GameContext;

pub struct EventMetadata {
    pub key_pressed: Option<KeyboardKey>,
    pub mouse_pressed: Option<MouseButton>,
    pub mouse_pos_x: i32,
    pub mouse_pos_y: i32,
}

pub struct KeyBoardEventHandler {
    listeners: HashMap<&'static str, Box<dyn FnMut(&mut GameContext, EventMetadata)>>,
    keyboard_fire_buffer: HashMap<KeyboardKey, Vec<&'static str>>,
    mouse_fire_buffer: HashMap<MouseButton, Vec<&'static str>>,
}

impl KeyBoardEventHandler {
    pub fn new() -> KeyBoardEventHandler {
        KeyBoardEventHandler {
            listeners: HashMap::new(),
            keyboard_fire_buffer: HashMap::new(),
            mouse_fire_buffer: HashMap::new(),
        }
    }
    pub fn add_keyboard_event<T>(&mut self, event_tag: &'static str, key: KeyboardKey, callback: T)
    where
        T: FnMut(&mut GameContext, EventMetadata) + 'static,
    {
        if self.listeners.contains_key(event_tag) {
            panic!("[ERROR]: Event handle already registered");
        }
        self.listeners.insert(event_tag, Box::new(callback));
        if let Some(thing) = self.keyboard_fire_buffer.get_mut(&key) {
            thing.push(event_tag);
            return;
        }
        self.keyboard_fire_buffer.insert(key, vec![event_tag]);
    }
    pub fn add_mouse_event<T>(&mut self, event_tag: &'static str, button: MouseButton, callback: T)
    where
        T: FnMut(&mut GameContext, EventMetadata) + 'static,
    {
        if self.listeners.contains_key(event_tag) {
            panic!("[ERROR]: Event handle already registered");
        }
        self.listeners.insert(event_tag, Box::new(callback));
        if let Some(thing) = self.mouse_fire_buffer.get_mut(&button) {
            thing.push(event_tag);
            return;
        }
        self.mouse_fire_buffer.insert(button, vec![event_tag]);
    }
    pub fn capture_keyboard_events(&mut self, ctx: &mut GameContext) {
        let Some(key_pressed) = ctx.raylib_handle.get_key_pressed() else {
            return;
        };
        let Some(buffer) = self.keyboard_fire_buffer.get(&key_pressed) else {
            return;
        };
        for tag in buffer {
            let Some(callback) = self.listeners.get_mut(tag) else {
                continue;
            };
            callback(
                ctx,
                EventMetadata {
                    key_pressed: Some(key_pressed),
                    mouse_pressed: None,
                    mouse_pos_x: ctx.raylib_handle.get_mouse_x(),
                    mouse_pos_y: ctx.raylib_handle.get_mouse_y(),
                },
            );
        }
    }
    pub fn capture_mouse_events(&mut self, ctx: &mut GameContext) {
        if !ctx
            .raylib_handle
            .is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT)
        {
            return;
        };
        let mouse_pressed = MouseButton::MOUSE_BUTTON_LEFT;
        let Some(buffer) = self.mouse_fire_buffer.get(&mouse_pressed) else {
            return;
        };
        for tag in buffer {
            let Some(callback) = self.listeners.get_mut(tag) else {
                continue;
            };
            callback(
                ctx,
                EventMetadata {
                    key_pressed: None,
                    mouse_pressed: Some(mouse_pressed),
                    mouse_pos_x: ctx.raylib_handle.get_mouse_x(),
                    mouse_pos_y: ctx.raylib_handle.get_mouse_y(),
                },
            );
        }
    }
}
