use std::rc::Rc;

use crate::{
    hit::{Hit, HitRecord},
    interval::Interval,
    ray::Ray,
};

pub struct HitList<'a> {
    objects: Vec<Box<dyn 'a + Hit>>,
}

impl<'a> HitList<'a> {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: Box<dyn 'a + Hit>) {
        self.objects.push(object);
    }
}

impl<'a> Hit for HitList<'a> {
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
}
