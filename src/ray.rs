use crate::vector::{P3, V3};

pub struct Ray {
    pub origin: P3,
    pub direction: V3,
}

impl Ray {
    pub fn new(origin: P3, dir: V3) -> Self {
        Ray {
            origin,
            direction: dir,
        }
    }

    pub fn at(&self, t: f64) -> P3 {
        self.origin + t * self.direction
    }
}
