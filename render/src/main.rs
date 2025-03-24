mod mandelbrot;
mod frame_handler;
mod color_handler;
mod viewer_handler;

use mandelbrot::MandelbrotFrame;
use frame_handler::FrameHandler;
use viewer_handler::ViewerHandler;
use minifb::Key;

fn main() -> std::io::Result<()> {
    let width = 800;
    let height = 600;
    
    let mut frame_handler = FrameHandler::new(width, height);
    let mut viewer = ViewerHandler::new(width as usize, height as usize, "Mandelbrot Viewer");
    
    // Initial view state
    let mut center_x = -0.5;
    let mut center_y = 0.0;
    let mut zoom: f64 = 1.0;
    let mut base_iterations = 100;
    
    // Movement speed control
    let base_speed = 0.02;
    
    println!("Controls:");
    println!("Arrow keys: Move around");
    println!("+/-: Zoom in/out");
    println!("]/[: Increase/decrease base iterations");
    println!("Space: Toggle fine movement");
    println!("Escape: Exit");
    
    // Main loop
    while viewer.is_open() {
        // Handle keyboard input
        let movement_speed = base_speed / zoom;
        
        // Track if any movement or zoom keys are pressed
        let is_moving = viewer.is_key_down(Key::Left) 
            || viewer.is_key_down(Key::Right)
            || viewer.is_key_down(Key::Up)
            || viewer.is_key_down(Key::Down);
        let is_zooming = viewer.is_key_down(Key::Equal) || viewer.is_key_down(Key::Minus);
        let should_record = is_moving || is_zooming;
        
        // Handle movement
        if viewer.is_key_down(Key::Left) { center_x -= movement_speed; }
        if viewer.is_key_down(Key::Right) { center_x += movement_speed; }
        if viewer.is_key_down(Key::Up) { center_y -= movement_speed; }
        if viewer.is_key_down(Key::Down) { center_y += movement_speed; }
        if viewer.is_key_down(Key::Equal) { zoom *= 1.1; }
        if viewer.is_key_down(Key::Minus) { zoom /= 1.1; }
        if viewer.is_key_down(Key::RightBracket) { base_iterations += 10; }
        if viewer.is_key_down(Key::LeftBracket) && base_iterations > 10 { base_iterations -= 10; }
        
        // Dynamic detail adjustment based on zoom and movement
        let detail_multiplier = (1.0 + zoom.log10() * 2.0) as u32;
        let max_iterations = base_iterations * detail_multiplier;
        
        // Adjust sampling based on zoom level and movement/zooming
        let sample_step = if should_record {
            if zoom < 10.0 { 2 }
            else if zoom < 100.0 { 2 }
            else if zoom < 1000.0 { 3 }
            else { 4 }
        } else {
            1
        };
        
        // Create and update frame
        let mut frame_calc = MandelbrotFrame::new(width/sample_step, height/sample_step);
        frame_calc.x_min = center_x - (1.5 / zoom);
        frame_calc.x_max = center_x + (1.5 / zoom);
        frame_calc.y_min = center_y - (1.0 / zoom);
        frame_calc.y_max = center_y + (1.0 / zoom);
        frame_calc.max_iterations = max_iterations;
        
        // Calculate and render frame
        let iterations = frame_calc.calculate();
        frame_handler.render_frame(&iterations, frame_calc.max_iterations, sample_step);
        
        // Update viewer
        viewer.update(frame_handler.get_draw_target());
        
        
        // Print current view state
        print!("\rCenter: ({:.3}, {:.3}), Zoom: {:.1}x, Iterations: {}, Detail: {}x, Sample: {}px{}    ", 
               center_x, center_y, zoom, base_iterations, detail_multiplier, sample_step,
               if should_record { " (Recording)" } else { "" });
    }
    
    println!(); // Final newline
    Ok(())
}