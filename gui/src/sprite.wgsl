struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct InstanceInput {
    @location(2) inst_position: vec2<f32>,
    @location(3) inst_size: vec2<f32>,
    @location(4) inst_uv_offset: vec2<f32>,
    @location(5) inst_uv_size: vec2<f32>,
    @location(6) inst_color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Transform vertex position by instance position and size
    let world_pos = instance.inst_position + model.position * instance.inst_size;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);
    
    // Transform texture coordinates
    out.tex_coords = instance.inst_uv_offset + model.tex_coords * instance.inst_uv_size;
    out.color = instance.inst_color;
    
    return out;
}

@group(0) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(2)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    return tex_color * in.color;
}