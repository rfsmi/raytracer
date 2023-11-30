use crate::{interval::Interval, ray::Ray, vec3::Vec3};

pub struct HitRecord {
    pub p: Vec3,
    pub t: f64,
    pub normal: Vec3,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(r: &Ray, p: Vec3, t: f64, outward_normal: Vec3) -> Self {
        let front_face = r.direction.dot(&outward_normal) < 0.0;
        Self {
            p,
            t,
            normal: if front_face { 1.0 } else { -1.0 } * outward_normal,
            front_face,
        }
    }
}

pub trait Hit {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord>;
}
