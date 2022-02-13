use crate::selector;
use bevy::{math::Quat, prelude::*};
use dolly::rig::CameraRig;
use k::{
    connect,
    nalgebra::{Isometry3, Translation3, UnitQuaternion, Vector3},
    prelude::*,
    JacobianIkSolver, JointType, NodeBuilder, SerialChain,
};
use nalgebra::Unit;
use parry3d::{
    math::{Point, Real, Vector},
    query::{Ray, RayCast, RayIntersection},
    shape::Cuboid,
};
use std::collections::HashMap;

const DEFAULT_ANGLES: &[f32] = &[0.2, 0.2, 0.0, -1.5, 0.0, -0.3, 0.0];

#[derive(Component)]
pub struct IkCubes {
    half_extents: Vec3,
}

impl RayCast for IkCubes {
    fn cast_local_ray_and_get_normal(
        &self,
        ray: &Ray,
        max_toi: Real,
        solid: bool,
    ) -> Option<RayIntersection> {
        Cuboid::new(Vector::new(
            self.half_extents.x,
            self.half_extents.y,
            self.half_extents.z,
        ))
        .cast_local_ray_and_get_normal(&ray, f32::INFINITY, true)
    }
}

#[derive(Default, Debug)]
pub struct IkCubeTargetLocation(Option<Vec3>);

#[derive(Default, Debug)]
pub struct IkHitImpact(pub Option<Vec3>);

#[derive(Default, Debug)]
pub struct SelectedIkCube(Option<usize>);

impl SelectedIkCube {
    fn set_selected(&mut self, selection: usize) {
        self.0 = Some(selection);
    }

    fn get(&self) -> Option<usize> {
        self.0
    }
}

fn build_joints() -> k::Node<f32> {
    let fixed: k::Node<f32> = NodeBuilder::new()
        .name("fixed")
        .joint_type(JointType::Fixed)
        .translation(Translation3::new(0.0, 0.0, 0.6))
        .finalize()
        .into();
    let l0: k::Node<f32> = NodeBuilder::new()
        .name("shoulder_pitch")
        .joint_type(JointType::Rotational {
            axis: Vector3::y_axis(),
        })
        .translation(Translation3::new(0.0, 0.1, 0.0))
        .finalize()
        .into();
    let l1: k::Node<f32> = NodeBuilder::new()
        .name("shoulder_roll")
        .joint_type(JointType::Rotational {
            axis: Vector3::x_axis(),
        })
        .translation(Translation3::new(0.0, 0.1, 0.0))
        .finalize()
        .into();
    let l2: k::Node<f32> = NodeBuilder::new()
        .name("shoulder_yaw")
        .joint_type(JointType::Rotational {
            axis: Vector3::z_axis(),
        })
        .translation(Translation3::new(0.0, 0.0, -0.30))
        .finalize()
        .into();
    let l3: k::Node<f32> = NodeBuilder::new()
        .name("elbow_pitch")
        .joint_type(JointType::Rotational {
            axis: Vector3::y_axis(),
        })
        .translation(Translation3::new(0.0, 0.0, -0.15))
        .finalize()
        .into();
    let l4: k::Node<f32> = NodeBuilder::new()
        .name("wrist_yaw")
        .joint_type(JointType::Rotational {
            axis: Vector3::z_axis(),
        })
        .translation(Translation3::new(0.0, 0.0, -0.15))
        .finalize()
        .into();
    let l5: k::Node<f32> = NodeBuilder::new()
        .name("wrist_pitch")
        .joint_type(JointType::Rotational {
            axis: Vector3::y_axis(),
        })
        .translation(Translation3::new(0.0, 0.0, -0.15))
        .finalize()
        .into();
    let l6: k::Node<f32> = NodeBuilder::new()
        .name("wrist_roll")
        .joint_type(JointType::Rotational {
            axis: Vector3::x_axis(),
        })
        .translation(Translation3::new(0.0, 0.0, -0.10))
        .finalize()
        .into();
    connect![fixed => l0 => l1 => l2 => l3 => l4 => l5 => l6];
    fixed
}

