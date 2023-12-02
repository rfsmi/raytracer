use crate::{
    material::Material,
    ray::{Interval, Ray, AABB},
    vector::{P3, V3},
};

pub struct HitRecord<'a> {
    pub front_face: bool,
    pub material: &'a dyn Material,
    pub normal: V3,
    pub p: P3,
    pub t: f64,
}

impl<'a> HitRecord<'a> {
    pub fn new(r: &Ray, p: P3, t: f64, outward_normal: V3, material: &'a dyn Material) -> Self {
        let front_face = r.direction.dot(&outward_normal) < 0.0;
        Self {
            front_face,
            material,
            normal: if front_face { 1.0 } else { -1.0 } * outward_normal,
            p,
            t,
        }
    }

    pub fn ray(&self, direction: V3) -> Ray {
        Ray::new(self.p + direction * 1e-5, direction)
    }
}

pub trait Hit: Sync {
    fn aabb(&self) -> &AABB;
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval) -> Option<HitRecord<'a>>;
}

pub struct HitList {
    objects: Vec<Box<dyn Hit>>,
    aabb: AABB,
}

impl HitList {
    pub fn new(object: Box<dyn Hit>) -> Self {
        Self {
            aabb: *object.aabb(),
            objects: vec![object],
        }
    }

    pub fn add(&mut self, object: Box<dyn Hit>) {
        self.aabb = self.aabb.union(object.aabb());
        self.objects.push(object);
    }
}

impl Hit for HitList {
    fn hit(&self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord> {
        if !r.intersects_aabb(&self.aabb, &ray_t) {
            return None;
        }
        self.objects
            .iter()
            .filter_map(move |object| {
                let hr = object.hit(r, ray_t)?;
                ray_t.max = hr.t;
                Some(hr)
            })
            .last()
    }

    fn aabb(&self) -> &AABB {
        &self.aabb
    }
}
