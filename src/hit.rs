use crate::{
    aabb::AABB,
    material::Material,
    ray::{Interval, Ray},
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
        Ray::new(self.p, direction)
    }
}

pub trait Hit: Sync {
    fn aabb(&self) -> &AABB;
    fn hit<'a>(&'a self, r: &Ray, ray_t: Interval) -> Option<HitRecord<'a>>;
}

pub struct HitList {
    objects: Vec<Box<dyn Hit>>,
    aabb: AABB, // Maybe don't need this
}

impl HitList {
    pub fn new(objects: impl IntoIterator<Item = Box<dyn Hit>>) -> Self {
        let objects: Vec<_> = objects.into_iter().collect();
        Self {
            aabb: AABB::union(objects.iter().map(|o| o.aabb())),
            objects,
        }
    }
}

impl Hit for HitList {
    fn hit(&self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord> {
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
