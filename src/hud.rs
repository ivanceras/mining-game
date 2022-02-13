use crate::selector;
use bevy::{math::Vec3, prelude::*};
use dolly::rig::CameraRig;
use parry3d::{
    math::{Real, Vector},
    query::{Ray, RayCast, RayIntersection},
    shape::Cuboid,
};

#[derive(Component, Copy, Clone)]
struct Hud;

#[derive(Component, Copy, Clone)]
pub struct UiButton {
    half_extents: Vec3, //describe by halfextents to each access
}

impl RayCast for UiButton {
    fn cast_local_ray_and_get_normal(
        &self,
        ray: &Ray,
        max_toi: Real,
        solid: bool,
    ) -> Option<RayIntersection> {
        Cuboid::new(self.half_extents.into()).cast_local_ray_and_get_normal(
            &ray,
            f32::INFINITY,
            true,
        )
    }
}

pub(crate) fn setup(
    parent: &mut ChildBuilder,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    parent
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(0.1, 0.1, 0.0001))),
            material: materials.add(StandardMaterial {
                base_color: Color::GREEN,
                ..Default::default()
            }),
            transform: Transform {
                translation: Vec3::new(-0.09, -0.03, -0.2),
                //rotation: Quat::from_rotation_y(20.0_f32.to_radians()),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Hud)
        .with_children(|ui| {
            for j in 1..5 {
                for i in 1..5 {
                    let location =
                        Vec3::new(i as f32 * 0.02 - 0.05, j as f32 * 0.02 - 0.05, 0.0001);
                    let half_extents = Vec3::new(0.005, 0.01, 0.0);

                    ui.spawn_bundle(PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(0.01, 0.01, 0.0001))),
                        transform: Transform {
                            translation: location,
                            ..Default::default()
                        },
                        material: materials.add(StandardMaterial {
                            base_color: Color::RED,
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .insert(UiButton { half_extents });
                }
            }
        });
}

pub(crate) fn button_undercursor(
    cubes: Query<(&GlobalTransform, &UiButton)>,
    keys: Res<Input<KeyCode>>,
    mouse_buttons: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    camera_rig: Res<CameraRig>,
    camera: Query<&Camera>,
) {
    if mouse_buttons.pressed(MouseButton::Left) {
        if let Some((cube_index, hit)) =
            selector::component_under_cursory_ray(cubes, windows, camera_rig, camera)
        {
            println!("selected ui: {} at: {}", cube_index, hit);
        } else {
            println!("No hit..");
        }
    }
}
