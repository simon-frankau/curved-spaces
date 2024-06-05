//
// Build a height-map of a real-valued function of two real variables,
// representing a 2D manifold (?) embedded in 3D space, and try to
// draw geodesics (curves of minimal length) on its surface, as an
// attempt to understand how curved surfaces work.
//

use anyhow::*;
use glow::{Context, *};

mod tracer;
mod vec3;

use crate::tracer::*;

////////////////////////////////////////////////////////////////////////
// winit: Shared between wasm32 and glutin_winit.
//

#[cfg(any(target_arch = "wasm32", feature = "glutin_winit"))]
#[derive(Debug)]
pub enum UserEvent {
    Redraw(std::time::Duration),
}

#[cfg(any(target_arch = "wasm32", feature = "glutin_winit"))]
struct Platform {
    gl: std::sync::Arc<Context>,
    shader_version: &'static str,
    window: winit::window::Window,
    event_loop: Option<winit::event_loop::EventLoop<UserEvent>>,

    #[cfg(feature = "glutin_winit")]
    gl_surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    #[cfg(feature = "glutin_winit")]
    gl_context: glutin::context::PossiblyCurrentContext,
}

#[cfg(any(target_arch = "wasm32", feature = "glutin_winit"))]
impl Platform {
    fn run(mut self, mut drawable: Drawable) {
        use winit::event::*;

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
        let mut left_button_down = false;

        let event_fn =
            move |event,
                  event_loop_window_target: &winit::event_loop::EventLoopWindowTarget<
                UserEvent,
            >| {
                let mut redraw = || {
                    let mut quit = false;

                    egui_glow.run(&self.window, |egui_ctx| {
                        drawable.ui(egui_ctx, &self.gl);
                    });

                    if quit {
                        event_loop_window_target.exit();
                    } else {
                        event_loop_window_target.set_control_flow(if repaint_delay.is_zero() {
                            self.window.request_redraw();
                            winit::event_loop::ControlFlow::Poll
                        } else if let Some(repaint_after_instant) =
                            web_time::Instant::now().checked_add(repaint_delay)
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
                        let size = self.window.inner_size();
                        drawable.draw(&self.gl, size.width, size.height);

                        egui_glow.paint(&self.window);

                        // draw things on top of egui here

                        self.swap_buffers();
                    }
                };

                match event {
                    Event::WindowEvent { event, .. } => {
                        if matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed) {
                            event_loop_window_target.exit();
                            return;
                        }

                        if matches!(event, WindowEvent::RedrawRequested) {
                            redraw();
                            return;
                        }

                        if let WindowEvent::Resized(physical_size) = &event {
                            self.resize(physical_size);
                        }

                        let event_response = egui_glow.on_window_event(&self.window, &event);

                        if event_response.repaint {
                            self.window.request_redraw();
                        }

                        if !event_response.consumed {
                            match event {
                                // We check the WindowEvent rather
                                // than the DeviceEvent in order to
                                // allow egui to consume it first.
                                WindowEvent::MouseInput { state, button, .. } => {
                                    if button == MouseButton::Left {
                                        left_button_down = state == ElementState::Pressed;
                                    }
                                }
                                // We will make use of keyboard
                                // auto-repeat for movement, rather
                                // than doing our own key-held
                                // logic. As we're using WASD keys,
                                // we'll use the PhysicalKey.
                                WindowEvent::KeyboardInput { event, .. } => {
                                    use winit::keyboard::*;
                                    if let KeyEvent {
                                        physical_key: PhysicalKey::Code(k),
                                        state: ElementState::Pressed,
                                        ..
                                    } = event
                                    {
                                        let t = &mut drawable.tracer;
                                        match k {
                                            KeyCode::KeyW => {
                                                t.update_origin(&self.gl, 0.0, 0.01, 0.0)
                                            }
                                            KeyCode::KeyS => {
                                                t.update_origin(&self.gl, 0.0, -0.01, 0.0)
                                            }
                                            KeyCode::KeyA => {
                                                t.update_origin(&self.gl, -0.01, 0.0, 0.0)
                                            }
                                            KeyCode::KeyD => {
                                                t.update_origin(&self.gl, 0.01, 0.0, 0.0)
                                            }
                                            KeyCode::KeyQ => {
                                                t.update_origin(&self.gl, 0.0, 0.0, -1.0)
                                            }
                                            KeyCode::KeyE => {
                                                t.update_origin(&self.gl, 0.0, 0.0, 1.0)
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    Event::DeviceEvent { event, .. } => {
                        if left_button_down {
                            // DeviceEvent is better than WindowEvent for
                            // this kind of camera dragging, according to
                            // the docs.
                            if let DeviceEvent::MouseMotion { delta } = event {
                                let size = self.window.inner_size();
                                let x = (delta.0 as f32) * 360.0 / size.width as f32;
                                let y = (delta.1 as f32) * 180.0 / size.height as f32;

                                let turn = &mut drawable.turn;
                                let tilt = &mut drawable.tilt;
                                *turn += x;
                                if *turn > 180.0 {
                                    *turn -= 360.0;
                                } else if *turn < -180.0 {
                                    *turn += 360.0;
                                }
                                *tilt = (*tilt + y).min(90.0).max(-90.0);
                            }
                        }
                    }

                    Event::UserEvent(UserEvent::Redraw(delay)) => {
                        repaint_delay = delay;
                    }
                    Event::LoopExiting => {
                        egui_glow.destroy();
                        drawable.close(&self.gl);
                    }
                    Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                        self.window.request_redraw();
                    }

                    _ => (),
                }
            };

        Self::run_event_loop(event_loop, event_fn);
    }
}

////////////////////////////////////////////////////////////////////////
// wasm32: Create a context from a WebGL2 context on wasm32 targets.
//

#[cfg(target_arch = "wasm32")]
const NAME: &str = "wasm-canvas";

#[cfg(target_arch = "wasm32")]
impl Platform {
    fn new(width: u32, height: u32, name: &str) -> Result<Platform> {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowBuilderExtWebSys;
        use winit::platform::web::WindowExtWebSys;

        let event_loop =
            winit::event_loop::EventLoopBuilder::<UserEvent>::with_user_event().build()?;

        let doc = web_sys::window()
            .ok_or_else(|| anyhow!("Couldn't get window"))?
            .document()
            .ok_or_else(|| anyhow!("Couldn't get document"))?;

        let dest = doc
            .get_element_by_id(name)
            .ok_or_else(|| anyhow!("Couldn't get element '{}'", name))?;

        // Try casting the element into canvas:
        let canvas_opt = dest.clone().dyn_into::<web_sys::HtmlCanvasElement>().ok();

        let window = winit::window::WindowBuilder::new()
            .with_inner_size(winit::dpi::LogicalSize::new(width, height))
            .with_canvas(canvas_opt.clone())
            .build(&event_loop)?;

        // WindowBuilder will construct a canvas if we didn't have
        // one.
        let canvas = window
            .canvas()
            .ok_or_else(|| anyhow!("Couldn't get canvas"))?;

        // Size the canvas correctly.
        canvas
            .set_attribute(
                "style",
                &format!("width: {}px; height: {}px;", width, height),
            )
            .map_err(|_| anyhow!("Couldn't set style on canvas"))?;

        if canvas_opt.is_none() {
            // Element wasn't a canvas, a canvas has been created by
            // WindowBuilder, but we need to insert it into the doc.
            dest.append_child(&web_sys::Element::from(canvas.clone()))
                .map_err(|_| anyhow!("Couldn't add canvas to HTML"))?;
        }

        let webgl2_context = canvas
            .get_context("webgl2")
            .map_err(|_| anyhow!("Couldn't get webgl2 context"))?
            .ok_or_else(|| anyhow!("Couldn't get webgl2 context"))?
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .map_err(|_| anyhow!("Couldn't cast to WebGL2RenderingContext"))?;
        let gl = glow::Context::from_webgl2_context(webgl2_context);

        Ok(Platform {
            gl: std::sync::Arc::new(gl),
            shader_version: "#version 300 es",
            window,
            event_loop: Some(event_loop),
        })
    }

    fn swap_buffers(&self) {
        // Not needed on wasm, as automatically
        // switches when the function returns.
    }

    fn resize(&self, physical_size: &winit::dpi::PhysicalSize<u32>) {
        // On web, when the zoom is changed, we want everything to
        // scale in a consistent manner, so that it behaves like the
        // rest of the web page.
        //
        // So, we keep the logical size the same, and change the
        // physical size.
        //
        // This is different from eframe,
        // because eframe uses the whole window, so that the logical
        // size (points) changes but the physical size (pixels) does
        // not. For us, we expect the canvas to be just part of the
        // page.
        use winit::platform::web::WindowExtWebSys;
        let canvas = self.window.canvas().unwrap();
        canvas.set_width(physical_size.width);
        canvas.set_height(physical_size.height);
    }

    fn run_event_loop(
        event_loop: winit::event_loop::EventLoop<UserEvent>,
        event_fn: impl FnMut(
                winit::event::Event<UserEvent>,
                &winit::event_loop::EventLoopWindowTarget<UserEvent>,
            ) + 'static,
    ) {
        use winit::platform::web::EventLoopExtWebSys;
        event_loop.spawn(event_fn);
    }
}

////////////////////////////////////////////////////////////////////////
// glutin_winit: Create a context from a glutin window on non-wasm32
// targets.
//
#[cfg(feature = "glutin_winit")]
const NAME: &str = "Curved Surfaces";

#[cfg(feature = "glutin_winit")]
impl Platform {
    fn new(width: u32, height: u32, name: &str) -> Result<Platform> {
        use glutin::{
            config::{ConfigTemplateBuilder, GlConfig},
            context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContext},
            display::{GetGlDisplay, GlDisplay},
            surface::{GlSurface, SwapInterval},
        };
        use glutin_winit::{DisplayBuilder, GlWindow};
        use raw_window_handle::HasRawWindowHandle;
        use std::num::NonZeroU32;

        let event_loop =
            winit::event_loop::EventLoopBuilder::<UserEvent>::with_user_event().build()?;

        let window_builder = winit::window::WindowBuilder::new()
            .with_title(name)
            .with_inner_size(winit::dpi::LogicalSize::new(width as f32, height as f32));
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
            .map_err(|_| anyhow!("Couldn't build display"))?;

        let raw_window_handle = window.as_ref().map(|window| window.raw_window_handle());

        let window = window.ok_or_else(|| anyhow!("Couldn't get window"))?;

        let gl_display = gl_config.display();
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(glutin::context::Version {
                major: 4,
                minor: 10,
            })))
            .build(raw_window_handle);

        let (gl, gl_surface, gl_context) = unsafe {
            let not_current_gl_context =
                gl_display.create_context(&gl_config, &context_attributes)?;
            let attrs = window.build_surface_attributes(Default::default());
            let gl_surface = gl_display.create_window_surface(&gl_config, &attrs)?;
            let gl_context = not_current_gl_context.make_current(&gl_surface)?;
            let gl = glow::Context::from_loader_function_cstr(|s| gl_display.get_proc_address(s));
            (gl, gl_surface, gl_context)
        };

        gl_surface
            .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))?;

        Ok(Platform {
            gl: std::sync::Arc::new(gl),
            shader_version: "#version 410",
            window,
            event_loop: Some(event_loop),

            gl_surface,
            gl_context,
        })
    }

    fn swap_buffers(&self) {
        use glutin::prelude::GlSurface;
        self.gl_surface.swap_buffers(&self.gl_context).unwrap();
        self.window.set_visible(true);
    }

    fn resize(&self, physical_size: &winit::dpi::PhysicalSize<u32>) {
        // In a native window, resizing the window changes both
        // logical and physical size. Thus the ratio stays the same,
        // and the egui interface stays the same size. Zoom is handled
        // separately, and works like web zoom.
        use glutin::prelude::GlSurface;
        self.gl_surface.resize(
            &self.gl_context,
            physical_size.width.try_into().unwrap(),
            physical_size.height.try_into().unwrap(),
        );
    }

    fn run_event_loop(
        event_loop: winit::event_loop::EventLoop<UserEvent>,
        event_fn: impl FnMut(
                winit::event::Event<UserEvent>,
                &winit::event_loop::EventLoopWindowTarget<UserEvent>,
            ) + 'static,
    ) {
        let _ = event_loop.run(event_fn);
    }
}

////////////////////////////////////////////////////////////////////////
// SDL2: Create a context from an sdl2 window.
//
// TODO: No egui integration.
//

#[cfg(feature = "sdl2")]
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
const NAME: &str = "Curved Surfaces";

#[cfg(feature = "sdl2")]
impl Platform {
    fn new(width: u32, height: u32, name: &str) -> Result<Platform> {
        Self::new_inner(width, height, name).map_err(|s| anyhow!("{}", s))
    }

