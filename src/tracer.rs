//
// Code to draw the grid for, and trace paths over, 2D surfaces
// embedded in 3D. We use an implicit tracer over surfaces of the form
// f(x, y, z) = 0.
//

use glow::{Context, *};

use crate::vec3::*;

// Size of a step when tracing a ray.
const RAY_STEP: f64 = 0.01;

////////////////////////////////////////////////////////////////////////
// Shape: Representation of something to be drawn in OpenGL with a
// single `draw_elements` call.
//

pub struct Shape {
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

    fn rebuild(&mut self, gl: &Context, vertices: &[f32], indices: &[u32]) {
        unsafe {
            let vertices_u8: &[u8] = core::slice::from_raw_parts(
                vertices.as_ptr() as *const u8,
                std::mem::size_of_val(vertices),
            );
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_u8, glow::STATIC_DRAW);

            let indices_u8: &[u8] = core::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                std::mem::size_of_val(indices),
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
            gl.draw_elements(gl_type, self.num_elts, glow::UNSIGNED_INT, 0);
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

////////////////////////////////////////////////////////////////////////
// The core path tracer.

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum Function {
    Plane,
    PosCurve,
    NegCurve,
    SinXLin,
    SinXQuad,
    Hole,
}

impl Function {
    fn label(&self) -> &'static str {
        match self {
            Function::Plane => "Plane",
            Function::PosCurve => "Positive curvature",
            Function::NegCurve => "Negative curvature",
            Function::SinXLin => "Sin x Linear",
            Function::SinXQuad => "Sin x Quad",
            Function::Hole => "Wormhole",
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
    origin_ok: bool,
    func: Function,
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
            origin_ok: true,
            func: Function::SinXQuad,
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, gl: &Context) {
        use egui::Color32;
        let mut needs_regrid = false;
        let mut needs_repath = false;
        // Is there a less unpleasant way to make this type nicely?
        let red_on_fail: &dyn Fn(egui::Slider) -> egui::Slider = &|x| {
            if self.origin_ok {
                x
            } else {
                x.text_color(Color32::RED)
            }
        };
        needs_regrid |= ui
            .add(egui::Slider::new(&mut self.grid_size, 2..=100).text("Grid size"))
            .changed();
        needs_regrid |= ui
            .add(egui::Slider::new(&mut self.z_scale, -1.0..=1.0).text("Z scale"))
            .changed();
        needs_repath |= ui
            .add(red_on_fail(
                egui::Slider::new(&mut self.ray_start.0, -1.0..=1.0).text("X ray origin"),
            ))
            .changed();
        needs_repath |= ui
            .add(red_on_fail(
                egui::Slider::new(&mut self.ray_start.1, -1.0..=1.0).text("Y ray origin"),
            ))
            .changed();
        needs_repath |= ui
            .add(egui::Slider::new(&mut self.ray_dir, -180.0..=180.0).text("Ray angle"))
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
                    Function::Hole,
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
    }

    // Regenerate the grid used by OpenGL.
    pub fn regrid(&mut self, gl: &Context) {
        let (vertices, indices) = self.create_grid();
        self.grid.rebuild(gl, &vertices, &indices);
    }

    pub fn repath(&mut self, gl: &Context) {
        {
            let (vertices, indices) = self.repath_aux(self.ray_dir);
            self.paths.rebuild(gl, &vertices, &indices);
        }
        {
            let (vertices, indices) = self.repath_aux(self.ray_dir + 180.0);
            self.paths2.rebuild(gl, &vertices, &indices);
        }
    }

    pub fn close(&self, gl: &Context) {
        self.grid.close(gl);
        self.paths.close(gl);
        self.paths2.close(gl);
    }

    // Not a true distance, but the implicit surface function, where
    // the surface is all points where dist == 0.
    fn dist(&self, point: &Vec3) -> f64 {
        // If z_scale is zero, the implicit surface needs to be
        // special-cased to work.
        if self.z_scale.abs() <= 1.0e-7 {
            return point.z;
        }

        let (x, y, z) = (point.x, point.y, point.z / self.z_scale);
        match self.func {
            Function::Plane => (x + y) * 0.5 - z,
            Function::PosCurve => -(x * x + y * y) * 0.5 - z,
            Function::NegCurve => (x * x - y * y) * 0.5 - z,
            Function::SinXLin => (y * 4.0 * std::f64::consts::PI).sin() * x - z,
            Function::SinXQuad => (y * 4.0 * std::f64::consts::PI).sin() * x * x - z,
            Function::Hole => x * x + y * y - z * z - 0.1,
        }
    }

    // TODO: Need to deal with the cases where the solver fails for
    // reasons other than the origin's not ok. In particular, this can
    // happen when trying to draw the grid (where the path is forced
    // along grid axes.

    fn intersect_line(&self, point: &Vec3, direction: &Vec3) -> Option<Vec3> {
        // Newton-Raphson solver on dist(point + lambda direction)
        const EPSILON: f64 = 1.0e-7;
        // In practice, it's locally flat enough that a a single
        // iteration seems to suffice.
        const MAX_ITER: usize = 10;

        let mut lambda = 0.0;
        for _ in 0..MAX_ITER {
            let guess = point.add(&direction.scale(lambda));
            let guess_val = self.dist(&guess);
            if guess_val.abs() < EPSILON {
                return Some(guess);
            }

            let guess2 = point.add(&direction.scale(lambda + EPSILON));
            let guess2_val = self.dist(&guess2);

            let dguess_val = (guess2_val - guess_val) / EPSILON;

            lambda -= guess_val / dguess_val;
        }

        // Could fall back to binary chop, but as it generally seems
        // to converge in <= 2 iterations, this seems excessive.
        None
    }

    // Intersect the surface with a line in the z-axis from the
    // point. Roughly like the "z" function, except it should find the
    // nearest intersection.
    fn project_vertical(&self, point: &Vec3) -> Option<Vec3> {
        const VERTICAL: Vec3 = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        };
        self.intersect_line(point, &VERTICAL)
    }

    fn plot_path(&self, point: &Vec3, prev: &Vec3, vertices: &mut Vec<f32>) {
        const EPSILON: f64 = 1.0e-7;

        let mut p = point.clone();
        let mut old_p = prev.clone();

        while p.x.abs() <= 1.0 && p.y.abs() <= 1.0 {
            p.push_to(vertices);

            let delta = p.sub(&old_p).norm().scale(RAY_STEP);

            let base_dist = self.dist(&p);
            let mut norm = Vec3 {
                x: self.dist(&Vec3 {
                    x: p.x + EPSILON,
                    ..p
                }) - base_dist,
                y: self.dist(&Vec3 {
                    y: p.y + EPSILON,
                    ..p
                }) - base_dist,
                z: self.dist(&Vec3 {
                    z: p.z + EPSILON,
                    ..p
                }) - base_dist,
            };

            norm = norm.norm();

            if let Some(new_p) = self.intersect_line(&p.add(&delta), &norm) {
                (p, old_p) = (new_p, p);
            } else {
                log::error!("plot_path could not extend path");
                return;
            }
        }
        // Clip last point against grid and add.
        let delta = p.sub(&old_p);
        let x_excess = ((p.x.abs()) - 1.0) / delta.x.abs();
        let y_excess = ((p.y.abs()) - 1.0) / delta.y.abs();
        let fract = x_excess.max(y_excess);
        // We assume we can always find an intersection point at the
        // grid's edge.
        self.project_vertical(&p.sub(&delta.scale(fract)))
            .unwrap()
            .push_to(vertices);
    }

    fn repath_aux(&mut self, ray_dir: f64) -> (Vec<f32>, Vec<u32>) {
        const NOTHING: (Vec<f32>, Vec<u32>) = (vec![], vec![]);

        // Generate the vertices.
        let mut vertices: Vec<f32> = Vec::new();

        let (x0, y0) = self.ray_start;
        let p = if let Some(p) = self.project_vertical(&Vec3 {
            x: x0,
            y: y0,
            z: 1.0,
        }) {
            p
        } else {
            // No intersection point at ray_start. Give up.
            self.origin_ok = false;
            return NOTHING;
        };

        let ray_dir_rad = ray_dir * std::f64::consts::PI / 180.0;
        let delta = Vec3 {
            x: ray_dir_rad.sin() * RAY_STEP,
            y: ray_dir_rad.cos() * RAY_STEP,
            z: 0.0,
        };

        // Take a step back, roughly, for initial previous point.
        let old_p = if let Some(p) = self.project_vertical(&p.sub(&delta)) {
            p
        } else {
            // No intersection point near ray_start. Give up.
            self.origin_ok = false;
            return NOTHING;
        };

        self.origin_ok = true;

        self.plot_path(&p, &old_p, &mut vertices);

        // Generate the indices.
        let indices = (0..vertices.len() as u32 / 3).collect::<Vec<u32>>();

        (vertices, indices)
    }

    // This version of plot_path forces the line to lie within a given
    // plane, used for drawing the grid.
    fn plot_path_constrained(
        &self,
        point: &Vec3,
        prev: &Vec3,
        vertices: &mut Vec<f32>,
        constraint: &Vec3,
    ) {
        const EPSILON: f64 = 1.0e-7;

        // "constraint" should be pre-normalised.
        assert!((constraint.dot(constraint) - 1.0).abs() <= EPSILON);

        let mut p = point.clone();
        let mut old_p = prev.clone();

        while p.x.abs() <= 1.0 && p.y.abs() <= 1.0 {
            p.push_to(vertices);

            let mut delta = p.sub(&old_p).norm().scale(RAY_STEP);

            let base_dist = self.dist(&p);
            let mut norm = Vec3 {
                x: self.dist(&Vec3 {
                    x: p.x + EPSILON,
                    ..p
                }) - base_dist,
                y: self.dist(&Vec3 {
                    y: p.y + EPSILON,
                    ..p
                }) - base_dist,
                z: self.dist(&Vec3 {
                    z: p.z + EPSILON,
                    ..p
                }) - base_dist,
            };

            // Constrain the curvature to lie in the given plane.
            let projection_len = norm.dot(constraint);
            let projection_vec = constraint.scale(projection_len);
            norm = norm.sub(&projection_vec).norm();

            // Constraining the curvature direction so that the normal
            // is no longer genuinely normal to the surface can make
            // it so that in areas of high curvature there's no
            // intersection. So, if it fails, try again a few more
            // times with a smaller subdivision.
            const MAX_ITER: usize = 4;
            let mut new_p = None;
            let mut iter = 0;
            while new_p.is_none() && iter < MAX_ITER {
                new_p = self.intersect_line(&p.add(&delta), &norm);
                delta = delta.scale(0.5);
                iter += 1;
            }

            if let Some(new_p) = new_p {
                (p, old_p) = (new_p, p);
            } else {
                log::error!("plot_path_constrained could not extend path");
                return;
            }
        }

        // Clip last point against grid and add.
        let delta = p.sub(&old_p);
        let x_excess = ((p.x.abs()) - 1.0) / delta.x.abs();
        let y_excess = ((p.y.abs()) - 1.0) / delta.y.abs();
        let fract = x_excess.max(y_excess);
        // We assume we can always find an intersection point at the
        // grid's edge.
        self.project_vertical(&p.sub(&delta.scale(fract)))
            .unwrap()
            .push_to(vertices);
    }

    fn create_grid(&self) -> (Vec<f32>, Vec<u32>) {
        let mut v = Vec::new(); // Vertices
        let mut i = Vec::new(); // Indices

        let mut build = |constraint: &Vec3, flip: f64| {
            let x_scale = constraint.x;
            let y_scale = constraint.y;
            for idx in 0..=self.grid_size {
                let coord = (idx as f64 / self.grid_size as f64) * 2.0;
                let p = Vec3 {
                    x: coord * x_scale - 1.0,
                    y: coord * y_scale - 1.0,
                    z: 1.0,
                }
                .scale(flip);
                let p_prev = Vec3 {
                    x: coord * x_scale - 1.0 - (1.0 - x_scale) * RAY_STEP,
                    y: coord * y_scale - 1.0 - (1.0 - y_scale) * RAY_STEP,
                    z: 1.0,
                }
                .scale(flip);

                let old_len = v.len() / 3;
                // Build vertices. We assume we can always
                // project_vertical the points at the grid's edge.
                self.plot_path_constrained(
                    &self.project_vertical(&p).unwrap(),
                    &self.project_vertical(&p_prev).unwrap(),
                    &mut v,
                    constraint,
                );
                // And indices.
                let len = v.len() / 3;
                for idx in old_len..len - 2 {
                    i.push(idx as u32);
                    i.push(idx as u32 + 1);
                }
            }
        };

        build(
            &Vec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            1.0,
        );
        build(
            &Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            1.0,
        );
        // Fun special case
        if self.func == Function::Hole {
            build(
                &Vec3 {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                -1.0,
            );
            build(
                &Vec3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                -1.0,
            );
        }

        (v, i)
    }
}
