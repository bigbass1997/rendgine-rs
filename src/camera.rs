
use cgmath::{Vector3, Matrix4, Zero, vec3, InnerSpace, SquareMatrix, Point3, Rotation3, Rad, Deg, Rotation};
use std::ops::{Sub, Mul, Add, AddAssign};
use cgmath::num_traits::abs;
use cgmath::num_traits::One;
use cgmath::EuclideanSpace;

pub struct Camera {
    pub position:   Vector3<f32>,
    pub direction:  Vector3<f32>,
    pub up:         Vector3<f32>,
    
    pub projection: Matrix4<f32>,
    pub view:       Matrix4<f32>,
    pub combined:   Matrix4<f32>,
    
    pub viewport_width: f32,
    pub viewport_height: f32,
    
    pub near: f32,
    pub far: f32,
    
    pub fov: f32,
    pub aspect: f32,
    pub is_perspective: bool,
}

impl Camera {
    pub fn new(x: f32, y: f32, z: f32, viewport_width: f32, viewport_height: f32) -> Camera {
        let mut cam = Camera {
            position:   Vector3::new(x, y, z),
            direction:  Vector3::new(0.0, 0.0, -1.0),
            up:         Vector3::new(0.0, 1.0,  0.0),
            
            projection: Matrix4::zero(),
            view:       Matrix4::zero(),
            combined:   Matrix4::zero(),
            
            viewport_width,
            viewport_height,
            
            near: -1.0,
            far: 1.0,
            
            fov: 90.0,
            aspect: viewport_width / viewport_height,
            is_perspective: false
        };
        cam.update();
        
        cam
    }
    
    pub fn with_perspective(mut self, near: f32, far: f32) -> Self {
        self.near = near;
        self.far = far;
        self.is_perspective = true;
        self.update();
        
        self
    }
    
    pub fn update(&mut self) {
        self.aspect = self.viewport_width / self.viewport_height;
        
        if self.is_perspective {
            let mut mat = Matrix4::<f32>::zero();
            let fov = Rad::from(Deg(self.fov)).0;
            let h = (fov * 0.5).tan();
            
            mat.x.x = 1.0 / (h * self.aspect);
            mat.y.y = 1.0 / h;
            
            let far_inf = self.far > 0.0 && self.far.is_infinite();
            let near_inf = self.near > 0.0 && self.near.is_infinite();
            if far_inf {
                mat.z.z = 0.000001 - 1.0;
                mat.w.z = (0.000001 - 2.0) * self.near;
            } else if near_inf {
                mat.z.z = 1.0 - 0.000001;
                mat.w.z = (2.0 - 0.000001) * self.far;
            } else {
                mat.z.z = (self.far + self.near) / (self.near - self.far);
                mat.w.z = (self.far + self.far) * self.near / (self.near - self.far);
            }
            mat.z.w = -1.0;
            
            self.projection = mat;
        } else {
            Self::set_ortho(&mut self.projection, -(self.viewport_width / 2.0), self.viewport_width / 2.0, -(self.viewport_height / 2.0), self.viewport_height / 2.0, self.near, self.far);
        }
        
        let tmp = self.position.add(self.direction);
        self.view = Matrix4::<f32>::look_at_rh(Point3::from_vec(self.position), Point3::from_vec(tmp), self.up);
        self.combined = self.projection.mul(self.view);
    }
    
    fn set_ortho(mat: &mut Matrix4<f32>, left: f32, right: f32, bottom: f32, top: f32, z_near: f32, z_far: f32) -> Matrix4<f32> {
        if !mat.is_identity() {
            mat.set_one();
        }
        mat.x.x = 2.0 / (right - left);
        mat.y.y = 2.0 / (top - bottom);
        mat.z.z = 2.0 / (z_near - z_far);
        mat.w.x = (right + left) / (left - right);
        mat.w.y = (top + bottom) / (bottom - top);
        
        *mat
    }
    
    pub fn look_at(&mut self, x: f32, y: f32, z: f32){
        let tmp = vec3(x, y, z).sub(&self.position).normalize();
        if !tmp.eq(&Vector3::zero()) {
            let dot = tmp.dot(self.up);
            if abs(dot - 1.0) < 0.000000001 {
                self.up = self.direction.mul(-1.0);
            } else if abs(dot + 1.0) < 0.000000001 {
                self.up = self.direction;
            }
            self.direction = tmp;
            self.normalize_up();
        }
    }
    
    pub fn normalize_up(&mut self) {
        let tmp = self.direction.cross(self.up).normalize();
        self.up = tmp.cross(self.direction).normalize();
    }
    
    pub fn rotate(&mut self, angle: f32, axis: Vector3<f32>) {
        let rot = cgmath::Basis3::from_axis_angle(axis.normalize(), Deg(angle));
        self.direction = rot.rotate_vector(self.direction);
        self.up = rot.rotate_vector(self.up);
        
        self.update();
    }
    
    pub fn translate(&mut self, x: f32, y: f32, z: f32) {
        self.position.add_assign(vec3(x, y, z));
        
        self.update();
    }
}