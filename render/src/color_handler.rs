use raqote::SolidSource;

pub struct ColorHandler {
    saturation: f32,
    value: f32,
}

impl ColorHandler {
    pub fn new() -> Self {
        ColorHandler {
            saturation: 1.0,
            value: 1.0,
        }
    }

    pub fn get_color(&self, iterations: u32, max_iterations: u32) -> SolidSource {
        if iterations == max_iterations {
            // Point is in the set - color it black
            SolidSource::from_unpremultiplied_argb(255, 0, 0, 0)
        } else {
            // Point is outside the set - create a color based on iterations
            let hue = (iterations as f32 / max_iterations as f32) * 360.0;
            let (r, g, b) = Self::hsv_to_rgb(hue, self.saturation, self.value);
            SolidSource::from_unpremultiplied_argb(255, r, g, b)
        }
    }

    fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
        let c = v * s;
        let h_prime = h / 60.0;
        let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
        let m = v - c;

        let (r, g, b) = match h_prime as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            5 => (c, 0.0, x),
            _ => (0.0, 0.0, 0.0),
        };

        (
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }
} 