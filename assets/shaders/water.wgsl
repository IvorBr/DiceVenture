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

    let world_from_local = get_world_from_local(input.instance_index);
    var wp = mesh_position_local_to_world(world_from_local, vec4<f32>(input.position, 1.0)).xyz;

    let amp = 0.1;
    let e0 = getElevation(wp.x, wp.z) * amp;
    wp.y += e0;

    let eps = 0.001;
    let ex = getElevation(wp.x + eps, wp.z) * amp;
    let ez = getElevation(wp.x, wp.z + eps) * amp;

    let dx = ex - e0;
    let dz = ez - e0;

    let N = normalize(vec3<f32>(-dx, 1.0, -dz));

    out.world_pos = wp;
    out.normal = N;
    out.uv = input.uv;
    out.clip_position = position_world_to_clip(wp);
    return out;
}

const DEBUG = false;

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    // Base water color
    var surfaceColor = vec3<f32>(0.0, 0.65, 0.9);
    var peakColor = vec3<f32>(1.0, 1.0, 1.0);

    let elevation = getElevation(input.world_pos.x, input.world_pos.z);
    let highlight = smoothstep(0.28, 0.36, elevation);
    let boosted = elevation * mix(1.0, 4.0, highlight);
    let elevationFactor = (boosted - 0.1) * 0.35;

    if (DEBUG) { surfaceColor = vec3(0.0); peakColor = vec3(1.0); }
    var color = mix(surfaceColor, peakColor, elevationFactor);

    // Reflection
    let clip = reflection.clip_from_world * vec4<f32>(input.world_pos, 1.0);
    let ndc  = clip.xy / clip.w;
    var uv   = ndc * 0.5 + vec2<f32>(0.5, 0.5);
    uv.y = 1.0 - uv.y;

    let refl = textureSample(terrain_texture, terrain_sampler, uv);
    let viewDir = normalize(view.world_position - input.world_pos);
    let ndotv = max(dot(input.normal, viewDir), 0.0);

    let N = normalize(input.normal);
    let V = normalize(view.world_position - input.world_pos);
    let ndv = clamp(dot(N, V), 0.0, 1.0);

    let base_refl = 0.8;   // reflection when looking straight down (tune 0.1–0.35)
    let grazing_boost = 0.80; // how much extra at grazing (tune 0.5–1.0)
    let power = 3.0;    // curve sharpness (2–5 looks good)

    let fresnel_like = base_refl + grazing_boost * pow(1.0 - ndv, power);
    var weight = clamp(fresnel_like, 0.0, 1.0);
    weight *= 0.8 + 0.2 * saturate(1.0 - abs(N.y)); 

    let mask = weight * refl.a;

    let final_rgb = mix(color, refl.rgb, mask);
    return vec4(final_rgb, 1.0);
}


// Still need to add
// Add foam at the waterlines using depth buffer
// Depth-based opacity
// let opacity = mix(0.7, 1.0, fresnel); // More transparent when looking down
// Add slight color tint to reflections for more realistic water
// finalColor = mix(finalColor, finalColor * vec3(0.9, 0.95, 1.0), 0.1);
// Boost contrast slightly for more vibrant look
// finalColor = pow(finalColor, vec3(0.95));