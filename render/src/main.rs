mod mandelbrot;
mod frame_handler;
mod color_handler;
mod viewer_handler;
mod animation_handler;

use mandelbrot::MandelbrotFrame;
use frame_handler::FrameHandler;
use viewer_handler::ViewerHandler;
use animation_handler::AnimationHandler;
use minifb::Key;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let width = 800;
    let height = 600;
    
    let mut frame_handler = FrameHandler::new(width, height);
    let mut viewer = ViewerHandler::new(width as usize, height as usize, "Mandelbrot Viewer");
    let mut animation_handler = AnimationHandler::new(width, height, "animation.gif", 30)?;
    
    // Initial view state
    let mut center_x = -0.5;
    let mut center_y = 0.0;
    let mut zoom: f64 = 1.0;
    let mut base_iterations = 100;
    
    // Movement speed control
    let base_speed = 0.02;
    
    // Animation state
    let start_time = Instant::now();
    
    println!("Controls:");
    println!("Arrow keys: Move around");
    println!("+/-: Zoom in/out");
    println!("]/[: Increase/decrease base iterations");
    println!("Space: Toggle fine movement");
    println!("S: Set start node for animation");
    println!("E: Set end node for animation");
    println!("C: Clear animation nodes");
    println!("A: Create animation (if start and end nodes are set)");
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

        // Handle animation controls
        if viewer.is_key_pressed(Key::S) {
            let current_time = start_time.elapsed().as_secs_f64();
            animation_handler.set_start_node(center_x, center_y, current_time, zoom);
            println!("Start node set at ({:.3}, {:.3}) with zoom {:.1}x", center_x, center_y, zoom);
        }
        if viewer.is_key_pressed(Key::E) {
            let current_time = start_time.elapsed().as_secs_f64();
            animation_handler.set_end_node(center_x, center_y, current_time, zoom);
            println!("End node set at ({:.3}, {:.3}) with zoom {:.1}x", center_x, center_y, zoom);
        }
        if viewer.is_key_pressed(Key::C) {
            animation_handler.clear_nodes();
            println!("Animation nodes cleared");
        }
        if viewer.is_key_pressed(Key::A) {
            if animation_handler.has_start_node() && animation_handler.has_end_node() {
                println!("Creating animation...");
                // Create a fresh MandelbrotFrame for the animation with current settings
                let mut animation_frame = MandelbrotFrame::new(width, height);
                animation_frame.max_iterations = base_iterations * (1.0 + zoom.log10() * 2.0) as u32;
                animation_handler.create_animation(&mut frame_handler, &mut animation_frame)?;
                println!("Animation created!");
            } else {
                println!("Please set both start and end nodes first");
            }
        }
        
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
        
        // Print current view state and animation status
        print!("\rCenter: ({:.3}, {:.3}), Zoom: {:.1}x, Iterations: {}, Detail: {}x, Sample: {}px{} {} {}    ", 
               center_x, center_y, zoom, base_iterations, detail_multiplier, sample_step,
               if should_record { " (Recording)" } else { "" },
               if animation_handler.has_start_node() { "[Start Set]" } else { "" },
               if animation_handler.has_end_node() { "[End Set]" } else { "" });
    }
    
    println!(); // Final newline
    Ok(())
}