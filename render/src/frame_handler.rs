use raqote::*;
use crate::color_handler::ColorHandler;
use rayon::prelude::*;

pub struct FrameHandler {
    width: u32,
    height: u32,
    draw_target: DrawTarget,
    color_handler: ColorHandler,
}

impl FrameHandler {
    pub fn new(width: u32, height: u32) -> Self {
        FrameHandler {
            width,
            height,
            draw_target: DrawTarget::new(width as i32, height as i32),
            color_handler: ColorHandler::new(),
        }
    }

    pub fn render_frame(&mut self, iterations: &[u32], max_iterations: u32, sample_step: u32) {
        let pixels = self.draw_target.get_data_mut();
        let width = self.width as usize;
        let sampled_width = (self.width / sample_step) as usize;
        let sampled_height = (self.height / sample_step) as usize;
        
        // Process each row in parallel
        pixels.chunks_mut(width)
            .enumerate()
            .par_bridge()
            .for_each(|(y, row)| {
                // Ensure we don't sample beyond our input data
                let sample_y = (y / sample_step as usize).min(sampled_height - 1);
                
                for (x, pixel) in row.iter_mut().enumerate() {
                    // Ensure we don't sample beyond our input data
                    let sample_x = (x / sample_step as usize).min(sampled_width - 1);
                    let idx = sample_y * sampled_width + sample_x;
                    
                    let iterations = iterations[idx];
                    let color = self.color_handler.get_color(iterations, max_iterations);
                    *pixel = color.to_u32();
                }
            });
    }

    pub fn get_draw_target(&self) -> &DrawTarget {
        &self.draw_target
    }
} 