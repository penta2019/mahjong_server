use std::f32::consts::PI;

use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, WindowFocused},
};

use super::{debug::DebugState, menu::MenuState};

pub struct ControlPlugin;

impl Plugin for ControlPlugin {
    fn build(&self, app: &mut App) {
        let mut ctx = ControlContext::default();
        ctx.set_mouse_sensitivity(30.0);

        app.insert_state(FlyState::Off)
            .insert_resource(ctx)
            .add_event::<CameraEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    keyboard_handler_global,
                    camera_event,
                    (
                        camera_move_by_keyboard,
                        camera_look_by_mouse,
                        window_focus_handler,
                    )
                        .run_if(in_state(FlyState::On)),
                ),
            );
    }
}

#[derive(Component, Debug)]
pub struct FlyCamera;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FlyState {
    On,
    Off,
}

#[derive(Resource, Debug)]
pub struct ControlContext {
    can_fly: bool,
    sensitivity: (f32, f32), // (value, percent)
    yaw: f32,                // [-180., 180) 度数法
    pitch: f32,              // [-89.9, 89.9] 度数法
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
            can_fly: false,
            sensitivity: (0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

#[derive(Event, Debug)]
pub struct CameraEvent {
    pub translation: Vec3,
    pub yaw: f32,   // [-180., 180) 度数法
    pub pitch: f32, // [-89.9, 89.9] 度数法
}

impl CameraEvent {
    pub fn new(pos: Vec3, look_at: Vec3) -> Self {
        let dir = (look_at - pos).normalize();

        let yaw = dir.x.atan2(dir.z) - PI; // Y軸まわり
        let pitch = dir.y.atan2((dir.x * dir.x + dir.z * dir.z).sqrt()); // X軸まわり

        Self {
            translation: pos,
            yaw,
            pitch,
        }
    }
}

fn set_cursor_visibility(window: &mut Window, visible: bool) {
    if visible {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    } else {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
    }
}

fn setup(mut commands: Commands, mut context: ResMut<ControlContext>) {
    let mut tf_camera = Transform::from_translation(Vec3::new(0., 5., 5.));
    context.set_pitch(-30.0);
    tf_camera.rotation = context.camera_rotation();
    commands.spawn((Camera3d::default(), tf_camera, FlyCamera));
}

fn keyboard_handler_global(
    keys: Res<ButtonInput<KeyCode>>,
    mut context: ResMut<ControlContext>,
    mut next_fly_state: ResMut<NextState<FlyState>>,
    debug_state: Res<State<DebugState>>,
    mut next_debug_state: ResMut<NextState<DebugState>>,
    menu_state: Res<State<MenuState>>,
    mut next_menu_state: ResMut<NextState<MenuState>>,
    mut window: Single<&mut Window>,
) {
    let mut set_fly_state = |enable| {
        if enable {
            next_fly_state.set(FlyState::On);
            set_cursor_visibility(&mut window, false);
        } else {
            next_fly_state.set(FlyState::Off);
            set_cursor_visibility(&mut window, true);
        }
    };

    if keys.just_pressed(KeyCode::Tab) {
        next_debug_state.set(if **debug_state == DebugState::On {
            DebugState::Off
        } else {
            DebugState::On
        });
    }

    if keys.just_pressed(KeyCode::Space) {
        context.can_fly = !context.can_fly;
        set_fly_state(context.can_fly);
    }

    if keys.just_pressed(KeyCode::Escape) {
        match **menu_state {
            MenuState::On => {
                // メニューを閉じる State: Disabled -> On, Off -> Off
                next_menu_state.set(MenuState::Off);
                set_fly_state(context.can_fly);
            }
            MenuState::Off => {
                // メニューを開く State: On -> Disabled, Off -> Off
                next_menu_state.set(MenuState::On);
                set_fly_state(false);
            }
        }
    }
}

fn camera_event(
    mut reader: EventReader<CameraEvent>,
    mut context: ResMut<ControlContext>,
    mut camera: Single<&mut Transform, With<Camera>>,
) {
    for ev in reader.read() {
        println!("{ev:?}");
        camera.translation = ev.translation;
        context.set_yaw(ev.yaw);
        context.set_pitch(ev.pitch);
        camera.rotation = context.camera_rotation();
    }
}

fn camera_move_by_keyboard(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Single<&mut Transform, With<Camera>>,
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
        // flyモードのまま他のアプリケーションにフォーカスが移動した場合の処理
        set_cursor_visibility(&mut window, !event.focused);
    }
}