fn build_arm() -> SerialChain<f32> {
    let root = build_joints();
    let arm: SerialChain<f32> = k::SerialChain::new_unchecked(k::Chain::from_root(root));

    arm.set_joint_positions(&DEFAULT_ANGLES).unwrap();
    let base_rot = Isometry3::from_parts(
        Translation3::new(0.0, 0.0, -0.6),
        UnitQuaternion::from_euler_angles(0.0, -1.57, -1.57),
    );
    arm.iter().next().unwrap().set_origin(
        base_rot
            * Isometry3::from_parts(
                Translation3::new(0.0, 0.0, 0.6),
                UnitQuaternion::from_euler_angles(0.0, 0.0, 0.0),
            ),
    );
    arm.update_transforms();
    arm
}

pub(crate) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arm = build_arm();
    let arm_len = arm.iter().count();
    commands.insert_resource(arm);
    commands.insert_resource(IkHitImpact::default());
    commands.insert_resource(SelectedIkCube::default());
    commands.insert_resource(IkCubeTargetLocation::default());

    for i in 0..arm_len {
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
                material: materials.add(StandardMaterial {
                    base_color: Color::RED,
                    metallic: 1.0,
                    perceptual_roughness: 0.0,
                    reflectance: 1.0,
                    emissive: Color::RED,
                    ..Default::default()
                }),
                transform: Transform {
                    translation: Vec3::new(i as f32, 1.0, 0.0),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(IkCubes {
                half_extents: Vec3::new(0.1, 0.1, 0.1),
            });
    }
}

pub fn ik_box_undercursor(
    mut commands: Commands,
    cubes: Query<(&GlobalTransform, &IkCubes)>,
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut selected_cube: ResMut<SelectedIkCube>,
    camera_rig: Res<CameraRig>,
    mut hit_impact: ResMut<IkHitImpact>,
    camera: Query<&Camera>,
) {
    if !keys.pressed(KeyCode::LAlt) && mouse_buttons.pressed(MouseButton::Left) {
        if let Some((cube_index, hit)) =
            selector::component_under_cursory_ray(cubes, windows, camera_rig, camera)
        {
            selected_cube.set_selected(cube_index);
            hit_impact.0 = Some(hit);
        }
    }
}

pub fn solve(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    arm: ResMut<SerialChain<f32>>,
    mut cubes: Query<&mut Transform, With<IkCubes>>,
) {
    let end = arm.find("wrist_roll").unwrap();
    let mut target = end.world_transform().unwrap();

    let time_delta_seconds: f32 = time.delta_seconds();
    let mut move_vec = Vec3::ZERO;
    let multiplier = 1.0;
    let mut reset = false;

    if keys.pressed(KeyCode::R) {
        reset = true;
        arm.set_joint_positions(&DEFAULT_ANGLES).unwrap();
        arm.update_transforms();
    }

    if !keys.pressed(KeyCode::LShift) && keys.pressed(KeyCode::X) {
        move_vec.x += 1.0;
    }

    if keys.pressed(KeyCode::LShift) && keys.pressed(KeyCode::X) {
        move_vec.x -= 1.0;
    }

    if !keys.pressed(KeyCode::LShift) && keys.pressed(KeyCode::Y) {
        move_vec.y += 1.0;
    }

    if keys.pressed(KeyCode::LShift) && keys.pressed(KeyCode::Y) {
        move_vec.y -= 1.0;
    }

    if !keys.pressed(KeyCode::LShift) && keys.pressed(KeyCode::Z) {
        move_vec.z += 1.0;
    }

    if keys.pressed(KeyCode::LShift) && keys.pressed(KeyCode::Z) {
        move_vec.z -= 1.0;
    }

    target.translation.vector.x += move_vec.x * time_delta_seconds * multiplier;
    target.translation.vector.y += move_vec.y * time_delta_seconds * multiplier;
    target.translation.vector.z += move_vec.z * time_delta_seconds * multiplier;

    let solver: JacobianIkSolver<f32> = JacobianIkSolver::default();
    let constraints = k::Constraints::default();
    solver
        .solve_with_constraints(&arm, &target, &constraints)
        .ok();

    for (i, transform) in arm.update_transforms().iter().enumerate() {
        let translation = Vec3::new(
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
        );
        let rotation = Quat::from_xyzw(
            transform.rotation.quaternion().coords.x,
            transform.rotation.quaternion().coords.y,
            transform.rotation.quaternion().coords.z,
            transform.rotation.quaternion().coords.w,
        );
        cubes.iter_mut().nth(i).map(|mut cube| {
            cube.translation = Vec3::new(i as f32 * 0.1, 1.0, 0.0) + translation;
            cube.rotation = rotation;
        });
    }
}

