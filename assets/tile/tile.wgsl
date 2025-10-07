#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip}
#import bevy_pbr::lighting::{light, light_is_directional}
#import bevy_pbr::mesh_view_bindings::view

struct VertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) local_position: vec3<f32>, // ローカル座標追加
    @location(3) uv: vec2<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world = get_world_from_local(in.instance_index);
    out.position = mesh_position_local_to_clip(world, vec4<f32>(in.position, 1.0));
    out.world_position = (world * vec4<f32>(in.position, 1.0)).xyz;
    out.world_normal = normalize((world * vec4<f32>(in.normal, 0.0)).xyz);
    out.local_position = in.position;
    out.uv = in.uv;

    return out;
}

struct Material {
    blend: vec4<f32>,
};
@group(#{MATERIAL_BIND_GROUP}) @binding(0) var texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var texture_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var<uniform> material: Material;

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    // 牌の背面色
    let back_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    // 牌のテクスチャ(背面以外の色)
    let texture_color = textureSampleBias(texture, texture_sampler, in.uv, -0.5);

    // カメラ方向（ワールド空間）
    let view_dir = normalize(view.world_position.xyz - in.world_position);
    // 法線との内積（1.0: 正面、0.0: 真横）
    let angle_factor = 1.0 - dot(in.world_normal, view_dir);
    // カメラと牌の距離&角度を考慮して背面色とテクスチャの境界部分のグラデーション比率を決定 (異方性フィルタリングの代用)
    let edge_thickness = mix(0.0005, 0.015, pow(saturate(angle_factor), 8));
    let z = in.local_position.z + 0.003;
    let blend_factor = clamp(z / edge_thickness, 0.0, 1.0);
    
    // 背面色とテクスチャを配合
    let base_color = mix(back_color, texture_color, blend_factor);

    // ライト（ワールド内の他のライトは無視して真上から平行光を照射）
    let light_dir = normalize(vec3<f32>(0.0, -1.0, 0.0));
    let normal = normalize(in.world_normal);
    // 拡散反射
    let diff = max(dot(normal, -light_dir), 0.0);
    let color = base_color * (0.4 + diff * 0.6);

    // ハイライト,グレーアウト用のblende色を配合
    let blended = mix(color.rgb, material.blend.rgb, material.blend.a);

    return vec4<f32>(blended, 1.0);
}