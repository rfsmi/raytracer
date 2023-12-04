use crate::{
    ray::{Interval, Ray},
    vector::{P3, V3},
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum AABB {
    Empty,
    NonEmpty { min: P3, max: P3 },
}

impl AABB {
    pub fn new() -> Self {
        Self::Empty
    }

    pub fn min(&self) -> Option<P3> {
        match self {
            AABB::Empty => None,
            AABB::NonEmpty { min, .. } => Some(*min),
        }
    }

    pub fn max(&self) -> Option<P3> {
        match self {
            AABB::Empty => None,
            AABB::NonEmpty { max, .. } => Some(*max),
        }
    }

    pub fn centroid(&self) -> Option<P3> {
        let min = self.min()?;
        let max = self.max()?;
        Some(min + 0.5 * (max - min))
    }

    pub fn size(&self) -> V3 {
        match self {
            AABB::Empty => V3::new(),
            AABB::NonEmpty { min, max } => max - min,
        }
    }

    pub fn surface_area(&self) -> f64 {
        match self {
            AABB::Empty => 0.0,
            AABB::NonEmpty { min, max } => {
                let V3 { x, y, z } = max - min;
                2.0 * (x * y + x * z + z * y)
            }
        }
    }

    pub fn extend(&mut self, other: Self) {
        *self = Self::union([*self, other]);
    }

    pub fn bounding_box(points: impl IntoIterator<Item = P3>) -> Self {
        let mut points = points.into_iter();
        let Some(p) = points.next() else {
            return Self::Empty;
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
        Self::NonEmpty { min, max }
    }

    pub fn union(aabbs: impl IntoIterator<Item = Self>) -> Self {
        aabbs.into_iter().fold(Self::Empty, |a, b| {
            let Self::NonEmpty {
                min: a_min,
                max: a_max,
            } = a
            else {
                return b;
            };
            let Self::NonEmpty {
                min: b_min,
                max: b_max,
            } = b
            else {
                return a;
            };
            Self::NonEmpty {
                min: P3::new()
                    .x(a_min.x.min(b_min.x))
                    .y(a_min.y.min(b_min.y))
                    .z(a_min.z.min(b_min.z)),
                max: P3::new()
                    .x(a_max.x.max(b_max.x))
                    .y(a_max.y.max(b_max.y))
                    .z(a_max.z.max(b_max.z)),
            }
        })
    }

    pub fn ray_intersection(&self, r: &Ray, ray_t: Interval) -> Option<Interval> {
        let Self::NonEmpty { min, max } = self else {
            return None;
        };
        let t1 = (min - r.origin) * r.inv_direction;
        let t2 = (max - r.origin) * r.inv_direction;
        let tmin = t1.min(t2);
        let tmax = t1.max(t2);
        let tmin = tmin.x.max(tmin.y).max(tmin.z);
        let tmax = tmax.x.min(tmax.y).min(tmax.z);
        if tmax < tmin || tmax < ray_t.min || ray_t.max < tmin {
            return None;
        }
        Some(Interval::new(tmin, tmax))
    }
}
