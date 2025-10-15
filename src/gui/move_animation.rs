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
    frame_left: u32,
}

impl MoveAnimation {
    pub fn new(target: Vec3) -> Self {
        Self {
            target,
            frame_left: 12,
        }
    }

    pub fn with_frame(mut self, frame: u32) -> Self {
        self.frame_left = frame;
        self
    }
}

fn move_animation(
    mut commands: Commands,
    move_animations: Query<(Entity, &mut Transform, &mut MoveAnimation)>,
) {
    for (entity, mut tf, mut anim) in move_animations {
        if anim.frame_left > 0 && anim.target != tf.translation {
            let d = 1.0 / anim.frame_left as f32;
            tf.translation = tf.translation.lerp(anim.target, d);
            anim.frame_left -= 1;
        } else {
            // 残りフレームが0または現在位置が移動先の場合はMoveAnimationを削除
            tf.translation = anim.target; // 小数点誤差削除用
            commands.entity(entity).remove::<MoveAnimation>();
        }
    }
}
