use std::{cell::RefCell, rc::Rc};

use crate::hit::{Hit, HitRecord};

pub struct HitList {
    objects: Vec<Rc<RefCell<dyn Hit>>>,
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

    pub fn add(&mut self, object: Rc<RefCell<dyn Hit>>) {
        self.objects.push(object);
    }
}

impl Hit for HitList {
    fn hit(&self, r: &crate::ray::Ray, tmin: f64, mut tmax: f64) -> Option<HitRecord> {
        self.objects
            .iter()
            .filter_map(|object| {
                let hr = object.borrow().hit(r, tmin, tmax)?;
                tmax = hr.t;
                Some(hr)
            })
            .last()
    }
}
