use crate::{
    hit::{Hit, HitRecord},
    interval::Interval,
    ray::Ray,
};

pub struct HitList {
    objects: Vec<Box<dyn Hit>>,
}

impl HitList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }

    pub fn add(&mut self, object: Box<dyn Hit>) {
        self.objects.push(object);
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
}
