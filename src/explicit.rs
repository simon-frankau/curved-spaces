//
// Code to draw the grid for, and trace paths over, an explicit
// representation of a 2D surface in 3D. That is, surfaces of the form
// z = f(x, y). It's simple, but prevents surfaces that double back.
//

use glow::{Context, *};

use crate::vec3::*;

// Size of a step when tracing a ray.
const RAY_STEP: f64 = 0.01;

pub struct Shape {
    vao: VertexArray,
    vbo: Buffer,
    ibo: Buffer,
    num_elts: i32,
}

// TODO: Probably shouldn't be in this module.
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

    pub fn draw(&self, gl: &Context, gl_type: u32) {
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Function {
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

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Algorithm {
    ExtrapNearest,
    DiffEqn,
}

impl Algorithm {
    fn label(&self) -> &'static str {
        match self {
            Algorithm::ExtrapNearest => "Extrapolate & Nearest",
            Algorithm::DiffEqn => "Differential Equation",
        }
    }
}

pub struct Tracer {
    pub grid: Shape,
    pub paths: Shape,
    pub paths2: Shape,
    grid_size: usize,
    z_scale: f64,
    ray_start: (f64, f64),
    ray_dir: f64,
    iter: usize,
    func: Function,
    algo: Algorithm,
}

impl Tracer {
    pub fn new(gl: &Context) -> Tracer {
        Tracer {
            grid: Shape::new(gl),
            paths: Shape::new(gl),
            paths2: Shape::new(gl),
            grid_size: 30,
            z_scale: 0.25,
            ray_start: (0.0, -0.9),
            ray_dir: 0.0,
            iter: 1,
            func: Function::SinXQuad,
            algo: Algorithm::ExtrapNearest,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, gl: &Context) {
        let mut needs_regrid = false;
        let mut needs_repath = false;
        needs_regrid |= ui
            .add(egui::Slider::new(&mut self.grid_size, 2..=100).text("Grid size"))
            .changed();
        needs_regrid |= ui
            .add(egui::Slider::new(&mut self.z_scale, -1.0..=1.0).text("Z scale"))
            .changed();
        needs_repath |= ui
            .add(egui::Slider::new(&mut self.ray_start.0, -1.0..=1.0).text("X ray origin"))
            .changed();
        needs_repath |= ui
            .add(egui::Slider::new(&mut self.ray_start.1, -1.0..=1.0).text("Y ray origin"))
            .changed();
        needs_repath |= ui
            .add(egui::Slider::new(&mut self.ray_dir, -180.0..=180.0).text("Ray angle"))
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
        needs_repath |= egui::ComboBox::from_label("Algorithm")
            .selected_text(self.algo.label())
            .show_ui(ui, |ui| {
                [Algorithm::ExtrapNearest, Algorithm::DiffEqn]
                    .iter()
                    .map(|x| ui.selectable_value(&mut self.algo, *x, x.label()).changed())
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
    }

    pub fn close(&self, gl: &Context) {
        self.grid.close(gl);
        self.paths.close(gl);
        self.paths2.close(gl);
    }

    fn z(&self, x: f64, y: f64) -> f64 {
        (match self.func {
            Function::Plane => (x + y) * 0.5,
            Function::PosCurve => -(x * x + y * y) * 0.5,
            Function::NegCurve => (x * x - y * y) * 0.5,
            Function::SinXLin => (y * 4.0 * std::f64::consts::PI).sin() * x,
            Function::SinXQuad => (y * 4.0 * std::f64::consts::PI).sin() * x * x,
        }) * self.z_scale
    }

    fn create_grid(&self) -> Vec<f32> {
        let mut v = Vec::new();
        for x in 0..=self.grid_size {
            let x_coord = (x as f64 / self.grid_size as f64) * 2.0 - 1.0;
            for y in 0..=self.grid_size {
                let y_coord = (y as f64 / self.grid_size as f64) * 2.0 - 1.0;
                Vec3 {
                    x: x_coord,
                    y: y_coord,
                    z: self.z(x_coord, y_coord),
                }
                .push_to(&mut v);
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
    pub fn regrid(&mut self, gl: &Context) {
        let vertices = self.create_grid();
        let indices = self.create_grid_indices();
        self.grid.rebuild(gl, &vertices, &indices);
    }

    fn normal_at(&self, p: &Vec3) -> Vec3 {
        // dz/dx gives a tangent vector: (1, 0, dz/dx).
        // dz/dy gives a tangent vector: (0, 1, dz/dy).
        // Cross product is normal: (-dz/dx, -dy/dx, 1).
        //
        // (This generalises into higher dimensions, but is a simple
        // explanation for 3D.)

        // We could do this algebraically, but finite difference is
        // easy and general.

        let z0 = self.z(p.x, p.y);
        const EPS: f64 = 1.0e-7;

        let dzdx = (self.z(p.x + EPS, p.y) - z0) / EPS;
        let dzdy = (self.z(p.x, p.y + EPS) - z0) / EPS;

        let norm = (1.0 + dzdx * dzdx + dzdy * dzdy).powf(-0.5);

        Vec3 {
            x: -dzdx * norm,
            y: -dzdy * norm,
            z: norm,
        }
    }

    fn nearest_point_to(&self, p: &Vec3) -> Vec3 {
        // Initial guess at solution is the starting point, projected
        // down onto the surface.
        let mut sol = Vec3 {
            z: self.z(p.x, p.y),
            ..*p
        };
        for _ in 0..self.iter {
            let delta = p.sub(&sol);
            let normal = self.normal_at(&sol);
            let len = normal.dot(&delta);

            let step = delta.sub(&normal.scale(len));
            sol = sol.add(&step);
            sol.z = self.z(sol.x, sol.y)
        }
        sol
    }

    fn solve_diff_eqn(&self, p: &Vec3, delta: &Vec3) -> Vec3 {
        let z0 = self.z(p.x, p.y);
        const EPS: f64 = 1.0e-7;

        let dzdx = (self.z(p.x + EPS, p.y) - z0) / EPS;
        let dzdy = (self.z(p.x, p.y + EPS) - z0) / EPS;

        // Generate the vector representing the second derivative.
        //
        // Huh. This looks suspiciously similar to calculations we do
        // in `normal_at`, which makes sense, having the curvature
        // vector being the normal vector.
        let d2 = Vec3 {
            x: -dzdx,
            y: -dzdy,
            z: 1.0,
        };

        // TODO: Doesn't always converge, probably because it's not
        // real Newton-Raphson. Fun.
        let mut new_p = p.add(&delta);
        for _ in 0..self.iter {
            let diff = self.z(new_p.x, new_p.y) - new_p.z;
            let d2_scaled = d2.scale(diff);
            new_p = new_p.add(&d2_scaled);
        }
        new_p
    }

    pub fn repath(&mut self, gl: &Context) {
        {
            let (vertices, indices) = self.repath_aux(self.ray_dir);
            self.check_path(&vertices);
            self.paths.rebuild(gl, &vertices, &indices);
        }
        {
            let (vertices, indices) = self.repath_aux(self.ray_dir + 180.0);
            self.check_path(&vertices);
            self.paths2.rebuild(gl, &vertices, &indices);
        }
    }

    fn repath_aux(&mut self, ray_dir: f64) -> (Vec<f32>, Vec<u16>) {
        // Generate the vertices.
        let mut vertices: Vec<f32> = Vec::new();

        let (x0, y0) = self.ray_start;
        let mut p = Vec3 {
            x: x0,
            y: y0,
            z: self.z(x0, y0),
        };

        let ray_dir_rad = ray_dir * std::f64::consts::PI / 180.0;
        let mut delta = Vec3 {
            x: ray_dir_rad.sin() * RAY_STEP,
            y: ray_dir_rad.cos() * RAY_STEP,
            z: 0.0,
        };
        // Calculate initial dz by taking a step back.
        delta.z = p.z - self.z(p.x - delta.x, p.y - delta.y);

        while p.x.abs() <= 1.0 && p.y.abs() <= 1.0 {
            p.push_to(&mut vertices);
            let old_p = p.clone();

            match self.algo {
                // Linear extrapolation in the embedding space, and
                // then find the nearest point on the embedded space.
                Algorithm::ExtrapNearest => {
                    p = p.add(&delta);

                    // See README.md for why we do this.
                    p = self.nearest_point_to(&p);
                    p.z = self.z(p.x, p.y);
                }
                // Follow the differential equation constraints for
                // the geodesic (see maths.md).
                Algorithm::DiffEqn => {
                    p = self.solve_diff_eqn(&p, &delta);
                }
            }

            // TODO: Normalise?
            delta = p.sub(&old_p);
        }
        // Clip last point against grid and add.
        let x_excess = ((p.x.abs()) - 1.0) / delta.x.abs();
        let y_excess = ((p.y.abs()) - 1.0) / delta.y.abs();
        let fract = x_excess.max(y_excess);
        p = p.sub(&delta.scale(fract));
        p.z = self.z(p.x, p.y);
        p.push_to(&mut vertices);

        // Generate the indices.
        let indices: Vec<u16> = (0..vertices.len() as u16 / 3).collect::<Vec<u16>>();

        (vertices, indices)
    }

    // Check that the local constraints for a geodesic are met. See
    // maths.md for details.
    fn check_path(&self, points: &[f32]) {
        // Convert flattened array into points.
        let points = points
            .chunks_exact(3)
            .map(|p| Vec3 {
                x: p[0] as f64,
                y: p[1] as f64,
                z: p[2] as f64,
            })
            .collect::<Vec<_>>();

        log::info!("Path check:");
        for point in points.windows(3) {
            let (a, b, c) = (&point[0], &point[1], &point[2]);
            // First derivatives along path.
            let diff1 = b.sub(&a).norm();
            let diff2 = c.sub(&b).norm();
            // Second derivative
            let dd = diff2.sub(&diff1);
            // And derivatives on the surface.
            const EPSILON: f64 = 1.0e-7;
            let dzdx = (self.z(b.x + EPSILON, b.y) - self.z(b.x, b.y)) / EPSILON;
            let dzdy = (self.z(b.x, b.y + EPSILON) - self.z(b.x, b.y)) / EPSILON;

            // Calculate errors from expected value.
            let x_err = dd.x + dzdx * dd.z;
            let y_err = dd.y + dzdy * dd.z;
            let total_curve = dd.len();

            log::info!(
                "    curve {:.7} x_err {:.7} y_err {:.7}",
                total_curve,
                x_err,
                y_err
            );
        }
    }
}
