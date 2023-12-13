use crate::vector::{P3, V3};

pub struct Ray {
    pub origin: P3,
    pub direction: V3,
    pub inv_direction: V3,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Ray {
    pub fn new(origin: P3, direction: V3) -> Self {
        let direction = direction.unit();
        Ray {
            origin,
            direction,
            inv_direction: 1.0 / direction,
        }
    }

    pub fn at(&self, t: f64) -> P3 {
        self.origin + t * self.direction
    }
}

impl Interval {
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    pub fn clamp(&self, x: f64) -> f64 {
        x.max(self.min).min(self.max)
    }
}
