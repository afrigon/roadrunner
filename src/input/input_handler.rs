use glutin::event::{KeyboardInput, VirtualKeyCode, ElementState};

use crate::input::{CursorDelta, CursorHandler, Keyboard};
use crate::input::axis::{Axis, KeyboardAxis};
use std::collections::HashMap;

#[derive(Default)]
pub struct InputHandler {
    keyboard: Keyboard,
    cursor: CursorHandler,
    axes: HashMap<String, Vec<Box<dyn Axis>>>
}

impl <'a>InputHandler {

    pub fn new() -> Self {
        let mut i = Self::default();
        i.register_axis("z", KeyboardAxis::new(VirtualKeyCode::W, VirtualKeyCode::S));
        i.register_axis("x", KeyboardAxis::new(VirtualKeyCode::D, VirtualKeyCode::A));
        i
    }

    pub fn process_keyboard(
        &mut self,
        KeyboardInput {
            virtual_keycode,
            state,
            ..
        }: KeyboardInput
    ) {

        if let Some(keycode) = virtual_keycode {
            match state {
                ElementState::Pressed => { self.keyboard.press(keycode) }
                ElementState::Released => { self.keyboard.release(keycode) }
            };
        };
    }

    pub fn process_cursor(&mut self, input: (f64, f64)) {
        self.cursor.process(input)
    }

    pub fn key_pressed(&self, keycode: VirtualKeyCode) -> bool {
        self.keyboard.pressed(keycode)
    }

    pub fn get_cursor_delta(&self) -> &CursorDelta {
        self.cursor.get_delta()
    }

    pub fn clear(&mut self) {
        self.cursor.clear();
    }

    pub fn register_axis<T : 'static + Axis>(&mut self, name: &str, axis: T) {
        self.axes.entry(name.to_string()).or_default().push(Box::new(axis))
    }

    pub fn get_axis(&self, name: &str) -> f32 {

        let mut val = 0.0;

        if let Some(axes) = self.axes.get(name) {
            for axis in axes {
                val += axis.get_value(self);
            }
        }

        return val
    }
}
