use std::fs::File;
use std::io::{self, Error};
use gif::{Frame, Encoder};
use raqote::DrawTarget;

pub struct AnimationHandler {
    width: u32,
    height: u32,
    encoder: Encoder<File>,
}

impl AnimationHandler {
    pub fn new(width: u32, height: u32, filename: &str) -> io::Result<Self> {
        let file = File::create(filename)?;
        let encoder = Encoder::new(file, width as u16, height as u16, &[])
            .map_err(|e| Error::new(io::ErrorKind::Other, e))?;
        
        Ok(AnimationHandler {
            width,
            height,
            encoder,
        })
    }

    pub fn add_frame(&mut self, draw_target: &DrawTarget, delay: u16) -> io::Result<()> {
        let pixels = draw_target.get_data();
        let mut buffer = Vec::with_capacity((self.width * self.height * 4) as usize);
        
        // Convert ARGB to RGB palette
        for pixel in pixels.iter() {
            let b = (pixel & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let r = ((pixel >> 16) & 0xFF) as u8;
            buffer.push(r);
            buffer.push(g);
            buffer.push(b);
            buffer.push(255); // Alpha
        }

        let mut frame = Frame::from_rgba_speed(
            self.width as u16,
            self.height as u16,
            &mut buffer,
            10, // Speed value between 1 and 30. Higher = faster but lower quality
        );
        frame.delay = delay; // In hundredths of a second
        
        self.encoder.write_frame(&frame)
            .map_err(|e| Error::new(io::ErrorKind::Other, e))?;
        
        Ok(())
    }
} 