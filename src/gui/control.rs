#[cfg(not(target_arch = "wasm32"))]
use bevy::pbr::wireframe::WireframeConfig;
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
        ctx.pitch = -25.0;

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

#[derive(Resource)]
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

#[derive(Component)]
pub struct FlyCamera;

fn setup(mut commands: Commands, context: Res<ControlContext>) {
    let mut transform = Transform::from_xyz(0.0, 0.1, 0.2);
    transform.rotation = context.camera_rotation();
    commands.spawn((Camera3d::default(), transform, FlyCamera, Msaa::Sample4));
}

fn keyboard_handler_global(
    keys: Res<ButtonInput<KeyCode>>,
    // state: Res<State<ControlState>>,
    mut next_state: ResMut<NextState<ControlState>>,
    debug_state: Res<State<DebugState>>,
    mut next_debug_state: ResMut<NextState<DebugState>>,
    menu_state: Res<State<MenuState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut window: Single<&mut Window>,
) {
    if keys.just_pressed(KeyCode::Space) {
        wireframe_config.global = !wireframe_config.global;
    }

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
    let forward = cam_q.forward().as_vec3(); // -Z 方向（回転を考慮）
    let right = cam_q.right().as_vec3(); // +X 方向（回転を考慮）
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
    context.yaw -= delta.x * context.sensitivity.0;
    context.pitch -= delta.y * context.sensitivity.0;
    // ピッチ制限（上下見過ぎ防止）
    context.pitch = context.pitch.clamp(-89.9, 89.9);
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
