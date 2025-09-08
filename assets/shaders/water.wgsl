#import bevy_pbr::{
    mesh_view_bindings::{globals, view},
    view_transformations::position_world_to_clip,
    mesh_functions::{get_world_from_local, mesh_position_local_to_world}
}

#import "shaders/noise.wgsl"::simplex_noise2;


@group(2) @binding(1) var terrain_texture: texture_2d<f32>;
@group(2) @binding(2) var terrain_sampler: sampler;

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

fn quantize_vec2(v: vec2<f32>, steps: f32) -> vec2<f32> {
    return floor(v * steps) / steps;
}

fn simplex01(n: f32) -> f32 { return n * 0.5 + 0.5; }

const PATTERN_SCALE = 10.0;
const WAVE_SPEED = 8.0;
fn getElevation(x: f32, z: f32) -> f32 {
    let pos = quantize_vec2(vec2<f32>(x, z),PATTERN_SCALE) * 8;
    let time = globals.time;

    // Scroll - wave
    var a_pos = vec2<f32>(pos.x * 0.05, pos.y * 0.5) + vec2<f32>(0.0, 0.03) * time * WAVE_SPEED;
    let a_noise = simplex01(simplex_noise2(a_pos));
    let a_term = (a_noise - 0.5) * 0.3;

    // Wave - bottom right
    var b_pos = vec2<f32>(pos.x * 0.2, pos.y * 0.4)+ vec2<f32>(-0.05, -0.05) * time * WAVE_SPEED;
    let b_noise = simplex01(simplex_noise2(b_pos));
    let b_term = (b_noise - 0.5) * 0.5;

    var add = a_term + b_term;
    return add;
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var world_from_local = get_world_from_local(input.instance_index);
    var world_pos : vec3<f32> = mesh_position_local_to_world(world_from_local, vec4<f32>(input.position, 1.0)).xyz;

    let elevation = getElevation(world_pos.x, world_pos.z) * 0.1;
    world_pos.y += elevation;

    let eps: f32 = 0.001;
    let tangent = normalize(vec3<f32>(
        eps, 
        getElevation(world_pos.x - eps, world_pos.z) - elevation, 
        0.0
    ));

    let bitangent = normalize(vec3<f32>(
        0.0, 
        getElevation(world_pos.x, world_pos.z - eps) - elevation, 
        eps
    ));

    let objectNormal = normalize(cross(tangent, bitangent));

    out.world_pos = world_pos;
    out.normal = objectNormal;
    out.uv = input.uv;
    out.clip_position = position_world_to_clip(world_pos);

    return out;
}

const DEBUG = false;

fn ndc_to_uv(ndc_xy: vec2<f32>) -> vec2<f32> {
    let uv = ndc_xy * 0.5 + 0.5;
    return vec2<f32>(uv.x, 1.0 - uv.y);
}

fn screen_uv_from_clip(clip: vec4<f32>) -> vec2<f32> {
    let ndc = clip.xy / clip.w;                     // [-1,1]
    let uv  = ndc * 0.5 + vec2<f32>(0.5, 0.5);      // [0,1]
    // if the image is upside-down, flip Y here:
    // return vec2<f32>(uv.x, 1.0 - uv.y);
    return uv;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let uv = screen_uv_from_clip(input.clip_position);
    // DEBUG: show the capture texture
    let c = textureSampleLevel(terrain_texture, terrain_sampler, input.world_pos.xz, 0.0);
    // If RGB looks dark because you clear to transparent, try viewing alpha:
    // return vec4(c.aaa, 1.0);
    return vec4(c.rgb, 1.0);
}

// @fragment
// fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
//     var surfaceColor = vec3<f32>(0.0, 0.65, 0.9);
//     var peakColor = vec3<f32>(1.0, 1.0, 1.0);

//     let elevation = getElevation(input.world_pos.x, input.world_pos.z);
//     let highlight = smoothstep(0.28, 0.36, elevation);
//     let boosted = elevation * mix(1.0, 4.0, highlight);
//     let elevationFactor = (boosted - 0.1) * 0.35;

//     if (DEBUG) { surfaceColor = vec3(0.0); peakColor = vec3(1.0); }
//     var color = mix(surfaceColor, peakColor, elevationFactor);

//     return vec4(color, 0.92);
// }
