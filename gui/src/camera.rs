use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: orthographic_projection(800.0, 800.0),
        }
    }

    pub fn update_size(&mut self, width: f32, height: f32) {
        self.view_proj = orthographic_projection(width, height);
    }
}

fn orthographic_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    // Create an orthographic projection matrix that maps:
    // x: [0, width] -> [-1, 1]
    // y: [0, height] -> [1, -1] (flip y to have origin at top-left)
    let a = 2.0 / width;
    let b = -2.0 / height;
    let c = -1.0;
    let d = 1.0;

    [
        [a, 0.0, 0.0, 0.0],
        [0.0, b, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [c, d, 0.0, 1.0],
    ]
}
