//
// TODO: Initial GL-displaying thing.
//

use anyhow::*;
use glow::{Context, *};

// Size of a step when tracing a ray.
const RAY_STEP: f32 = 0.01f32;

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
                            self.resize(physical_size);
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
                        drawable.close(&self.gl);
                    }
                    winit::event::Event::NewEvents(
                        winit::event::StartCause::ResumeTimeReached { .. },
                    ) => {
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
impl Platform {
    fn new() -> Result<Platform> {
        // TODO: Make parameters?
        let width = 1024;
        let height = 768;
        let dest_id = "wasm-canvas";

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
            .get_element_by_id(dest_id)
            .ok_or_else(|| anyhow!("Couldn't get element '{}'", dest_id))?;

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
impl Platform {
    fn new() -> Result<Platform> {
        // TODO: Make parameters?
        let width = 1024;
        let height = 768;
        let title = "Hello triangle!";

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
            .with_title(title)
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
impl Platform {
    fn new() -> Result<Platform> {
        Self::new_inner().map_err(|s| anyhow!("{}", s))
    }

    fn new_inner() -> std::result::Result<Platform, String> {
        // TODO: Make parameters?
        let width = 1024;
        let height = 768;
        let title = "Hello triangle!";

        let sdl = sdl2::init()?;
        let video = sdl.video()?;
        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
        gl_attr.set_context_version(3, 0);
        let window = video
            .window(title, width, height)
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

fn main() -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();
    #[cfg(target_arch = "wasm32")]
    let _ = console_log::init_with_level(log::Level::Info);
    #[cfg(target_arch = "wasm32")]
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let mut p = Platform::new()?;

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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Function {
    Plane,
    PosCurve,
    NegCurve,
    SinXLin,
    SinXQuad,
}

impl Function {
    fn label(&self) -> &'static str {
        match self {
            Function::Plane => "Plane",
            Function::PosCurve => "Positive curvature",
            Function::NegCurve => "Negative curvature",
            Function::SinXLin => "Sin x Linear",
            Function::SinXQuad => "Sin x Quad",
        }
    }
}

struct Shape {
    vao: VertexArray,
    vbo: Buffer,
    ibo: Buffer,
    num_elts: i32,
}

impl Shape {
    // Create vertex and index buffers, and vertex array to describe vertex buffer.
    fn new(gl: &Context) -> Shape {
        unsafe {
            // We construct buffer, data will be uploaded later.
            let ibo = gl.create_buffer().unwrap();
            let vbo = gl.create_buffer().unwrap();
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            // We now construct a vertex array to describe the format of the input buffer
            let vao = gl.create_vertex_array().unwrap();
            gl.bind_vertex_array(Some(vao));
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                glow::FLOAT,
                false,
                core::mem::size_of::<f32>() as i32 * 3,
                0,
            );

            Shape {
                vbo,
                vao,
                ibo,
                num_elts: 0,
            }
        }
    }

    fn rebuild(&mut self, gl: &Context, vertices: &[f32], indices: &[u16]) {
        unsafe {
            let vertices_u8: &[u8] = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                vertices.len() * core::mem::size_of::<f32>(),
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

            let indices_u8: &[u8] = core::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                indices.len() * core::mem::size_of::<f32>(),
            );
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ibo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_u8, glow::STATIC_DRAW);

            self.num_elts = indices.len() as i32;
        }
    }

    fn draw(&self, gl: &Context, gl_type: u32) {
        // Assumes program, uniforms, etc. are set.
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.enable_vertex_attrib_array(0);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ibo));
            gl.draw_elements(gl_type, self.num_elts, glow::UNSIGNED_SHORT, 0);
            gl.disable_vertex_attrib_array(0);
        }
    }

    fn close(&self, gl: &Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ibo);
        }
    }
}

struct Drawable {
    program: Program,
    grid: Shape,
    paths: Shape,
    tilt_id: UniformLocation,
    tilt: f32,
    turn_id: UniformLocation,
    turn: f32,
    x_scale_id: UniformLocation,
    y_scale_id: UniformLocation,
    color_id: UniformLocation,
    grid_size: usize,
    z_scale: f32,
    ray_start: (f32, f32),
    ray_dir: f32,
    iter: usize,
    func: Function,
}

