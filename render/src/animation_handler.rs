use std::fs::File;
use std::io::{self, Error};
use gif::{Frame, Encoder};
use raqote::DrawTarget;
use crate::frame_handler::FrameHandler;
use crate::mandelbrot::MandelbrotFrame;

#[derive(Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy)]
pub struct AnimationNode {
    pub position: Position,
    pub time: f64,  // Time in seconds when this node should be reached
    pub zoom: f64,  // Zoom level at this node
}

pub struct AnimationHandler {
    width: u32,
    height: u32,
    encoder: Encoder<File>,
    fps: u32,
    start_node: Option<AnimationNode>,
    end_node: Option<AnimationNode>,
}

impl AnimationHandler {
    pub fn new(width: u32, height: u32, filename: &str, fps: u32) -> io::Result<Self> {
        let file = File::create(filename)?;
        let encoder = Encoder::new(file, width as u16, height as u16, &[])
            .map_err(|e| Error::new(io::ErrorKind::Other, e))?;
        
        Ok(AnimationHandler {
            width,
            height,
            encoder,
            fps,
            start_node: None,
            end_node: None,
        })
    }

    pub fn set_start_node(&mut self, x: f64, y: f64, time: f64, zoom: f64) {
        self.start_node = Some(AnimationNode {
            position: Position { x, y },
            time,
            zoom,
        });
    }

    pub fn set_end_node(&mut self, x: f64, y: f64, time: f64, zoom: f64) {
        self.end_node = Some(AnimationNode {
            position: Position { x, y },
            time,
            zoom,
        });
    }

    pub fn has_start_node(&self) -> bool {
        self.start_node.is_some()
    }

    pub fn has_end_node(&self) -> bool {
        self.end_node.is_some()
    }

    pub fn clear_nodes(&mut self) {
        self.start_node = None;
        self.end_node = None;
    }

    fn interpolate_position(start: &Position, end: &Position, t: f64) -> Position {
        Position {
            x: start.x + (end.x - start.x) * t,
            y: start.y + (end.y - start.y) * t,
        }
    }

    pub fn create_animation(&mut self, frame_handler: &mut FrameHandler, mandelbrot: &mut MandelbrotFrame) -> io::Result<()> {
        // Get the nodes and their data before the mutable borrow
        let start_node = self.start_node.expect("Start node must be set before creating animation");
        let end_node = self.end_node.expect("End node must be set before creating animation");
            
        let duration = end_node.time - start_node.time;
        let total_frames = (duration * self.fps as f64) as u32;
        
        for frame in 0..total_frames {
            let t = frame as f64 / total_frames as f64;
            let current_pos = Self::interpolate_position(&start_node.position, &end_node.position, t);
            let current_zoom = start_node.zoom + (end_node.zoom - start_node.zoom) * t;
            
            // Update Mandelbrot frame with interpolated position and zoom
            mandelbrot.x_min = current_pos.x - (1.5 / current_zoom);
            mandelbrot.x_max = current_pos.x + (1.5 / current_zoom);
            mandelbrot.y_min = current_pos.y - (1.0 / current_zoom);
            mandelbrot.y_max = current_pos.y + (1.0 / current_zoom);
            
            // Calculate and render the frame
            let iterations = mandelbrot.calculate();
            frame_handler.render_frame(&iterations, mandelbrot.max_iterations, 1);
            
            // Calculate delay in hundredths of a second (gif delay unit)
            let delay = (100.0 / self.fps as f64) as u16;
            self.add_frame(frame_handler.get_draw_target(), delay)?;
            
            // Print progress
            print!("\rGenerating animation: {:.1}%", (frame as f64 / total_frames as f64) * 100.0);
        }
        println!(); // New line after progress
        
        Ok(())
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