use glutin::event::VirtualKeyCode;
use std::collections::HashMap;

#[derive(Debug, Default)]
struct KeyState {
    pressed: bool
}

#[derive(Debug, Default)]
pub struct Keyboard {
    states: HashMap<VirtualKeyCode, KeyState>
}

impl Keyboard {
    pub fn press(&mut self, keycode: VirtualKeyCode) {
        self.states.entry(keycode).or_default().pressed = true;
    }

    pub fn release(&mut self, keycode: VirtualKeyCode) {
        self.states.entry(keycode).or_default().pressed = false;
    }

    pub fn pressed(&self, keycode: VirtualKeyCode) -> bool {
        if let Some(state) = self.states.get(&keycode) {
            return state.pressed
        }

        return false
    }
}
