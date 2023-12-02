use crate::vector::{P3, V3};

pub struct Ray {
    pub origin: P3,
    pub direction: V3,
    pub inv_direction: V3,
}

#[derive(Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AABB {
    pub min: P3,
    pub max: P3,
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

    pub fn intersects_aabb(&self, aabb: &AABB, interval: &Interval) -> bool {
        let t1 = (aabb.min - self.origin) * self.inv_direction;
        let t2 = (aabb.max - self.origin) * self.inv_direction;
        let tmin = (t1.x.min(t2.x)).max(t1.y.min(t2.y)).max(t1.z.min(t2.z));
        let tmax = (t1.x.max(t2.x)).min(t1.y.max(t2.y)).min(t1.z.max(t2.z));
        tmin <= interval.max && interval.min <= tmax
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

impl AABB {
    pub fn new(min: P3, size: V3) -> Self {
        Self {
            min,
            max: min + size,
        }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: P3::new()
                .x(self.min.x.min(other.min.x))
                .y(self.min.y.min(other.min.y))
                .z(self.min.z.min(other.min.z)),
            max: P3::new()
                .x(self.max.x.max(other.max.x))
                .y(self.max.y.max(other.max.y))
                .z(self.max.z.max(other.max.z)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aabb_union() {
        let a = AABB::new(P3::new().z(2.0), V3::new().x(2.0));
        let b = AABB::new(P3::new(), V3::new().x(1.0));
        assert_eq!(a.union(&b), AABB::new(P3::new(), V3::new().x(2.0).z(2.0)))
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let aabb = AABB::new(
            P3::new().x(100.0).y(100.0).z(100.0),
            V3::new().x(5.0).y(5.0).z(5.0),
        );
        let ray = Ray::new(P3::new().x(101.0).y(101.0), V3::new().z(1.0));
        assert!(ray.intersects_aabb(&aabb, &Interval::new(0.1, 100.5)));
        assert!(ray.intersects_aabb(&aabb, &Interval::new(100.4, 100.5)));
        assert!(!ray.intersects_aabb(&aabb, &Interval::new(0.1, 99.5)));
        assert!(!ray.intersects_aabb(&aabb, &Interval::new(105.1, 106.0)));
    }

    #[test]
    fn test_ray_aabb_intersection_angle() {
        // Diagonal through the AABB is length 1
        let s = (1.0f64 / 3.0).sqrt();
        let aabb = AABB::new(P3::new(), V3::new().x(s).y(s).z(s));
        let ray = Ray::new(P3::new().x(-s).y(-s).z(-s), V3::new().x(1.0).y(1.0).z(1.0));
        assert!(!ray.intersects_aabb(&aabb, &Interval::new(0.1, 0.9)));
        assert!(ray.intersects_aabb(&aabb, &Interval::new(0.1, 1.1)));
        assert!(ray.intersects_aabb(&aabb, &Interval::new(0.1, 2.1)));
        assert!(ray.intersects_aabb(&aabb, &Interval::new(1.1, 2.1)));
        assert!(!ray.intersects_aabb(&aabb, &Interval::new(2.1, 3.1)));
    }
}
