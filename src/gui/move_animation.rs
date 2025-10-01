use bevy::prelude::*;

pub struct MoveAnimationPlugin;

impl Plugin for MoveAnimationPlugin {
    fn build(&self, app: &mut App) {
        // StageのUpdateと同時実行するとremoveとinsertが重なって
        // insertしたばかりのMoveAnimationが削除されることがあるのでPostUpdateを使用
        app.add_systems(PostUpdate, move_animation);
    }
}

// 等速移動アニメーション
#[derive(Component, Debug)]
pub struct MoveAnimation {
    // 移動アニメーションの目標(終了)位置
    target: Vec3,
    // アニメーションの残りフレーム数
    // フレームごとに値を1づつ下げていき, 1/frame_left * (target - 現在位置)つづ移動
    // frame_left == 1のときはtargetをそのまま現在位置にセットしてanimationを終了 (= MoveAnimationを削除)
    frame_left: usize,
}

impl MoveAnimation {
    pub fn new(target: Vec3) -> Self {
        Self {
            target,
            frame_left: 12,
        }
    }
}

fn move_animation(
    mut commands: Commands,
    move_tos: Query<(Entity, &mut Transform, &mut MoveAnimation)>,
) {
    for (entity, mut tf, mut move_to) in move_tos {
        if move_to.frame_left > 1 {
            let diff_vec = move_to.target - tf.translation;
            tf.translation += 1.0 / move_to.frame_left as f32 * diff_vec;
            move_to.frame_left -= 1;
        } else {
            tf.translation = move_to.target;
            commands.entity(entity).remove::<MoveAnimation>();
        }
    }
}
