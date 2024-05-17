//
// vec3.rs: The inevitable 3D vector class. Writing my own to keep it
// simple and avoid another dependency.
//

#[derive(Clone)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    // Add the vector to a Vec<f32> to be used by OpenGL.
    pub fn push_to(&self, v: &mut Vec<f32>) {
        v.push(self.x as f32);
        v.push(self.y as f32);
        v.push(self.z as f32);
    }

    pub fn scale(&self, m: f64) -> Vec3 {
        Vec3 {
            x: self.x * m,
            y: self.y * m,
            z: self.z * m,
        }
    }

    pub fn add(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }

    pub fn sub(&self, rhs: &Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }

    pub fn dot(&self, rhs: &Vec3) -> f64 {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }

    pub fn len(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2) + self.z.powi(2)).sqrt()
    }

    pub fn norm(&self) -> Vec3 {
        self.scale(self.len().recip())
    }
}
