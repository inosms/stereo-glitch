use bevy_ecs::system::Resource;
use cgmath::InnerSpace;

#[derive(Resource)]
pub struct StereoCamera {
    /// The camera eye for the center (left and right eye are calculated from this)
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    /// Eye distance in world space units
    /// This is the distance between the left and right eye
    /// The left eye is at -eye_distance/2 and the right eye is at eye_distance/2
    eye_distance: f32,

    eye_distance_factor: f32,
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl StereoCamera {
    /// Build view projection matrices for the left and right eye
    fn build_view_projection_matrices(&self) -> (cgmath::Matrix4<f32>, cgmath::Matrix4<f32>) {
        let looking_vec = (self.target - self.eye).normalize();
        let eye_displacement_direction = looking_vec.cross(cgmath::Vector3::unit_z());

        let left_eye = self.eye - eye_displacement_direction * self.eye_distance * self.eye_distance_factor * 0.5;
        let right_eye = self.eye + eye_displacement_direction * self.eye_distance * self.eye_distance_factor * 0.5;

        let left_view = cgmath::Matrix4::look_at_rh(left_eye, self.target, self.up);
        let right_view = cgmath::Matrix4::look_at_rh(right_eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);

        (
            OPENGL_TO_WGPU_MATRIX * proj * left_view,
            OPENGL_TO_WGPU_MATRIX * proj * right_view,
        )
    }

    /// Create a new camera
    pub fn new(
        eye: cgmath::Point3<f32>,
        target: cgmath::Point3<f32>,
        up: cgmath::Vector3<f32>,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
        eye_distance: f32,
    ) -> Self {
        Self {
            eye,
            target,
            up,
            aspect,
            fovy,
            znear,
            zfar,
            eye_distance,
            eye_distance_factor: 0.0,
        }
    }

    pub fn set_eye_distance_factor(&mut self, eye_distance_factor: f32) {
        self.eye_distance_factor = eye_distance_factor;
    }

    /// Set the eye distance in world space units
    pub fn set_eye_distance(&mut self, eye_distance: f32) {
        self.eye_distance = eye_distance;
    }

    /// Set the camera target
    pub fn set_target(&mut self, target: cgmath::Point3<f32>) {
        self.target = target;
    }

    /// Smoothly set the camera target
    /// If the distance between the current target and the new target is greater than jump_distance
    /// then the target is set immediately
    pub fn smooth_set_target(&mut self, target: cgmath::Point3<f32>, smooth_factor: f32, jump_distance: f32) {
        let distance = (self.target - target).magnitude();
        if distance > jump_distance {
            self.target = target;
        } else {
            self.target = self.target + (target - self.target) * smooth_factor;
        }
    }

    /// Smoothly set the camera eye
    /// If the distance between the current eye and the new eye is greater than jump_distance
    /// then the eye is set immediately
    pub fn smooth_set_eye(&mut self, eye: cgmath::Point3<f32>, smooth_factor: f32, jump_distance: f32) {
        let distance = (self.eye - eye).magnitude();
        if distance > jump_distance {
            self.eye = eye;
        } else {
            self.eye = self.eye + (eye - self.eye) * smooth_factor;
        }
    }

    /// Set the camera eye
    pub fn set_eye(&mut self, eye: cgmath::Point3<f32>) {
        self.eye = eye;
    }

    /// Set the camera aspect ratio
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Get the direction the camera looks at as a vector projected to the ground
    pub fn get_camera_view_direction_projected_to_ground(&self) -> cgmath::Vector3<f32> {
        let camera_look_vec = (self.target - self.eye).normalize();

        let camera_look_vec_projected_to_ground =
            cgmath::Vector3::new(camera_look_vec.x, camera_look_vec.y, 0.0).normalize();

        camera_look_vec_projected_to_ground
    }
}

/// A uniform struct to hold the view projection matrix (needed for WGSL)
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct StereoCameraUniform {
    view_proj_left: [[f32; 4]; 4],
    view_proj_right: [[f32; 4]; 4],
    // no padding needed with two 4x4 f32 matrices = 2 * 4*4 * 4 bytes = 128 bytes
}

impl StereoCameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj_left: cgmath::Matrix4::identity().into(),
            view_proj_right: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &StereoCamera) {
        let (left, right) = camera.build_view_projection_matrices();
        self.view_proj_left = left.into();
        self.view_proj_right = right.into();
    }
}

pub enum EyeTarget {
    Left,
    Right,
}

/// A uniform struct to hold th eye target.
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RenderEyeTarget {
    /// -1 for left eye, 1 for right eye
    eye_target: f32,

    // padding to 16 bytes
    padding_0: f32,
    padding_1: f32,
    padding_2: f32,
}

impl RenderEyeTarget {
    pub fn new(target: EyeTarget) -> Self {
        Self {
            eye_target: match target {
                EyeTarget::Left => -1.0,
                EyeTarget::Right => 1.0,
            },
            padding_0: 0.0,
            padding_1: 0.0,
            padding_2: 0.0,
        }
    }
}
