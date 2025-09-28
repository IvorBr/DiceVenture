#import bevy_pbr::{
    mesh_view_bindings::{globals, view},
    view_transformations::{position_world_to_clip, position_world_to_view, view_z_to_depth_ndc},
    mesh_functions::{get_world_from_local, mesh_position_local_to_world},
    prepass_utils::prepass_depth
}

#import "shaders/noise.wgsl"::simplex_noise2;

@group(2) @binding(0) var<uniform> reflection: ReflectionParams;
@group(2) @binding(1) var terrain_texture: texture_2d<f32>;
@group(2) @binding(2) var terrain_sampler: sampler;

struct ReflectionParams {
    clip_from_world: mat4x4<f32>,
}

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
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

fn quantize_vec3(v: vec3<f32>, steps: f32) -> vec3<f32> {
    return floor(v * steps) / steps;
}

fn simplex01(n: f32) -> f32 { return n * 0.5 + 0.5; }

const PATTERN_SCALE = 10.0;
const WAVE_SPEED = 8.0;
fn getElevation(wp: vec3<f32>) -> f32 {
    let pos = quantize_vec3(wp, PATTERN_SCALE) * 8;
    let time = globals.time;

    // Scroll - wave
    var a_pos = vec2<f32>(pos.x * 0.05, pos.z * 0.5) + vec2<f32>(0.0, 0.03) * time * WAVE_SPEED;
    let a_noise = simplex01(simplex_noise2(a_pos));
    let a_term = (a_noise - 0.5) * 0.3;

    // Wave - bottom right
    var b_pos = vec2<f32>(pos.x * 0.2, pos.z * 0.4)+ vec2<f32>(-0.05, -0.05) * time * WAVE_SPEED;
    let b_noise = simplex01(simplex_noise2(b_pos));
    let b_term = (b_noise - 0.5) * 0.5;

    var add = a_term + b_term;

    return add;
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_from_local = get_world_from_local(input.instance_index);
    var wp = mesh_position_local_to_world(world_from_local, vec4<f32>(input.position, 1.0)).xyz;

    let amp = 0.1;
    let e0 = getElevation(wp) * amp;
    //wp.y += e0;

    // let eps = 0.001;
    // let ex = getElevation(wp.x + eps, wp.z, 0) * amp;
    // let ez = getElevation(wp.x, wp.z + eps, 0) * amp;

    // let dx = ex - e0;
    // let dz = ez - e0;
    // let N = normalize(vec3<f32>(-dx, 1.0, -dz));
    
    // out.normal = N;

    out.world_pos = wp;
    out.uv = input.uv;
    out.clip_position = position_world_to_clip(wp + vec3(0.0, e0, 0.0));
    return out;
}

const DEBUG = false;

@fragment
fn fragment(input: VertexOutput, @builtin(sample_index) si: u32) -> @location(0) vec4<f32> {
    // Base water color
    var deepColor = vec3<f32>(0.0, 0.33, 0.51);
    var surfaceColor = vec3<f32>(0.0, 0.65, 0.9);
    var peakColor = vec3<f32>(1.0, 1.0, 1.0);

    let elevation = getElevation(input.world_pos);
    let highlight = smoothstep(0.28, 0.36, elevation);
    let boosted = elevation * mix(1.0, 4.0, highlight);
    let elevationFactor = (boosted - 0.1) * 0.35;

    if (DEBUG) { surfaceColor = vec3(0.0); peakColor = vec3(1.0); }
    var color = mix(surfaceColor, peakColor, elevationFactor);

    // Water depth
    let scene_d = prepass_depth(input.clip_position, si);
    let view_pos = position_world_to_view(input.world_pos);
    let water_d = view_z_to_depth_ndc(view_pos.z);

    var thickness = max(0.0, scene_d+0.0012 - water_d);
    thickness = smoothstep(0.00, 0.002, thickness);

    thickness = pow(thickness, 1.2);
    
    // let quant_wp = quantize_vec3(input.world_pos, PATTERN_SCALE);
    // let quant_scene_d = prepass_depth(position_world_to_clip(quant_wp), si);
    // let quant_view_pos = position_world_to_view(quant_wp);
    // let quant_water_d = view_z_to_depth_ndc(quant_view_pos.z);

    // var quant_thickness = max(0.0, quant_scene_d+0.0012 - quant_water_d);
    // quant_thickness = smoothstep(0.00, 0.002, thickness);
    // quant_thickness = pow(quant_thickness, 1.2);

    // Foam
    if thickness > 0.58 {
        return vec4(0.8);
    }

    //return vec4(vec3(1.0)*scene_d, 1.0);
    return vec4(color, 1.0 - thickness);
}


// Still need to add
// finalColor = mix(finalColor, finalColor * vec3(0.9, 0.95, 1.0), 0.1);
// Boost contrast slightly for more vibrant look
// finalColor = pow(finalColor, vec3(0.95));

//get the thickness of the same quantized position used to get the elevation