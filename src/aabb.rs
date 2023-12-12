use crate::{
    bvh::Axis,
    ray::{Interval, Ray},
    vector::{P3, V3},
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
        Axis::ALL
            .iter()
            .any(|&axis| self.min.axis(axis) > self.max.axis(axis))
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

    pub fn extend(&mut self, other: Self) {
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
