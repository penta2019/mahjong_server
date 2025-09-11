use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, WindowFocused},
};

use super::{debug::DebugState, menu::MenuState};

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlState {
    On,
    Off,
}

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        let mut ctx = ControlContext::default();
        ctx.set_mouse_sensitivity(30.0);

        app.insert_state(ControlState::On)
            .insert_resource(ctx)
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    keyboard_handler_global,
                    (keyboard_handler, mouse_look_camera, window_focus_handler)
                        .run_if(in_state(ControlState::On)),
                ),
            );
    }
}

#[derive(Resource, Debug)]
pub struct ControlContext {
    sensitivity: (f32, f32), // (value, percent)
    yaw: f32,
    pitch: f32,
}

impl ControlContext {
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
        while yaw < -180.0 {
            yaw += 360.0;
        }
        while yaw > 180.0 {
            yaw -= 360.0;
        }
        self.yaw = yaw;
    }

    fn set_pitch(&mut self, pitch: f32) {
        self.pitch = pitch.clamp(-89.9, 89.9);
    }

    fn camera_rotation(&self) -> Quat {
        let yaw_radians = self.yaw.to_radians();
        let pitch_radians = self.pitch.to_radians();
        Quat::from_axis_angle(Vec3::Y, yaw_radians) * Quat::from_axis_angle(Vec3::X, pitch_radians)
    }
}

impl Default for ControlContext {
    fn default() -> Self {
        Self {
            sensitivity: (0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

fn hide_cursor(window: &mut Window, value: bool) {
    if value {
        // ウィンドウがアクティブになったときの処理
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    } else {
        // ウィンドウが非アクティブになったときの処理
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

#[derive(Component, Debug)]
pub struct FlyCamera;

fn setup(mut commands: Commands, mut context: ResMut<ControlContext>) {
    let camera_pos = Vec3::new(0.0, 0.45, 0.30);
    let mut transform = Transform::from_translation(camera_pos);
    // let (yaw, pitch) = calc_yaw_pitch(camera_pos, Vec3::ZERO);
    // context.set_yaw(yaw.to_degrees());
    // context.set_pitch(pitch.to_degrees());
    context.set_pitch(-60.0);
    transform.rotation = context.camera_rotation();
    commands.spawn((Camera3d::default(), transform, FlyCamera));
}

fn keyboard_handler_global(
    keys: Res<ButtonInput<KeyCode>>,
    // state: Res<State<ControlState>>,
    mut next_state: ResMut<NextState<ControlState>>,
    debug_state: Res<State<DebugState>>,
    mut next_debug_state: ResMut<NextState<DebugState>>,
    menu_state: Res<State<MenuState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut window: Single<&mut Window>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        next_debug_state.set(if **debug_state == DebugState::On {
            DebugState::Off
        } else {
            DebugState::On
        });
    }

    if keys.just_pressed(KeyCode::Escape) {
        match **menu_state {
            MenuState::On => {
                next_menu_state.set(MenuState::Off);
                next_state.set(ControlState::On);
                hide_cursor(&mut window, true);
            }
            MenuState::Off => {
                next_menu_state.set(MenuState::On);
                next_state.set(ControlState::Off);
                hide_cursor(&mut window, false);
            }
        }
    }
}

fn keyboard_handler(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cam_q: Single<&mut Transform, With<Camera>>,
) {
    // カメラのローカル基準方向
    let mut forward = cam_q.forward().as_vec3(); // -Z 方向（回転を考慮）
    forward.y = 0.0;
    forward = forward.normalize();
    let mut right = cam_q.right().as_vec3(); // +X 方向（回転を考慮）
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
        cam_q.translation += dir.normalize() * speed * time.delta_secs();
    }
}

fn mouse_look_camera(
    mut mouse_events: EventReader<MouseMotion>,
    mut context: ResMut<ControlContext>,
    mut camera: Single<&mut Transform, With<FlyCamera>>,
) {
    if mouse_events.is_empty() {
        return;
    }

    // マウス移動量を集計
    let mut delta = Vec2::ZERO;
    for ev in mouse_events.read() {
        delta += ev.delta;
    }
    if delta == Vec2::ZERO {
        return;
    }

    // 感度を反映
    let yaw = context.yaw() - delta.x * context.sensitivity.0;
    context.set_yaw(yaw);
    let pitch = context.pitch() - delta.y * context.sensitivity.0;
    context.set_pitch(pitch);
    // 回転を適用
    camera.rotation = context.camera_rotation();
}

fn window_focus_handler(
    mut focus_events: EventReader<WindowFocused>,
    mut window: Single<&mut Window>,
) {
    if focus_events.is_empty() {
        return;
    }

    for event in focus_events.read() {
        hide_cursor(&mut window, event.focused);
    }
}

fn calc_yaw_pitch(cam_pos: Vec3, target: Vec3) -> (f32, f32) {
    let dir = (target - cam_pos).normalize();

    let yaw = dir.x.atan2(dir.z) - PI; // Y軸まわり
    let pitch = dir.y.atan2((dir.x * dir.x + dir.z * dir.z).sqrt()); // X軸まわり

    (yaw, pitch)
}
