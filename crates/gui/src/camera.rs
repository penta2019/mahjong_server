use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::{
    camera::Viewport,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused, WindowResized},
};

#[derive(Debug, Clone, Copy)]
pub enum ViewportMode {
    FullSize,
    AspectRatio(f32),
}

pub struct CameraPlugin {
    mode: ViewportMode,
}

impl CameraPlugin {
    pub fn new(mode: ViewportMode) -> Self {
        Self { mode }
    }
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        let mut ctx = CameraContext {
            mode: self.mode,
            ..default()
        };
        ctx.set_mouse_sensitivity(30.0);

        app.insert_state(CameraState::Fix)
            .insert_resource(ctx)
            .add_message::<CameraMove>()
            .add_systems(Startup, setup)
            .add_systems(OnEnter(CameraState::Fly), state_fly)
            .add_systems(OnEnter(CameraState::Fix), state_fix)
            .add_systems(
                Update,
                (
                    camera_move_by_keyboard,
                    camera_look_by_mouse,
                    window_focus_handler,
                )
                    .run_if(in_state(CameraState::Fly)),
            )
            .add_systems(PostUpdate, (update_viewport, camera_event));
    }
}

#[derive(Component, Debug)]
pub struct MainCamera;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraState {
    Fly,
    Fix,
}

#[derive(Resource, Debug)]
pub struct CameraContext {
    mode: ViewportMode,
    sensitivity: (f32, f32), // (value, percent)
    yaw: f32,                // [-PI, PI)
    pitch: f32,              // (-FRAC_PI_2, FRAC_PI_2)
}

impl CameraContext {
    pub fn set_mouse_sensitivity(&mut self, percent: f32) {
        // 0% -> 10%(0.01), 100% -> 100%(0.1)
        self.sensitivity = (0.1 * (0.1 + 0.9 * percent / 100.0), percent);
    }

    pub fn get_mouse_sensitivity(&self) -> f32 {
        self.sensitivity.1
    }

    pub fn yaw(&self) -> f32 {
        self.yaw
    }

    pub fn pitch(&self) -> f32 {
        self.pitch
    }

    fn set_yaw(&mut self, mut yaw: f32) {
        // [-PI, PI)の範囲に収まるように修正
        while yaw < -PI {
            yaw += TAU;
        }
        while yaw >= PI {
            yaw -= TAU;
        }
        self.yaw = yaw;
    }

    fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(-FRAC_PI_2 + 0.001, FRAC_PI_2 - 0.001);
    }

    fn camera_rotation(&self) -> Quat {
        Quat::from_axis_angle(Vec3::Y, self.yaw) * Quat::from_axis_angle(Vec3::X, self.pitch)
    }
}

impl Default for CameraContext {
    fn default() -> Self {
        Self {
            mode: ViewportMode::FullSize,
            sensitivity: (0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Message, Debug)]
pub struct CameraMove {
    translation: Vec3,
    yaw: f32,   // [-PI, PI)
    pitch: f32, // (-FRAC_PI_2, FRAC_PI_2)
}

impl CameraMove {
    pub fn new(pos: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            translation: pos,
            yaw,
            pitch,
        }
    }

    pub fn look(pos: Vec3, look_at: Vec3) -> Self {
        let dir = (look_at - pos).normalize();
        let yaw = dir.x.atan2(dir.z) - PI; // Y軸まわり
        let pitch = dir.y.atan2((dir.x * dir.x + dir.z * dir.z).sqrt()); // X軸まわり
        Self::new(pos, yaw, pitch)
    }
}

fn setup(mut cmd: Commands, mut context: ResMut<CameraContext>) {
    context.set_pitch(-45.0_f32.to_radians());
    let tf_camera = Transform {
        translation: Vec3::new(0.0, 1.0, 1.0),
        rotation: context.camera_rotation(),
        scale: Vec3::ONE,
    };
    cmd.spawn((
        MainCamera,
        Camera3d::default(),
        Projection::from(PerspectiveProjection {
            fov: 20.0_f32.to_radians(),
            ..default()
        }),
        tf_camera,
        Msaa::Sample8,
    ));
}

fn state_fly(mut options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_visibility(&mut options, false);
}

fn state_fix(mut options: Single<&mut CursorOptions, With<PrimaryWindow>>) {
    set_cursor_visibility(&mut options, true);
}

fn update_viewport(
    resize_reader: MessageReader<WindowResized>,
    context: ResMut<CameraContext>,
    mut camera: Single<&mut Camera, With<MainCamera>>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if resize_reader.is_empty() {
        return;
    }

    let ViewportMode::AspectRatio(target_aspect) = context.mode else {
        return;
    };
    let window_aspect = window.width() / window.height();

    if (window_aspect - target_aspect).abs() < f32::EPSILON {
        // 比率が同じ → フルスクリーン使用
        camera.viewport = None;
    } else if window_aspect > target_aspect {
        // 横長 → 左右に黒帯
        let height = window.height() as u32;
        let width = (window.height() * target_aspect) as u32;
        let x_offset = (window.width() as u32 - width) / 2;
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(x_offset, 0),
            physical_size: UVec2::new(width, height),
            ..default()
        });
    } else {
        // 縦長 → 上下に黒帯
        let width = window.width() as u32;
        let height = (window.width() / target_aspect) as u32;
        let y_offset = ((window.height() as u32) - height) / 2;
        camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, y_offset),
            physical_size: UVec2::new(width, height),
            ..default()
        });
    }
}

