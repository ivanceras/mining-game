use bevy::{ecs::query::WorldQuery, prelude::*};
use dolly::prelude::CameraRig;
use nalgebra::Unit;
use parry3d::{
    math::{Isometry, Point, Real, Vector},
    query::{Ray, RayCast, RayIntersection},
    shape::{Cuboid, HalfSpace},
};
use std::{cmp::Ordering, collections::HashMap};

pub fn cursor_ray(
    windows: Res<Windows>,
    camera: Query<&Camera>,
    camera_rig: Res<CameraRig>,
) -> Ray {
    let camera = camera.iter().next().expect("must have camera");
    let window = match windows.get(camera.window) {
        Some(window) => window,
        None => {
            panic!("WindowId {} does not exist", camera.window);
        }
    };
    let cursor_pos_screen = window.cursor_position().unwrap_or(Vec2::ZERO);
    cursor_position_to_ray(cursor_pos_screen, window, camera, camera_rig)
}

fn cursor_position_to_ray(
    cursor_pos_screen: Vec2,
    window: &Window,
    camera: &Camera,
    camera_rig: Res<CameraRig>,
) -> Ray {
    let camera_transform = Transform {
        translation: camera_rig.final_transform.position,
        rotation: camera_rig.final_transform.rotation,
        scale: Vec3::ONE,
    };
    let view = camera_transform.compute_matrix();

    let screen_size = Vec2::from([window.width() as f32, window.height() as f32]);
    let projection = camera.projection_matrix;

    // 2D Normalized device coordinate cursor position from (-1, -1) to (1, 1)
    let cursor_ndc: Vec2 = (cursor_pos_screen / screen_size) * 2.0 - Vec2::from([1.0, 1.0]);
    let ndc_to_world: Mat4 = view * projection.inverse();
    let world_to_ndc = projection * view;
    let is_orthographic = projection.w_axis[3] == 1.0;

    // Compute the cursor position at the near plane. The bevy camera looks at -Z.
    let ndc_near: f32 = world_to_ndc.transform_point3(-Vec3::Z * camera.near).z;
    let cursor_pos_near: Vec3 = ndc_to_world.transform_point3(cursor_ndc.extend(ndc_near));
    println!("cursor_pos_near: {}", cursor_pos_near);

    // Compute the ray's direction depending on the projection used.
    let ray_direction = match is_orthographic {
        true => view.transform_vector3(-Vec3::Z), // All screenspace rays are parallel in ortho
        false => cursor_pos_near - camera_transform.translation, // Direction from camera to cursor
    };

    Ray::new(camera_transform.translation.into(), ray_direction.into())
}

/// an algorithmn to test which of the components is under the cursor if a ray is to be casted
/// from the cursor location to the scene
pub(crate) fn component_under_cursory_ray<T>(
    components: Query<(&GlobalTransform, &T)>,
    windows: Res<Windows>,
    camera_rig: Res<CameraRig>,
    camera: Query<&Camera>,
) -> Option<(usize, Vec3)>
where
    T: Component + RayCast,
{
    let ray = cursor_ray(windows, camera, camera_rig);

    let closest: Option<(usize, f32)> = components
        .iter()
        .enumerate()
        .filter_map(|(index, (transform, component))| {
            let isometry = Isometry::new(transform.translation.into(), Vector::identity());

            component
                .cast_ray_and_get_normal(&isometry, &ray, f32::INFINITY, true)
                .map(|intersection| {
                    let hit = ray.point_at(intersection.toi);
                    (index, intersection.toi)
                })
        })
        .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));

    if let Some((i, closest)) = closest {
        let hit = ray.point_at(closest);
        Some((i, hit.into()))
    } else {
        None
    }
}

pub(crate) fn intersect_half_space(
    ray: Ray,
    hit_impact: Vec3,
    camera_location: Vec3,
) -> Option<Vec3> {
    let half_space = HalfSpace::new(Vector::z_axis());

    let space_transform = Isometry::face_towards(
        &hit_impact.into(),
        &camera_location.into(),
        &Vector::y_axis(),
    );
    half_space
        .cast_ray_and_get_normal(&space_transform, &ray, f32::INFINITY, true)
        .map(|intersection| {
            let hit = ray.point_at(intersection.toi);
            hit.into()
        })
}