pub fn command_move_selected_ik_object(
    mut commands: Commands,
    mouse_buttons: Res<Input<MouseButton>>,
    selected_cube: Res<SelectedIkCube>,
    mut cube_target_location: ResMut<IkCubeTargetLocation>,
    keys: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    camera_rig: Res<CameraRig>,
    camera: Query<&Camera>,
    hit_impact: Res<IkHitImpact>,
) {
    if keys.pressed(KeyCode::LAlt) && mouse_buttons.pressed(MouseButton::Left) {
        let camera_location = camera_rig.final_transform.position;
        let ray = selector::cursor_ray(windows, camera, camera_rig);
        if let Some(cube_index) = selected_cube.get() {
            let hit_impact = hit_impact.0.unwrap();
            if let Some(hit) = selector::intersect_half_space(ray, hit_impact, camera_location) {
                cube_target_location.0 = Some(hit);
            }
        }
    }
}

pub fn update_move_selected_ik_object(
    time: Res<Time>,
    mut cube_target_location: ResMut<IkCubeTargetLocation>,
    mut cubes: Query<&mut Transform, With<IkCubes>>,
    arm: ResMut<SerialChain<f32>>,
    selected_cube: Res<SelectedIkCube>,
) {
    if let Some(cube_index) = selected_cube.0 {
        let end = arm.iter().nth(cube_index).unwrap();
        let mut target = end.world_transform().unwrap();

        let cube_transform = cubes.iter_mut().nth(cube_index).unwrap();

        if let Some(target_location) = &cube_target_location.0 {
            let arm_translation = Vec3::new(
                target.translation.x,
                target.translation.y,
                target.translation.z,
            );
            let cube_translation = cube_transform.translation;

            let direction = (*target_location - cube_translation).normalize();
            let multiplier = time.delta_seconds() * 3.0;
            let distance = cube_translation.distance(*target_location);

            let move_vec = direction * multiplier;
            target.translation.x += move_vec.x;
            target.translation.y += move_vec.y;
            target.translation.z += move_vec.z;

            if distance <= move_vec.length() {
                target.translation.x = target_location.x;
                target.translation.y = target_location.y;
                target.translation.z = target_location.z;
            }
        }

        let solver: JacobianIkSolver<f32> = JacobianIkSolver::default();
        let constraints = k::Constraints {
            ..Default::default()
        };
        solver
            .solve_with_constraints(&arm, &target, &constraints)
            .unwrap_or_else(|e| {
                cube_target_location.0 = None;
            });

        for (i, transform) in arm.update_transforms().iter().enumerate() {
            let translation = Vec3::new(
                transform.translation.x,
                transform.translation.y,
                transform.translation.z,
            );
            let rotation = Quat::from_xyzw(
                transform.rotation.quaternion().coords.x,
                transform.rotation.quaternion().coords.y,
                transform.rotation.quaternion().coords.z,
                transform.rotation.quaternion().coords.w,
            );

            cubes.iter_mut().nth(i).map(|mut cube| {
                cube.translation = Vec3::new(i as f32 * 0.1, 1.0, 0.0) + translation;
                cube.rotation = rotation;
            });
        }
    }
}