fn camera_event(
    mut reader: MessageReader<CameraMove>,
    mut context: ResMut<CameraContext>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    for ev in reader.read() {
        camera.translation = ev.translation;
        context.set_yaw(ev.yaw);
        context.set_pitch(ev.pitch);
        camera.rotation = context.camera_rotation();
    }
}

fn camera_move_by_keyboard(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    // カメラのローカル基準方向
    let mut forward = camera.forward().as_vec3(); // -Z 方向（回転を考慮）
    forward.y = 0.0;
    forward = forward.normalize();
    let mut right = camera.right().as_vec3(); // +X 方向（回転を考慮）
    right.y = 0.0;
    right = right.normalize();
    let up = Vec3::Y; // ワールド上方向

    let mut dir = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        dir += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        dir -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        dir += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        dir -= right;
    }
    if keys.pressed(KeyCode::KeyE) {
        dir += up;
    }
    if keys.pressed(KeyCode::KeyQ) {
        dir -= up;
    }

    if dir != Vec3::ZERO {
        let mut speed = 0.1;
        if keys.pressed(KeyCode::ShiftLeft) {
            speed *= 4.0; // ダッシュ
        }
        camera.translation += dir.normalize() * speed * time.delta_secs();
    }
}

fn camera_look_by_mouse(
    mut mouse_motion: MessageReader<MouseMotion>,
    mut context: ResMut<CameraContext>,
    mut camera: Single<&mut Transform, With<MainCamera>>,
) {
    if mouse_motion.is_empty() {
        return;
    }

    // マウス移動量を集計
    let mut delta = Vec2::ZERO;
    for ev in mouse_motion.read() {
        delta += ev.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    // 感度を反映
    let yaw = context.yaw() - (delta.x * context.sensitivity.0).to_radians();
    context.set_yaw(yaw);
    let pitch = context.pitch() - (delta.y * context.sensitivity.0).to_radians();
    context.set_pitch(pitch);
    // 回転を適用
    camera.rotation = context.camera_rotation();
}

fn window_focus_handler(
    mut window_focused: MessageReader<WindowFocused>,
    mut options: Single<&mut CursorOptions, With<PrimaryWindow>>,
) {
    if window_focused.is_empty() {
        return;
    }

    for event in window_focused.read() {
        // flyモードのまま他のアプリケーションにフォーカスが移動した場合の処理
        set_cursor_visibility(&mut options, !event.focused);
    }
}

fn set_cursor_visibility(options: &mut CursorOptions, visible: bool) {
    if visible {
        options.visible = true;
        options.grab_mode = CursorGrabMode::None;
    } else {
        options.visible = false;
        options.grab_mode = CursorGrabMode::Locked;
    }
}
