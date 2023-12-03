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

#[cfg(test)]
mod test {
    use crate::aabb::AABB;

    use super::*;

    #[test]
    fn test_aabb_union() {
        let a = AABB::bounding_box([P3::new().z(2.0), P3::new().x(2.0)]);
        let b = AABB::bounding_box([P3::new(), P3::new().x(1.0)]);
        assert_eq!(
            AABB::union([&a, &b]),
            AABB::bounding_box([P3::new(), P3::new().x(2.0).z(2.0)])
        )
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let aabb = AABB::bounding_box([
            P3::new().x(100.0).y(100.0).z(100.0),
            P3::new().x(105.0).y(105.0).z(105.0),
        ]);
        let ray = Ray::new(P3::new().x(101.0).y(101.0), V3::new().z(1.0));
        assert!(aabb
            .ray_intersection(&ray, Interval::new(0.1, 100.5))
            .is_some());
        assert!(aabb
            .ray_intersection(&ray, Interval::new(100.4, 100.5))
            .is_some());
        assert!(aabb
            .ray_intersection(&ray, Interval::new(0.1, 99.5))
            .is_none());
        assert!(aabb
            .ray_intersection(&ray, Interval::new(105.1, 106.0))
            .is_none());
    }

    #[test]
    fn test_ray_aabb_intersection_angle() {
        // Diagonal through the AABB is length 1
        let s = (1.0f64 / 3.0).sqrt();
        let aabb = AABB::bounding_box([P3::new(), P3::new().x(s).y(s).z(s)]);
        let ray = Ray::new(P3::new().x(-s).y(-s).z(-s), V3::new().x(1.0).y(1.0).z(1.0));
        assert!(aabb
            .ray_intersection(&ray, Interval::new(0.1, 0.9))
            .is_none());
        assert!(aabb
            .ray_intersection(&ray, Interval::new(0.1, 1.1))
            .is_some());
        assert!(aabb
            .ray_intersection(&ray, Interval::new(0.1, 2.1))
            .is_some());
        assert!(aabb
            .ray_intersection(&ray, Interval::new(1.1, 2.1))
            .is_some());
        assert!(aabb
            .ray_intersection(&ray, Interval::new(2.1, 3.1))
            .is_none());
    }

    #[test]
    fn test_ray_aabb_intersection_failure() {
        let ray = Ray::new(
            P3::new().x(13.0).y(2.0).z(3.0),
            V3::new()
                .x(-0.99999581369879142)
                .y(-0.0022009767150356608)
                .z(-0.001878373336654929),
        );
        let aabb = AABB::bounding_box([P3::new().x(-5.0).z(-1.0), P3::new().x(5.0).y(2.0).z(1.0)]);
        let interval = Interval::new(1e-3, f64::INFINITY);
        let intersection = aabb.ray_intersection(&ray, interval);
        assert_eq!(intersection, None);
    }
}
