use lumina_capture::CaptureDevice;
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("LuminaRemote PoC - Local Render Test")
        .build(&event_loop)
        .unwrap();

    let context = unsafe { Context::new(&window) }.unwrap();
    let mut surface = unsafe { Surface::new(&context, &window) }.unwrap();

    let mut capturer = match lumina_capture::create_capture_device() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to initialize capture: {}", e);
            return;
        }
    };

    let mut is_focused = true;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event: WindowEvent::Focused(focused), .. } => {
                is_focused = focused;
                if focused {
                    println!("Window Focused: Sending StreamControl(Active) to Host");
                } else {
                    println!("Window Lost Focus: Sending StreamControl(Paused) to Host to save CPU/GPU");
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => {
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                if is_focused {
                    window.request_redraw();
                } else {
                    // Sleep to save CPU when window is inactive
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
            Event::RedrawRequested(_) => {
                if !is_focused { return; }
                
                if let Ok(frame) = capturer.capture_frame() {
                    let width = frame.width;
                    let height = frame.height;
                    
                    if width == 0 || height == 0 { return; }
                    
                    surface
                        .resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap();
                    
                    use rayon::prelude::*;
                    
                    let mut buffer = surface.buffer_mut().unwrap();
                    
                    // Convert RGBA from xcap to XRGB (0x00RRGGBB) for softbuffer in parallel
                    // This dramatically speeds up rendering on large 4K Retina screens.
                    buffer.par_iter_mut().zip(frame.data.par_chunks_exact(4)).for_each(|(dest, pixel)| {
                        let r = pixel[0] as u32;
                        let g = pixel[1] as u32;
                        let b = pixel[2] as u32;
                        *dest = b | (g << 8) | (r << 16);
                    });
                    
                    buffer.present().unwrap();
                }
            }
            _ => (),
        }
    });
}
