use minifb::{Window, WindowOptions, Key};
use raqote::DrawTarget;

pub struct ViewerHandler {
    window: Window,
    buffer: Vec<u32>,
    width: usize,
    height: usize,
    previous_keys: Vec<Key>,
}

impl ViewerHandler {
    pub fn new(width: usize, height: usize, title: &str) -> Self {
        let mut window = Window::new(
            title,
            width,
            height,
            WindowOptions {
                resize: true,
                scale: minifb::Scale::X1,
                ..WindowOptions::default()
            },
        )
        .expect("Failed to create window");

        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        ViewerHandler {
            window,
            buffer: vec![0; width * height],
            width,
            height,
            previous_keys: Vec::new(),
        }
    }

    pub fn is_open(&self) -> bool {
        self.window.is_open() && !self.window.is_key_down(Key::Escape)
    }

    pub fn is_key_down(&self, key: Key) -> bool {
        self.window.is_key_down(key)
    }

    pub fn is_key_pressed(&mut self, key: Key) -> bool {
        let is_down = self.window.is_key_down(key);
        let was_down = self.previous_keys.contains(&key);
        let is_pressed = is_down && !was_down;
        
        // Update key state
        if is_down && !was_down {
            self.previous_keys.push(key);
        } else if !is_down && was_down {
            if let Some(pos) = self.previous_keys.iter().position(|&k| k == key) {
                self.previous_keys.remove(pos);
            }
        }
        
        is_pressed
    }

    pub fn update(&mut self, draw_target: &DrawTarget) -> bool {
        let pixels = draw_target.get_data();
        self.buffer.copy_from_slice(pixels);
        self.window.update_with_buffer(&self.buffer, self.width, self.height).is_ok()
    }
} 