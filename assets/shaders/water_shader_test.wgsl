//#import bevy_pbr::mesh_view_bindings::globals

struct WaterMaterial {
    base_color: vec4<f32>,
};
@group(2) @binding(0) var<uniform> water: WaterMaterial;

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(input: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = vec4<f32>(input.position, 1.0); // Basic transformation
    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return water.base_color; // Simple solid color output
}
