use glam::DVec3;

use crate::ray::{Interval, Ray};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AABB {
    pub min: DVec3,
    pub max: DVec3,
}

impl AABB {
    pub fn new() -> Self {
        Self {
            min: DVec3::splat(f64::INFINITY),
            max: DVec3::splat(f64::NEG_INFINITY),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size().min_element() < 0.0
    }

    pub fn centroid(&self) -> DVec3 {
        self.min + 0.5 * (self.max - self.min)
    }

    pub fn size(&self) -> DVec3 {
        self.max - self.min
    }

    pub fn surface_area(&self) -> f64 {
        let DVec3 { x, y, z } = self.size();
        2.0 * (x * y + x * z + z * y)
    }

    pub fn update(&mut self, other: Self) {
        *self = Self::union([*self, other]);
    }

    pub fn bounding_box(points: impl IntoIterator<Item = DVec3>) -> Self {
        let mut points = points.into_iter();
        let Some(p) = points.next() else {
            return Self::new();
        };
        let (min, max) = points.fold((p, p), |(min, max), p| (min.min(p), max.max(p)));
        Self { min, max }
    }

    pub fn union(aabbs: impl IntoIterator<Item = Self>) -> Self {
        aabbs.into_iter().fold(Self::new(), |a, b| Self {
            min: DVec3::min(a.min, b.min),
            max: DVec3::max(a.max, b.max),
        })
    }

    pub fn intersection(aabbs: impl IntoIterator<Item = Self>) -> Self {
        aabbs
            .into_iter()
            .reduce(|a, b| Self {
                min: DVec3::max(a.min, b.min),
                max: DVec3::min(a.max, b.max),
            })
            .unwrap_or(Self {
                max: DVec3::INFINITY,
                min: DVec3::NEG_INFINITY,
            })
    }

    pub fn ray_intersection(&self, r: &Ray, ray_t: Interval) -> Option<(DVec3, f64)> {
        if self.is_empty() {
            return None;
        }
        let t1 = (self.min - r.origin) * r.inv_direction;
        let t2 = (self.max - r.origin) * r.inv_direction;
        let (t1, t2) = (t1.min(t2), t2.max(t1));
        let tmin = t1.x.max(t1.y).max(t1.z);
        let tmax = t2.x.min(t2.y).min(t2.z);
        if tmax < tmin || tmax < ray_t.min || ray_t.max < tmin {
            return None;
        }
        Some(if tmin > 0.0 { (t1, tmin) } else { (t2, tmax) })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_aabb_intersection_empty() {
        let test = |a, b| assert!(AABB::intersection([a, b]).is_empty());
        test(AABB::new(), AABB::new());
        test(AABB::bounding_box([DVec3::ZERO]), AABB::new());
        test(
            AABB::bounding_box([DVec3::ZERO]),
            AABB::bounding_box([DVec3::splat(1.0)]),
        );
        test(
            AABB {
                min: DVec3::ZERO,
                max: DVec3::new(0.0, 0.0, -1.0),
            },
            AABB::bounding_box([DVec3::splat(1.0)]),
        );
    }

    #[test]
    fn test_aabb_intersection() {
        let test = |a, b, eq| assert_eq!(AABB::intersection([a, b]), eq);
        test(
            AABB {
                min: DVec3::new(0.0, 0.0, -2.0),
                max: DVec3::new(5.0, 3.0, 3.0),
            },
            AABB {
                min: DVec3::splat(-1.0),
                max: DVec3::splat(4.0),
            },
            AABB {
                min: DVec3::new(0.0, 0.0, -1.0),
                max: DVec3::new(4.0, 3.0, 3.0),
            },
        )
    }

    #[test]
    fn test_aabb_union() {
        let a = AABB::bounding_box([DVec3::new(0.0, 0.0, 2.0), DVec3::new(2.0, 0.0, 0.0)]);
        let b = AABB::bounding_box([DVec3::ZERO, DVec3::new(1.0, 0.0, 0.0)]);
        assert_eq!(
            AABB::union([a, b]),
            AABB::bounding_box([DVec3::ZERO, DVec3::new(2.0, 0.0, 2.0)])
        )
    }

    #[test]
    fn test_ray_aabb_intersection() {
        let aabb = AABB::bounding_box([
            DVec3::new(100.0, 100.0, 100.0),
            DVec3::new(105.0, 105.0, 105.0),
        ]);
        let ray = Ray::new(DVec3::new(101.0, 101.0, 0.0), DVec3::new(0.0, 0.0, 1.0));
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
        let aabb = AABB::bounding_box([DVec3::ZERO, DVec3::splat(s)]);
        let ray = Ray::new(DVec3::splat(-s), DVec3::splat(1.0));
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
            DVec3::new(13.0, 2.0, 3.0),
            DVec3::new(
                -0.99999581369879142,
                -0.0022009767150356608,
                -0.001878373336654929,
            ),
        );
        let aabb = AABB::bounding_box([DVec3::new(-5.0, 0.0, -1.0), DVec3::new(5.0, 2.0, 1.0)]);
        let interval = Interval::new(1e-3, f64::INFINITY);
        let intersection = aabb.ray_intersection(&ray, interval);
        assert_eq!(intersection, None);
    }
}
