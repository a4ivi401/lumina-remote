use enigo::{Enigo, KeyboardControllable, MouseButton, MouseControllable, Key};
use serde::{Serialize, Deserialize};

/// Represents an abstract input event received from the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputEvent {
    MouseMove { x: i32, y: i32 },
    MouseDown { button: String },
    MouseUp { button: String },
    MouseClick { button: String },
    MouseScroll { x: i32, y: i32 },
    KeyDown { key: String },
    KeyUp { key: String },
    TypeString { text: String },
}

/// Controller responsible for injecting simulated input events into the host OS.
pub struct InputController {
    enigo: Enigo,
}

impl InputController {
    /// Creates a new `InputController` utilizing the `enigo` crate.
    pub fn new() -> Self {
        Self {
            enigo: Enigo::new(),
        }
    }

    /// Processes an incoming input event and executes the corresponding OS-level injection.
    pub fn handle_event(&mut self, event: InputEvent) {
        match event {
            InputEvent::MouseMove { x, y } => {
                self.enigo.mouse_move_to(x, y);
            }
            InputEvent::MouseDown { button } => {
                self.enigo.mouse_down(Self::map_mouse(&button));
            }
            InputEvent::MouseUp { button } => {
                self.enigo.mouse_up(Self::map_mouse(&button));
            }
            InputEvent::MouseClick { button } => {
                self.enigo.mouse_click(Self::map_mouse(&button));
            }
            InputEvent::MouseScroll { x, y } => {
                if x != 0 {
                    self.enigo.mouse_scroll_x(x);
                }
                if y != 0 {
                    self.enigo.mouse_scroll_y(y);
                }
            }
            InputEvent::KeyDown { key } => {
                self.enigo.key_down(Self::map_key(&key));
            }
            InputEvent::KeyUp { key } => {
                self.enigo.key_up(Self::map_key(&key));
            }
            InputEvent::TypeString { text } => {
                self.enigo.key_sequence(&text);
            }
        }
    }

    fn map_mouse(btn: &str) -> MouseButton {
        match btn {
            "right" => MouseButton::Right,
            "middle" => MouseButton::Middle,
            _ => MouseButton::Left,
        }
    }

    fn map_key(k: &str) -> Key {
        match k {
            "return" | "enter" => Key::Return,
            "tab" => Key::Tab,
            "space" => Key::Space,
            "backspace" => Key::Backspace,
            "escape" => Key::Escape,
            "super" | "command" | "windows" => Key::Super,
            "command" => Key::Command,
            "shift" => Key::Shift,
            "capslock" => Key::CapsLock,
            "alt" | "option" => Key::Alt,
            "control" | "ctrl" => Key::Control,
            _ => {
                if k.len() == 1 {
                    Key::Layout(k.chars().next().unwrap())
                } else {
                    Key::Space // default fallback
                }
            }
        }
    }
}

impl Default for InputController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_controller_init() {
        // Just verify that Enigo can initialize properly on this OS
        let _controller = InputController::new();
    }
}
