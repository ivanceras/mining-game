use crate::selector;
use crate::DefaultCamera;
use bevy::prelude::*;
use dolly::prelude::CameraRig;
use parry3d::{
    math::{Point, Vector},
    query::Ray,
};
/// flag whether to make projectile start from the center of the screen
///
/// false, the projectile come from the mouse position
const PROJECTILE_FROM_CENTER: bool = false;

/// flag whether to make the projectile seek the target, in this case the car
const PROJECTILE_SEEK_TARGET: bool = true;
/// speed of the projectile
const PROJECTILE_SEEK_SPEED: f32 = 100.0;
/// speed of the projectile at launch
const PROJECTILE_LAUNCH_SPEED: f32 = 20.0;

#[derive(Component)]
pub struct Projectile {
    /// the direction this projectile was fired
    direction: Vec3,
    /// time this projectile was fired
    fired: f64,
}

/// spawn a projectile at the mouse pointing direction
pub(crate) fn spawn_projectile(
    time: Res<Time>,
    mut commands: Commands,
    mouse_buttons: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    query: Query<&Transform, With<DefaultCamera>>,
    windows: Res<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_rig: Res<CameraRig>,
    camera: Query<&Camera>,
) {
    if keys.pressed(KeyCode::LShift) && mouse_buttons.pressed(MouseButton::Left) {
        let mouse_ray = selector::from_screenspace(windows, camera, camera_rig).unwrap();
        for camera_transform in query.iter() {
            let direction = if PROJECTILE_FROM_CENTER {
                camera_transform.forward()
            } else {
                let direction: Vec3 = mouse_ray.dir.into();
                camera_transform.rotation * direction
            };

            let offset = 10.0; //offset meters away in front of the camera
            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.1,
                        subdivisions: 8,
                    })),
                    material: materials.add(StandardMaterial {
                        base_color: Color::GREEN,
                        ..Default::default()
                    }),
                    transform: Transform {
                        translation: camera_transform.translation + direction.normalize() * offset,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Projectile {
                    direction,
                    fired: time.seconds_since_startup(),
                });
        }
    }
}

pub(crate) fn move_projectile(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&mut Transform, Entity, &mut Projectile)>,
) {
    let time_delta_seconds: f32 = time.delta_seconds();
    let seconds_since_startup = time.seconds_since_startup();
    for (mut transform, entity, projectile) in query.iter_mut() {
        let projectile_time = seconds_since_startup - projectile.fired;
        transform.translation +=
            projectile.direction * time_delta_seconds * PROJECTILE_LAUNCH_SPEED;
    }
}
