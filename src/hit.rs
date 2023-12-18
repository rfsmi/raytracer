use glam::DVec3;

use crate::{
    aabb::AABB,
    material::Material,
    ray::{Interval, Ray},
};

pub struct HitRecord<'a> {
    pub front_face: bool,
    pub material: &'a dyn Material,
    pub normal: DVec3,
    pub p: DVec3,
    pub t: f64,
}

impl<'a> HitRecord<'a> {
    pub fn new(r: &Ray, p: DVec3, t: f64, out_normal: DVec3, material: &'a dyn Material) -> Self {
        let front_face = r.direction.dot(out_normal) < 0.0;
        Self {
            front_face,
            material,
            normal: if front_face { 1.0 } else { -1.0 } * out_normal,
            p,
            t,
        }
    }

    pub fn ray(&self, direction: DVec3) -> Ray {
        Ray::new(self.p, direction)
    }
}

pub trait Hit: Sync {
    fn aabb(&self) -> AABB;
    fn clipped_aabb(&self, axis: DVec3, t1: f64, t2: f64) -> AABB;
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval) -> Option<HitRecord<'a>>;
}