    fn new_inner(width: u32, height: u32, name: &str) -> std::result::Result<Platform, String> {
        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
        let window = video
            .window(name, width, height)
            .opengl()
            .resizable()
            .build()
            .map_err(|e| e.to_string())?;
        let gl_context = window.gl_create_context()?;
        let gl = unsafe {
            glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _)
        };
        let event_loop = sdl.event_pump()?;
        std::result::Result::Ok(Platform {
            gl,
            shader_version: "#version 330",
            window,
            event_loop,
            gl_context,
        })
    }

    fn run(&mut self, mut drawable: Drawable) {
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

            let (width, height) = self.window.size();
            drawable.draw(&self.gl, width, height);
            self.window.gl_swap_window();
        }

        drawable.close(&self.gl);
    }
}

////////////////////////////////////////////////////////////////////////
// Main code.
//

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

fn main() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    #[cfg(target_arch = "wasm32")]
    let _ = console_log::init_with_level(log::Level::Info);
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let mut p = Platform::new(WIDTH, HEIGHT, NAME)?;

    let drawable = Drawable::new(&p.gl, p.shader_version);

    unsafe {
        p.gl.clear_color(0.1, 0.2, 0.3, 1.0);
    }

    // `run` should call `drawable.close(&p.gl)` when done. We don't
    // call it here, as `run` may run the event loop asynchronously
    // (e.g. for web).
    p.run(drawable);

    Ok(())
}

