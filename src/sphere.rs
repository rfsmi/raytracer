use std::sync::Arc;

use crate::{
    aabb::AABB,
    hit::{Hit, HitRecord},
    material::Material,
    ray::{Interval, Ray},
    vector::P3,
};

pub struct Sphere {
    center: P3,
    radius: f64,
    material: Arc<dyn Material>,
    aabb: AABB,
}

impl Sphere {
    pub fn new(center: P3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
            aabb: AABB::bounding_box([center - radius, center + radius]),
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

    fn aabb(&self) -> &AABB {
        &self.aabb
    }
}
