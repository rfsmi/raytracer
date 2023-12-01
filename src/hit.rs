use crate::{interval::Interval, material::Material, ray::Ray, vec3::Vec3};

pub struct HitRecord<'a> {
    pub front_face: bool,
    pub material: &'a dyn Material,
    pub normal: Vec3,
    pub p: Vec3,
    pub t: f64,
}

impl<'a> HitRecord<'a> {
    pub fn new(r: &Ray, p: Vec3, t: f64, outward_normal: Vec3, material: &'a dyn Material) -> Self {
        let front_face = r.direction.dot(&outward_normal) < 0.0;
        Self {
            front_face,
            material,
            normal: if front_face { 1.0 } else { -1.0 } * outward_normal,
            p,
            t,
        }
    }
}

pub trait Hit {
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval) -> Option<HitRecord<'a>>;
}
