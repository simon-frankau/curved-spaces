//
// TODO: Initial GL-displaying thing.
//
// Based on
// https://github.com/grovesNL/glow/blob/main/examples/hello/src/main.rs
//

use glow::*;

////////////////////////////////////////////////////////////////////////
// wasm32: Create a context from a WebGL2 context on wasm32 targets.
//

#[cfg(target_arch = "wasm32")]
type Program = WebProgramKey;

#[cfg(target_arch = "wasm32")]
struct Platform {
    gl: Context,
    shader_version: &'static str,
}

#[cfg(target_arch = "wasm32")]
impl Platform {
    fn new() -> Platform {
        use wasm_bindgen::JsCast;
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let webgl2_context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();
        let gl = glow::Context::from_webgl2_context(webgl2_context);
        Platform {
            gl,
            shader_version: "#version 300 es",
        }
    }

    fn run(&self) {
        // This could be called from `requestAnimationFrame`, a winit event
        // loop, etc.
        draw(&self.gl);
    }
}

////////////////////////////////////////////////////////////////////////
// glutin_winit: Create a context from a glutin window on non-wasm32
// targets.
//

#[cfg(feature = "glutin_winit")]
type Program = NativeProgram;

#[cfg(feature = "glutin_winit")]
#[derive(Debug)]
pub enum UserEvent {
    Redraw(std::time::Duration),
}

#[cfg(feature = "glutin_winit")]
struct Platform {
    gl: std::sync::Arc<Context>,
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    gl_context: glutin::context::PossiblyCurrentContext,
    shader_version: &'static str,
    window: winit::window::Window,
    event_loop: Option<winit::event_loop::EventLoop<UserEvent>>,
}

#[cfg(feature = "glutin_winit")]
impl Platform {
    fn new() -> Platform {
        use glutin::{
            config::{ConfigTemplateBuilder, GlConfig},
            context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
            display::{GetGlDisplay, GlDisplay},
            surface::{GlSurface, SwapInterval},
        };
        use glutin_winit::{DisplayBuilder, GlWindow};
        use raw_window_handle::HasRawWindowHandle;
        use std::num::NonZeroU32;

        let event_loop = winit::event_loop::EventLoopBuilder::<UserEvent>::with_user_event()
            .build()
            .unwrap();
        let window_builder = winit::window::WindowBuilder::new()
            .with_title("Hello triangle!")
            .with_inner_size(winit::dpi::LogicalSize::new(1024.0, 768.0));

        let template = ConfigTemplateBuilder::new();

        let display_builder = DisplayBuilder::new().with_window_builder(Some(window_builder));

        let (window, gl_config) = display_builder
            .build(&event_loop, template, |configs| {
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            })
            .unwrap();

        let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

        let window = window.unwrap();

        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
                major: 4,
                minor: 10,
            })))
            .build(raw_window_handle);

        let (gl, gl_surface, gl_context) = unsafe {
            let not_current_gl_context = gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap();

            let attrs = window.build_surface_attributes(Default::default());
            let gl_surface = gl_display
                .create_window_surface(&gl_config, &attrs)
                .unwrap();

            let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

            let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));

            (gl, gl_surface, gl_context)
        };

        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
            .unwrap();

        Platform {
            gl: std::sync::Arc::new(gl),
            gl_surface,
            gl_context,
            shader_version: "#version 410",
            window,
            event_loop: Some(event_loop),
        }
    }

    fn run(&mut self, program: Program, vertex_array: VertexArray, viewport: &[i32; 4]) {
        use glutin::prelude::GlSurface;

        // `run` "uses up" the event_loop, so we move it out.
        let mut event_loop = None;
        std::mem::swap(&mut event_loop, &mut self.event_loop);
        let event_loop = event_loop.expect("Event loop already run");

        let mut egui_glow =
            egui_glow::winit::EguiGlow::new(&event_loop, self.gl.clone(), None, None);

        let event_loop_proxy = egui::mutex::Mutex::new(event_loop.create_proxy());
        egui_glow
            .egui_ctx
            .set_request_repaint_callback(move |info| {
                event_loop_proxy
                    .lock()
                    .send_event(UserEvent::Redraw(info.delay))
                    .expect("Cannot send event");
            });

        let mut repaint_delay = std::time::Duration::MAX;

        let _ = event_loop.run(move |event, event_loop_window_target| {
            let mut redraw = || {
                let mut quit = false;

                egui_glow.run(&self.window, |egui_ctx| {
                    egui::SidePanel::left("my_side_panel").show(egui_ctx, |ui| {
                        ui.heading("Hello World!");
                        if ui.button("Quit").clicked() {
                            quit = true;
                        }
                        // TODO ui.color_edit_button_rgb(&mut clear_color);
                    });
                });

                if quit {
                    event_loop_window_target.exit();
                } else {
                    event_loop_window_target.set_control_flow(if repaint_delay.is_zero() {
                        self.window.request_redraw();
                        winit::event_loop::ControlFlow::Poll
                    } else if let Some(repaint_after_instant) =
                        std::time::Instant::now().checked_add(repaint_delay)
                    {
                        winit::event_loop::ControlFlow::WaitUntil(repaint_after_instant)
                    } else {
                        winit::event_loop::ControlFlow::Wait
                    });
                }

                {
                    unsafe {
                        use glow::HasContext as _;
                        // self.gl.clear_color(clear_color[0], clear_color[1], clear_color[2], 1.0);
                        self.gl.clear(glow::COLOR_BUFFER_BIT);
                    }

                    // draw things behind egui here
                    draw(&self.gl, program, vertex_array, viewport);

                    egui_glow.paint(&self.window);

                    // draw things on top of egui here

                    self.gl_surface.swap_buffers(&self.gl_context).unwrap();
                    self.window.set_visible(true);
                }
            };

            match event {
                winit::event::Event::WindowEvent { event, .. } => {
                    use winit::event::WindowEvent;
                    if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                        event_loop_window_target.exit();
                        return;
                    }

                    if matches!(event, WindowEvent::RedrawRequested) {
                        redraw();
                        return;
                    }

                    if let winit::event::WindowEvent::Resized(physical_size) = &event {
                        self.gl_surface.resize(
                            &self.gl_context,
                            physical_size.width.try_into().unwrap(),
                            physical_size.height.try_into().unwrap(),
                        );
                    }

                    let event_response = egui_glow.on_window_event(&self.window, &event);

                    if event_response.repaint {
                        self.window.request_redraw();
                    }
                }

                winit::event::Event::UserEvent(UserEvent::Redraw(delay)) => {
                    repaint_delay = delay;
                }
                winit::event::Event::LoopExiting => {
                    egui_glow.destroy();
                }
                winit::event::Event::NewEvents(winit::event::StartCause::ResumeTimeReached {
                    ..
                }) => {
                    self.window.request_redraw();
                }

                _ => (),
            }
        });
    }
}