struct Drawable {
    program: Program,
    tilt_id: UniformLocation,
    tilt: f32,
    turn_id: UniformLocation,
    turn: f32,
    x_scale_id: UniformLocation,
    y_scale_id: UniformLocation,
    color_id: UniformLocation,
    tracer: Tracer,
}

const VERT_SRC: &str = include_str!("shader/vertex.glsl");
const FRAG_SRC: &str = include_str!("shader/fragment.glsl");

impl Drawable {
    fn new(gl: &Context, shader_version: &str) -> Drawable {
        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            let shader_sources = [
                (glow::VERTEX_SHADER, VERT_SRC),
                (glow::FRAGMENT_SHADER, FRAG_SRC),
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

            let tilt_id = gl.get_uniform_location(program, "tilt").unwrap();
            let turn_id = gl.get_uniform_location(program, "turn").unwrap();
            let x_scale_id = gl.get_uniform_location(program, "x_scale").unwrap();
            let y_scale_id = gl.get_uniform_location(program, "y_scale").unwrap();
            let color_id = gl.get_uniform_location(program, "color").unwrap();

            for shader in shaders {
                gl.detach_shader(program, shader);
                gl.delete_shader(shader);
            }

            let mut this = Drawable {
                program,
                tilt_id,
                tilt: 30.0f32,
                turn_id,
                turn: 0.0f32,
                x_scale_id,
                y_scale_id,
                color_id,
                tracer: Tracer::new(gl),
            };
            this.tracer.regrid(gl);
            this.tracer.repath(gl);
            this
        }
    }