const VERT_SRC: &str = include_str!("shader/vertex.glsl");
const FRAG_SRC: &str = include_str!("shader/fragment.glsl");

impl Drawable {
    fn new(gl: &Context, shader_version: &str) -> Drawable {
        unsafe {
            let grid = Shape::new(gl);
            let paths = Shape::new(gl);

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
                grid,
                paths,
                tilt_id,
                tilt: 30.0f32,
                turn_id,
                turn: 0.0f32,
                x_scale_id,
                y_scale_id,
                color_id,
                grid_size: 30,
                z_scale: 0.25f32,
                ray_start: (0.0f32, -0.9f32),
                ray_dir: 0.0f32,
                iter: 1,
                func: Function::SinXQuad,
            };
            this.regrid(gl);
            this.repath(gl);
            this
        }
    }

    fn ui(&mut self, ctx: &egui::Context, gl: &Context) {
        egui::Window::new("Controls").show(ctx, |ui| {
            // TODO
            // if ui.button("Quit").clicked() {}
            let mut needs_regrid = false;
            let mut needs_repath = false;
            ui.add(egui::Slider::new(&mut self.tilt, -90.0..=90.0).text("Tilt"));
            ui.add(egui::Slider::new(&mut self.turn, -180.0..=180.0).text("Turn"));
            needs_regrid |= ui
                .add(egui::Slider::new(&mut self.grid_size, 2..=100).text("Grid size"))
                .changed();
            needs_regrid |= ui
                .add(egui::Slider::new(&mut self.z_scale, -1.0f32..=1.0f32).text("Z scale"))
                .changed();
            needs_repath |= ui
                .add(
                    egui::Slider::new(&mut self.ray_start.0, -1.0f32..=1.0f32).text("X ray origin"),
                )
                .changed();
            needs_repath |= ui
                .add(
                    egui::Slider::new(&mut self.ray_start.1, -1.0f32..=1.0f32).text("Y ray origin"),
                )
                .changed();
            needs_repath |= ui
                .add(egui::Slider::new(&mut self.ray_dir, -180.0f32..=180.0f32).text("Ray angle"))
                .changed();
            needs_repath |= ui
                .add(egui::Slider::new(&mut self.iter, 1..=10).text("Iterations"))
                .changed();
            needs_regrid |= egui::ComboBox::from_label("Function")
                .selected_text(self.func.label())
                .show_ui(ui, |ui| {
                    [
                        Function::Plane,
                        Function::PosCurve,
                        Function::NegCurve,
                        Function::SinXLin,
                        Function::SinXQuad,
                    ]
                    .iter()
                    .map(|x| ui.selectable_value(&mut self.func, *x, x.label()).changed())
                    // Force evaluation of whole list.
                    .fold(false, |a, b| a || b)
                })
                .inner
                .unwrap_or(false);
            if needs_regrid {
                self.regrid(gl);
            }
            if needs_regrid || needs_repath {
                self.repath(gl);
            }
        });
    }

    fn z64(&self, x: f64, y: f64) -> f64 {
        (match self.func {
            Function::Plane => (x + y) * 0.5,
            Function::PosCurve => (x * x + y * y) * 0.5,
            Function::NegCurve => (x * x - y * y) * 0.5,
            Function::SinXLin => (y * 4.0 * std::f64::consts::PI).sin() * x,
            Function::SinXQuad => (y * 4.0 * std::f64::consts::PI).sin() * x * x,
        }) * self.z_scale as f64
    }

    // TODO: Useful for OpenGL, but probably not worth it.
    fn z32(&self, x: f32, y: f32) -> f32 {
        self.z64(x as f64, y as f64) as f32
    }

    fn create_grid(&self) -> Vec<f32> {
        let mut v = Vec::new();
        for x in 0..=self.grid_size {
            let x_coord = (x as f32 / self.grid_size as f32) * 2.0f32 - 1.0f32;
            for y in 0..=self.grid_size {
                let y_coord = (y as f32 / self.grid_size as f32) * 2.0f32 - 1.0f32;
                v.push(x_coord);
                v.push(y_coord);
                v.push(self.z32(x_coord, y_coord));
            }
        }
        v
    }

    fn create_grid_indices(&self) -> Vec<u16> {
        let mut v = Vec::new();
        for x in 0..=self.grid_size as u16 {
            let x_idx = x * (self.grid_size as u16 + 1);
            for y in 0..self.grid_size as u16 {
                v.push(x_idx + y);
                v.push(x_idx + y + 1);
            }
        }
        for x in 0..self.grid_size as u16 {
            let x_idx = x * (self.grid_size as u16 + 1);
            for y in 0..=self.grid_size as u16 {
                v.push(x_idx + y);
                v.push(x_idx + y + self.grid_size as u16 + 1);
            }
        }
        v
    }

    // Regenerate the grid used by OpenGL.
    fn regrid(&mut self, gl: &Context) {
        let vertices = self.create_grid();
        let indices = self.create_grid_indices();
        self.grid.rebuild(gl, &vertices, &indices);
    }

    fn normal_at(&self, x: f32, y: f32) -> (f32, f32, f32) {
        // dz/dx gives a tangent vector: (1, 0, dz/dx).
        // dz/dy gives a tangent vector: (0, 1, dz/dy).
        // Cross product is normal: (-dz/dx, -dy/dx, 1).
        //
        // (This generalises into higher dimensions, but is a simple
        // explanation for 3D.)

        // We could do this algebraically, but finite difference is
        // easy and general.

        // Use f64s for extra precision here.
        let (x, y) = (x as f64, y as f64);
        let z0 = self.z64(x, y);
        const EPS: f64 = 1.0e-7;

        let dzdx = (self.z64(x + EPS, y) - z0) / EPS;
        let dzdy = (self.z64(x, y + EPS) - z0) / EPS;

        let norm = (1.0 + dzdx * dzdx + dzdy * dzdy).powf(-0.5);

        ((-dzdx * norm) as f32, (-dzdy * norm) as f32, norm as f32)
    }

    fn nearest_point_to(&self, x: f32, y: f32, z: f32) -> (f32, f32, f32) {
        let (mut px, mut py, mut pz) = (x, y, self.z32(x, y));
        for _ in 0..self.iter {
            let (dx, dy, dz) = (x - px, y - py, z - pz);
            let (nx, ny, nz) = self.normal_at(px, py);
            let len = nx * dx + ny * dy + nz * dz;

            px += dx - nx * len;
            py += dy - ny * len;
            pz = self.z32(px, py)
        }
        (px, py, pz)
    }

    fn repath(&mut self, gl: &Context) {
        // Generate the vertices.
        let mut vertices: Vec<f32> = Vec::new();
        let ray_dir_rad = self.ray_dir * std::f32::consts::PI / 180.0f32;
        let mut dx = ray_dir_rad.sin() * RAY_STEP;
        let mut dy = ray_dir_rad.cos() * RAY_STEP;
        let (mut x, mut y) = self.ray_start;
        let mut z = self.z32(x, y);
        // Calculate initial dz by taking a step back.
        let mut dz = z - self.z32(x - dx, y - dy);

        while x.abs() <= 1.0f32 && y.abs() <= 1.0f32 {
            vertices.push(x);
            vertices.push(y);
            vertices.push(z);
            let (old_x, old_y, old_z) = (x, y, z);

            x += dx;
            y += dy;
            z += dz;

            (x, y, z) = self.nearest_point_to(x, y, z);

            // TODO: Normalise?
            (dx, dy, dz) = (x - old_x, y - old_y, z - old_z);
        }
        // Clip last point against grid and add.
        let x_excess = ((x.abs()) - 1.0f32) / dx.abs();
        let y_excess = ((y.abs()) - 1.0f32) / dy.abs();
        let fract = x_excess.max(y_excess);
        x -= fract * dx;
        y -= fract * dy;
        vertices.push(x);
        vertices.push(y);
        vertices.push(self.z32(x, y));

        // Generate the indices.
        let indices: Vec<u16> = (0..vertices.len() as u16 / 3).collect::<Vec<u16>>();

        self.paths.rebuild(gl, &vertices, &indices);
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
            self.grid.draw(&gl, glow::LINES);

            gl.uniform_3_f32(Some(&self.color_id), 1.0f32, 0.5f32, 0.5f32);
            self.paths.draw(&gl, glow::LINE_STRIP);
        }
    }

    fn close(&self, gl: &Context) {
        unsafe {
            gl.delete_program(self.program);
        }
        self.grid.close(gl);
    }
}
