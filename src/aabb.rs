use crate::{
    ray::{Interval, Ray},
    vector::{Axis, P3, V3},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AABB {
    pub min: P3,
    pub max: P3,
}

impl AABB {
    pub fn new() -> Self {
        Self {
            min: P3::all(f64::INFINITY),
            max: P3::all(f64::NEG_INFINITY),
        }
    }

    pub fn is_empty(&self) -> bool {
        Axis::all().any(|axis| self.min.axis(axis) > self.max.axis(axis))
    }

    pub fn centroid(&self) -> P3 {
        self.min + 0.5 * (self.max - self.min)
    }

    pub fn size(&self) -> V3 {
        self.max - self.min
    }

    pub fn surface_area(&self) -> f64 {
        let V3 { x, y, z } = self.size();
        2.0 * (x * y + x * z + z * y)
    }

    pub fn update(&mut self, other: Self) {
        *self = Self::union([*self, other]);
    }

    pub fn bounding_box(points: impl IntoIterator<Item = P3>) -> Self {
        let mut points = points.into_iter();
        let Some(p) = points.next() else {
            return Self::new();
        };
        let (min, max) = points.fold((p, p), |(min, max), p| {
            (
                P3::new()
                    .x(min.x.min(p.x))
                    .y(min.y.min(p.y))
                    .z(min.z.min(p.z)),
                P3::new()
                    .x(max.x.max(p.x))
                    .y(max.y.max(p.y))
                    .z(max.z.max(p.z)),
            )
        });
        Self { min, max }
    }

    pub fn union(aabbs: impl IntoIterator<Item = Self>) -> Self {
        aabbs.into_iter().fold(Self::new(), |a, b| Self {
            min: P3::new()
                .x(a.min.x.min(b.min.x))
                .y(a.min.y.min(b.min.y))
                .z(a.min.z.min(b.min.z)),
            max: P3::new()
                .x(a.max.x.max(b.max.x))
                .y(a.max.y.max(b.max.y))
                .z(a.max.z.max(b.max.z)),
        })
    }

    pub fn intersection(aabbs: impl IntoIterator<Item = Self>) -> Self {
        aabbs
            .into_iter()
            .reduce(|a, b| Self {
                min: P3::new()
                    .x(a.min.x.max(b.min.x))
                    .y(a.min.y.max(b.min.y))
                    .z(a.min.z.max(b.min.z)),
                max: P3::new()
                    .x(a.max.x.min(b.max.x))
                    .y(a.max.y.min(b.max.y))
                    .z(a.max.z.min(b.max.z)),
            })
            .unwrap_or(Self {
                max: P3::all(f64::INFINITY),
                min: P3::all(f64::NEG_INFINITY),
            })
    }

    pub fn ray_intersection(&self, r: &Ray, ray_t: Interval) -> Option<(V3, f64)> {
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
        test(AABB::bounding_box([P3::new()]), AABB::new());
        test(
            AABB::bounding_box([P3::new()]),
            AABB::bounding_box([P3::all(1.0)]),
        );
        test(
            AABB {
                min: P3::new(),
                max: P3::new().x(-1.0),
            },
            AABB::bounding_box([P3::all(1.0)]),
        );
    }

    #[test]
    fn test_aabb_intersection() {
        let test = |a, b, eq| assert_eq!(AABB::intersection([a, b]), eq);
        test(
            AABB {
                min: P3::new().z(-2.0),
                max: P3::all(3.0).x(5.0),
            },
            AABB {
                min: P3::all(-1.0),
                max: P3::all(4.0),
            },
            AABB {
                min: P3::new().z(-1.0),
                max: P3::all(3.0).x(4.0),
            },
        )
    }

    #[test]
    fn test_aabb_union() {
        let a = AABB::bounding_box([P3::new().z(2.0), P3::new().x(2.0)]);
        let b = AABB::bounding_box([P3::new(), P3::new().x(1.0)]);
        assert_eq!(
            AABB::union([a, b]),
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