    fn ui(&mut self, ctx: &egui::Context, gl: &Context) {
        egui::Window::new("Controls").show(ctx, |ui| {
            // TODO
            // if ui.button("Quit").clicked() {}
            ui.add(egui::Slider::new(&mut self.tilt, -90.0..=90.0).text("Tilt"));
            ui.add(egui::Slider::new(&mut self.turn, -180.0..=180.0).text("Turn"));
            self.tracer.ui(ui, gl);
        });
    }

    fn draw(&mut self, gl: &Context, width: u32, height: u32) {
        unsafe {
            // Set up state shared across lines.
            gl.viewport(0, 0, width as i32, height as i32);
            gl.use_program(Some(self.program));
            gl.uniform_1_f32(Some(&self.tilt_id), self.tilt);
            gl.uniform_1_f32(Some(&self.turn_id), self.turn);
            gl.uniform_1_f32(
                Some(&self.x_scale_id),
                (height as f32 / width as f32).min(1.0f32),
            );
            gl.uniform_1_f32(
                Some(&self.y_scale_id),
                (width as f32 / height as f32).min(1.0f32),
            );

            gl.uniform_3_f32(Some(&self.color_id), 0.5f32, 0.5f32, 0.5f32);
            self.tracer.grid.draw(gl, glow::LINES);

            gl.uniform_3_f32(Some(&self.color_id), 1.0f32, 0.5f32, 0.5f32);
            self.tracer.paths.draw(gl, glow::LINES);

            gl.uniform_3_f32(Some(&self.color_id), 0.5f32, 01.0f32, 0.5f32);
            self.tracer.paths2.draw(gl, glow::LINES);
        }
    }

    fn close(&self, gl: &Context) {
        unsafe {
            gl.delete_program(self.program);
        }
        self.tracer.close(gl);
    }
}
