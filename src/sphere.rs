use crate::{
    hit::{Hit, HitRecord},
    interval::Interval,
    material::Material,
    ray::Ray,
    vector::P3,
};

pub struct Sphere<'a> {
    center: P3,
    radius: f64,
    material: &'a dyn Material,
}

impl<'a> Sphere<'a> {
    pub fn new(center: P3, radius: f64, material: &'a dyn Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hit for Sphere<'_> {
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
        Some(HitRecord::new(r, p, t, outward_normal, self.material))
    }
}
