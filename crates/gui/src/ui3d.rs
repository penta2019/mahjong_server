use bevy::{
    camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
    prelude::*,
    render::render_resource::TextureFormat,
};

pub const UI3D_LAYER: RenderLayers = RenderLayers::layer(1);

#[derive(Resource, Clone)]
struct Config {
    width: u32,
    height: u32,
    scale: f32,
}

pub struct Ui3dPlugin {
    config: Config,
}

impl Ui3dPlugin {
    pub fn new(width: u32, height: u32, scale: f32) -> Self {
        Self {
            config: Config {
                width,
                height,
                scale,
            },
        }
    }
}

impl Plugin for Ui3dPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .add_systems(Startup, setup)
            .add_systems(PostUpdate, update_transform);
    }
}

#[derive(Component)]
pub struct Ui3dCamera;

#[derive(Component)]
pub struct Ui3dViewport;

#[derive(Component, Debug)]
pub struct Ui3dTransform {
    node: Entity,
    rotation: Quat,
    scale: Vec3,
}

impl Ui3dTransform {
    pub fn new(node: Entity, rotation: Quat, scale: Vec3) -> Self {
        Self {
            node,
            rotation,
            scale,
        }
    }
}

fn setup(mut cmd: Commands, mut images: ResMut<Assets<Image>>, config: Res<Config>) {
    // 3DオブジェクトをUIにオーバーレイするためのテクスチャとカメラ
    let image =
        Image::new_target_texture(config.width, config.height, TextureFormat::bevy_default());
    let image = images.add(image);
    cmd.spawn((
        Ui3dCamera,
        Camera3d::default(),
        Camera {
            order: 1,
            clear_color: ClearColorConfig::Custom(Color::NONE),
            target: RenderTarget::Image(image.clone().into()),
            ..default()
        },
        Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
        Projection::from(OrthographicProjection {
            scaling_mode: ScalingMode::WindowSize,
            scale: config.scale,
            ..OrthographicProjection::default_2d()
        }),
        UI3D_LAYER,
    ));
    // 当たり判定のない透明なテクスチャで画面全体を覆う
    cmd.spawn((
        Ui3dViewport,
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        ImageNode { image, ..default() },
        ZIndex(1),
        Pickable::IGNORE,
    ));
}

fn update_transform(
    mut cmd: Commands,
    config: Res<Config>,
    viewport_node: Single<&ComputedNode, With<Ui3dViewport>>,
    camera: Single<(&GlobalTransform, &Camera), With<Ui3dCamera>>,
    node_tfs: Query<&UiGlobalTransform, With<Node>>,
    ui3d_tfs: Query<(Entity, &Ui3dTransform)>,
) {
    if ui3d_tfs.is_empty() {
        return;
    }

    for (entity, ui3d_tfs) in ui3d_tfs {
        let Ok(node_tf) = node_tfs.get(ui3d_tfs.node) else {
            continue;
        };
        let node_xy = node_tf.translation;
        // UIの座標計算が完了する前に呼ばれることがあるのでその対策
        if node_xy == Vec2::new(0.0, 0.0) {
            // 正しい位置に移動する前のエンティティが一瞬描画されるのを防ぐ
            // Visibility::Hiddenや描画範囲外に移動するとbevyのバグでクラッシュするのでscaleを0にする
            // おそらく,モデル読み込み,又はカスタムシェーダー周りの不具合 (bevy 0.17.2)
            cmd.entity(entity)
                .insert(Transform::from_scale(Vec3::splat(0.0)));
            continue;
        }

        // viewport座標 -> render target座標
        let rt_xy = Vec2::new(
            node_xy.x / viewport_node.size.x * config.width as f32,
            node_xy.y / viewport_node.size.y * config.height as f32,
        );

        let (camera_tf, camera) = &*camera;
        let ray = camera.viewport_to_world(camera_tf, rt_xy);
        let xyz = ray.unwrap().origin; // z軸方向の垂直投影なのでrayの起点のx,yをそのまま取得すればOK

        // println!("wn_xy: {:?}", node_tf.translation);
        // println!("rt_xy: {:?}", rt_xy);
        // println!("xyz: {}", xyz);

        cmd.entity(entity).insert(Transform {
            translation: Vec3::new(xyz.x, xyz.y, 0.0),
            rotation: ui3d_tfs.rotation,
            scale: ui3d_tfs.scale,
        });

        cmd.entity(entity).remove::<Ui3dTransform>();
    }
}
