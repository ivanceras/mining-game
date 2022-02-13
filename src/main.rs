use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use dolly::prelude::{CameraRig, Position, Smooth, YawPitch};

const ISOMETRIC_VIEW_YAW: f32 = 0.0;
const ISOMETRIC_VIEW_PITCH: f32 = -60.0;
const ISOMETRIC_VIEWING_HEIGHT: f32 = 20.0; //20m vantage point

const FPS_VIEW_YAW: f32 = 0.0;
const FPS_VIEW_PITCH: f32 = 0.0;
const FPS_VIEWING_HEIGHT: f32 = 2.0; // height of the model

/// flat whether to use iso metric view like in popular MOBAs
/// otherwise use first person shooter
const USE_ISOMETRIC_VIEW: bool = false;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(setup_camera)
        .add_system(drive_camera)
        .run();
}

#[derive(Component)]
struct DefaultCamera;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 20.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            ..Default::default()
        }),
        ..Default::default()
    });

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(0., 2.5, 10.).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(DefaultCamera);
}

fn setup_camera(mut commands: Commands) {
    let (yaw, pitch) = if USE_ISOMETRIC_VIEW {
        (ISOMETRIC_VIEW_YAW, ISOMETRIC_VIEW_PITCH)
    } else {
        (FPS_VIEW_YAW, FPS_VIEW_PITCH)
    };

    let viewing_height = if USE_ISOMETRIC_VIEW {
        ISOMETRIC_VIEWING_HEIGHT
    } else {
        FPS_VIEWING_HEIGHT
    };

    // Not required, just a nice camera driver to give easy, smooth, camera controls.
    let camera_rig = CameraRig::builder()
        .with(Position::new(dolly::glam::Vec3::new(
            0.0,
            viewing_height,
            4.0,
        )))
        .with(YawPitch::new().yaw_degrees(yaw).pitch_degrees(pitch))
        .with(Smooth::new_position_rotation(1.0, 1.0))
        .build();

    commands.insert_resource(camera_rig);
}

fn drive_camera(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut camera_rig: ResMut<CameraRig>,
    mut query: Query<&mut Transform, With<DefaultCamera>>,
) {
    let time_delta_seconds: f32 = time.delta_seconds();

    let mut move_vec = Vec3::ZERO;
    let mut boost = 0.0;

    if keys.pressed(KeyCode::LShift) {
        boost = 1.0;
    }
    if keys.pressed(KeyCode::LControl) {
        boost = -1.0;
    }

    if keys.pressed(KeyCode::W) {
        move_vec.z -= 1.0;
    }
    if keys.pressed(KeyCode::S) {
        move_vec.z += 1.0;
    }
    if keys.pressed(KeyCode::A) {
        move_vec.x -= 1.0;
    }
    if keys.pressed(KeyCode::D) {
        move_vec.x += 1.0;
    }

    if keys.pressed(KeyCode::E) {
        move_vec.y += 1.0;
    }
    if keys.pressed(KeyCode::Q) {
        move_vec.y -= 1.0;
    }

    let mouse_sensitivity = 0.5;
    let mut mouse_delta = Vec2::ZERO;
    if mouse_buttons.pressed(MouseButton::Right) {
        for event in mouse_motion_events.iter() {
            mouse_delta += event.delta;
        }
    }

    let move_vec = camera_rig.final_transform.rotation * move_vec * 10.0f32.powf(boost);

    camera_rig
        .driver_mut::<Position>()
        .translate(move_vec * time_delta_seconds * 2.5);

    camera_rig.driver_mut::<YawPitch>().rotate_yaw_pitch(
        -0.1 * mouse_delta.x * mouse_sensitivity,
        -0.1 * mouse_delta.y * mouse_sensitivity,
    );

    camera_rig.update(time_delta_seconds);

    let mut camera_transform = query.iter_mut().next().unwrap();
    camera_transform.translation = camera_rig.final_transform.position;
    camera_transform.rotation = camera_rig.final_transform.rotation;
}
