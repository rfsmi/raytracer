use std::sync::Arc;

use crate::{
    aabb::AABB,
    bvh::{Axis, Plane},
    hit::{Hit, HitRecord},
    material::Material,
    ray::{Interval, Ray},
    vector::P3,
};

pub struct Sphere {
    center: P3,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: P3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = oc.dot(&r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        // Check both intersection points.
        let t = [-1.0, 1.0]
            .iter()
            .map(|f| (-half_b + f * discriminant.sqrt()) / a)
            .find(|t| ray_t.surrounds(*t))?;
        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        Some(HitRecord::new(r, p, t, outward_normal, &*self.material))
    }

    fn aabb(&self) -> AABB {
        AABB::bounding_box([self.center - self.radius, self.center + self.radius])
    }

    fn split_aabb(&self, plane: Plane) -> (AABB, AABB) {
        let aabb = self.aabb();

        let mut lhs = aabb;
        let mut rhs = aabb;
        *lhs.max.axis_mut(plane.axis) = plane.pos;
        *rhs.min.axis_mut(plane.axis) = plane.pos;
        lhs = AABB::intersection([aabb, lhs]);
        rhs = AABB::intersection([aabb, rhs]);

        let x = plane.pos - self.center.axis(plane.axis);
        if x.abs() > self.radius {
            let sign = x.signum();
            let y = (self.radius * self.radius - x * x).sqrt();
            let mut split_plane = AABB::new();
            for &axis in Axis::ALL {
                let (max, min) = if axis == plane.axis {
                    (plane.pos, plane.pos)
                } else {
                    (y, -y)
                };
                *split_plane.min.axis_mut(axis) = min;
                *split_plane.max.axis_mut(axis) = max;
            }
        }

        if x.abs() >= self.radius {
            // Sphere lies entirely on one side of plane.
            let big = if x < 0.0 { &mut rhs.min } else { &mut lhs.max };
        } else {
            // Plane intersects sphere
            // let mut lhs = aabb;
            // plane.axis.get_v3_mut(&mut lhs.min) =
            // let big = AABB::bounding_box([points])
        }
        (lhs, rhs)
    }
}
