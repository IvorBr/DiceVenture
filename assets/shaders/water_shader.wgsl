#import bevy_pbr::{
    mesh_view_bindings::{globals, view},
    view_transformations::position_world_to_clip,
    mesh_functions::{get_world_from_local, mesh_position_local_to_world}
}

struct WaterMaterialUniform {
    random_num: i32
};

@group(2) @binding(0)
var<uniform> water: WaterMaterialUniform;

@group(2) @binding(1) var depth_texture: texture_2d<f32>;
@group(2) @binding(2) var depth_sampler: sampler;

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

fn mod289(x: vec2f) -> vec2f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn mod289_3(x: vec3f) -> vec3f {
    return x - floor(x * (1. / 289.)) * 289.;
}

fn permute3(x: vec3f) -> vec3f {
    return mod289_3(((x * 34.) + 1.) * x);
}

fn simplexNoise2(v: vec2f) -> f32 {
    let C = vec4(
        0.211324865405187,
        0.366025403784439,
        -0.577350269189626,
        0.024390243902439 
    );

    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    var i1 = select(vec2(0., 1.), vec2(1., 0.), x0.x > x0.y);

    var x12 = x0.xyxy + C.xxzz;
    x12.x = x12.x - i1.x;
    x12.y = x12.y - i1.y;

    i = mod289(i);

    var p = permute3(permute3(i.y + vec3(0., i1.y, 1.)) + i.x + vec3(0., i1.x, 1.));
    var m = max(0.5 - vec3(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3(0.));
    m *= m;
    m *= m;

    let x = 2. * fract(p * C.www) - 1.;
    let h = abs(x) - 0.5;
    let ox = floor(x + 0.5);
    let a0 = x - ox;

    m *= 1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h);

    let g = vec3(a0.x * x0.x + h.x * x0.y, a0.yz * x12.xz + h.yz * x12.yw);
    return 130. * dot(m, g);
}

fn getElevation(x: f32, z: f32) -> f32 {
    let pos = vec2<f32>(x, z);

    var elevation: f32 = 0.0;
    var amplitude: f32 = 1.0;

    var wave_amplitude: f32 = 0.02;
    var frequency: f32 = 0.8;
    var speed: f32 = 0.25;
    var persisitence: f32 = 0.25;
    var lacunarity: f32 = 2.0;
    var iterations: u32 = 6;
    
    var p: vec2<f32> = pos.xy;

    for (var i: u32 = 0; i < iterations; i++) {
        let noise_value: f32 = simplexNoise2(p * frequency + speed * globals.time);
        elevation += amplitude * noise_value;
        amplitude *= persisitence; 
        frequency *= lacunarity;
    }

    elevation *= wave_amplitude;

    return elevation;
}

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    var world_from_local = get_world_from_local(input.instance_index);
    var world_pos : vec3<f32> = mesh_position_local_to_world(world_from_local, vec4<f32>(input.position, 1.0)).xyz;

    let elevation = getElevation(world_pos.x, world_pos.z);
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

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let troughColor = vec3<f32>(0.01, 0.15, 0.15);
    let surfaceColor = vec3<f32>(0.01, 0.3, 0.2);
    let peakColor = vec3<f32>(0.1, 0.4, 0.4);
    
    let troughThreshold :f32 = 0.0;
    let troughTransition :f32 = 0.3;

    let peakThreshold :f32 = 0.135;
    let peakTransition :f32 = 0.1;

    let fresnelScale : f32 = 0.2;
    let fresnelPower : f32 = 2.0;
    
    let elevation = input.world_pos.y;

    let troughFactor = smoothstep(
        troughThreshold - troughTransition,
        troughThreshold + troughTransition,
        elevation
    );

    let mixedColor1 = mix(troughColor, surfaceColor, troughFactor);
    
    let peakFactor = smoothstep(
        peakThreshold - peakTransition,
        peakThreshold + peakTransition,
        elevation
    );

    let mixedColor2 = mix(mixedColor1, peakColor, peakFactor);
    
    let camera_position = view.world_position;

    let viewDirection = normalize(input.world_pos - camera_position);

    let dotVN = dot(viewDirection, input.normal);
    let clampedDot = clamp(dotVN, 0.0, 1.0);
    let fresnel = fresnelScale * pow(1.0 - clampedDot, fresnelPower);

    let finalColor = mix(mixedColor2, vec3(0.5, 0.8, 0.9), fresnel);

    return vec4<f32>(finalColor, 0.4);
}
