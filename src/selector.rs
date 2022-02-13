use bevy::{ecs::query::WorldQuery, prelude::*};
use parry3d::{
    math::{Isometry, Point, Real, Vector},
    query::{Ray, RayCast, RayIntersection},
    shape::{Cuboid, HalfSpace},
};

use dolly::prelude::CameraRig;
use std::{cmp::Ordering, collections::HashMap};

/// calculate the ray starting location with respect to mouse position relative to the window
pub(crate) fn cursor_ray(window: &Window) -> Vec3 {
    let cursor = window.cursor_position().unwrap_or(Vec2::new(0.0, 0.0));

    let w = window.width();
    let h = window.height();

    let aspect_ratio = w / h;

    Vec3::new(
        (cursor.x / w - 0.5) * aspect_ratio,
        cursor.y / h - 0.5,
        -1.0,
    )
    .normalize()
}

/// an algorithmn to test which of the components is under the cursor if a ray is to be casted
/// from the cursor location to the scene
pub(crate) fn component_under_cursory_ray<T>(
    components: Query<(&GlobalTransform, &T)>,
    windows: Res<Windows>,
    camera: Res<CameraRig>,
) -> Option<(usize, Vec3)>
where
    T: Component + RayCast,
{
    let window = windows.get_primary().unwrap();
    let camera_transform = camera.final_transform;
    let mouse_ray = cursor_ray(window);
    let direction = camera_transform.rotation * mouse_ray;
    let ray = Ray::new(
        Point::new(
            camera_transform.position.x,
            camera_transform.position.y,
            camera_transform.position.z,
        ),
        Vector::new(direction.x, direction.y, direction.z),
    );

    let closest: Option<(usize, f32)> = components
        .iter()
        .enumerate()
        .filter_map(|(index, (transform, component))| {
            let isometry = Isometry::new(
                Vector::new(
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ),
                Vector::identity(),
            );

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
        Some((i, Vec3::new(hit.x, hit.y, hit.z)))
    } else {
        None
    }
}
