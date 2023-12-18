use glam::DVec3;

pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
    pub inv_direction: DVec3,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Ray {
    pub fn new(origin: DVec3, direction: DVec3) -> Self {
        let direction = direction.normalize();
        Ray {
            origin,
            direction,
            inv_direction: 1.0 / direction,
        }
    }

    pub fn at(&self, t: f64) -> DVec3 {
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
