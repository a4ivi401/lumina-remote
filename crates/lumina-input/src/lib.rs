use enigo::{Enigo, KeyboardControllable, MouseButton, MouseControllable, Key};

/// Represents an abstract input event received from the network.
#[derive(Debug, Clone)]
pub enum InputEvent {
    MouseMove { x: i32, y: i32 },
    MouseDown { button: MouseButton },
    MouseUp { button: MouseButton },
    MouseClick { button: MouseButton },
    MouseScroll { x: i32, y: i32 },
    KeyDown { key: Key },
    KeyUp { key: Key },
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
                self.enigo.mouse_down(button);
            }
            InputEvent::MouseUp { button } => {
                self.enigo.mouse_up(button);
            }
            InputEvent::MouseClick { button } => {
                self.enigo.mouse_click(button);
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
                self.enigo.key_down(key);
            }
            InputEvent::KeyUp { key } => {
                self.enigo.key_up(key);
            }
            InputEvent::TypeString { text } => {
                self.enigo.key_sequence(&text);
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
