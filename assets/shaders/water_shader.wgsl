#import bevy_pbr::mesh_view_bindings::{globals, view}
#import bevy_pbr::view_transformations::position_world_to_clip

struct WaterMaterialUniform {
    base_color: vec4<f32>,
    wave_strength: f32,
};

@group(2) @binding(0)
var<uniform> water: WaterMaterialUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
};

fn wave_offset(x: f32, z: f32) -> f32 {
    return sin(x + globals.time) 
        * sin(z + globals.time)
        * water.wave_strength;
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let wave_offset = wave_offset(input.position.x, input.position.z);

    let displaced_pos = vec3<f32>(
        input.position.x,
        input.position.y + wave_offset,
        input.position.z,
    );

    out.world_pos = displaced_pos;
    out.clip_position = position_world_to_clip(out.world_pos.xyz);
    let fx = water.wave_strength *
            cos(input.position.x + globals.time) *
            sin(input.position.z + globals.time);
    let fz = water.wave_strength *
            sin(input.position.x + globals.time) *
            cos(input.position.z + globals.time);

    let wave_normal = normalize(vec3<f32>(-fx, 1.0, -fz));
    out.normal = wave_normal;
    out.uv = input.uv;

    return out;
}

fn hash2D(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let wave_offset = wave_offset(input.world_pos.x, input.world_pos.z);
    let base_color = water.base_color.rgb + vec3<f32>(wave_offset * 0.5);

    let normal = normalize(input.normal);
    let view_dir = normalize(view.world_position - input.world_pos);

    let fresnel_factor = 1.0 - dot(normal, view_dir);
    let fresnel_factor_pow = pow(fresnel_factor, 3.0);
    var fresnel_color = vec3<f32>(0.8, 0.9, 1.0) * 0.2 * fresnel_factor_pow;
    fresnel_color = vec3<f32>(0.0, 0.0, 0.0);
    let final_color =  fresnel_color + base_color;

    return vec4<f32>(final_color, water.base_color.a);
}
