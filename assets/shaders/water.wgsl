#import bevy_pbr::{
    mesh_view_bindings::{globals, view},
    view_transformations::position_world_to_clip,
    mesh_functions::{get_world_from_local, mesh_position_local_to_world}
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

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Project water world pos into the reflection camera
    let clip = position_world_to_clip(input.world_pos);//reflection.clip_from_world * vec4<f32>(input.world_pos, 1.0);
    let ndc = clip.xy / clip.w;

    var uv = ndc * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let col = textureSample(terrain_texture, terrain_sampler, uv);
    
    return col;
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