////////////////////////////////////////////////////////////////////////
// SDL2: Create a context from an sdl2 window.
//

#[cfg(feature = "dsl2")]
type Program = NativeProgram;

#[cfg(feature = "sdl2")]
struct Platform {
    gl: Context,
    shader_version: &'static str,
    window: sdl2::video::Window,
    event_loop: sdl2::EventPump,
    gl_context: sdl2::video::GLContext,
}

#[cfg(feature = "sdl2")]
impl Platform {
    fn new() -> Platform {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
        let window = video
            .window("Hello triangle!", 1024, 769)
            .opengl()
            .resizable()
            .build()
            .unwrap();
        let gl_context = window.gl_create_context().unwrap();
        let gl = unsafe {
            glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
        };
        let event_loop = sdl.event_pump().unwrap();
        Platform {
            gl,
            shader_version: "#version 330",
            window,
            event_loop,
            gl_context,
        }
    }

    fn run(&mut self) {
        let mut running = true;
        while running {
            {
                for event in self.event_loop.poll_iter() {
                    match event {
                        sdl2::event::Event::Quit { .. } => running = false,
                        _ => {}
                    }
                }
            }

            draw(&self.gl);
            self.window.gl_swap_window();
        }
    }
}

////////////////////////////////////////////////////////////////////////
// Main code.
//

fn main() {
    let mut p = Platform::new();

    let mut viewport = [0, 0, 0, 0];
    unsafe {
        p.gl.get_parameter_i32_slice(VIEWPORT, &mut viewport);
    }

    let (program, vertex_array) = build_program(&p.gl, p.shader_version);

    unsafe {
        p.gl.use_program(Some(program));
        p.gl.clear_color(0.1, 0.2, 0.3, 1.0);
    }

    p.run(program, vertex_array, &viewport);

    unsafe {
        p.gl.delete_program(program);
        p.gl.delete_vertex_array(vertex_array);
    }
}

fn build_program(gl: &Context, shader_version: &str) -> (Program, VertexArray) {
    unsafe {
        let vertex_array = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vertex_array));

        let program = gl.create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", shader_version, shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                panic!("{}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("{}", gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        return (program, vertex_array);
    }
}

fn draw(gl: &Context, program: Program, vertex_array: VertexArray, viewport: &[i32; 4]) {
    unsafe {
        gl.use_program(Some(program));
        gl.bind_vertex_array(Some(vertex_array));
        gl.viewport(viewport[0], viewport[1], viewport[2], viewport[3]);
        gl.draw_arrays(glow::TRIANGLES, 0, 3);
        // gl.draw_arrays(glow::LINE_LOOP, 0, 3);
    }
}
