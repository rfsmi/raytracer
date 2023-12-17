use std::sync::Arc;

use crate::{
    aabb::AABB,
    hit::{Hit, HitRecord},
    material::Material,
    ray::{Interval, Ray},
    vector::{Axis, P3},
};

pub struct Sphere {
    center: P3,
    radius: f64,
    material: Arc<dyn Material>,
}

impl Sphere {
    pub fn new(center: P3, radius: f64, material: Arc<dyn Material>) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval) -> Option<HitRecord> {
        let oc = r.origin - self.center;
        let a = r.direction.length_squared();
        let half_b = oc.dot(&r.direction);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }

        // Check both intersection points.
        let t = [-1.0, 1.0]
            .iter()
            .map(|f| (-half_b + f * discriminant.sqrt()) / a)
            .find(|t| ray_t.surrounds(*t))?;
        let p = r.at(t);
        let outward_normal = (p - self.center) / self.radius;
        Some(HitRecord::new(r, p, t, outward_normal, &*self.material))
    }

    fn aabb(&self) -> AABB {
        AABB::bounding_box([self.center - self.radius, self.center + self.radius])
    }

    fn clipped_aabb(&self, axis: Axis, t1: f64, t2: f64) -> AABB {
        let d1 = (t1 - self.center) * axis;
        let d2 = (t2 - self.center) * axis;
        let mut h = (self.radius * self.radius - d1.min(d2).length_squared()).sqrt();
        if h.is_nan() {
            h = -self.radius;
        }
        let mut aabb = AABB {
            min: self.center + d1 - axis.others() * h,
            max: self.center + d2 + axis.others() * h,
        };
        if axis.value(d1) < 0.0 && 0.0 < axis.value(d2) {
            aabb.update(AABB::bounding_box([
                self.center - axis.others() * self.radius,
                self.center + axis.others() * self.radius,
            ]));
        }
        AABB::intersection([self.aabb(), aabb])
    }
}

#[cfg(test)]
mod test {
    use crate::{material::Lambertian, vector::Axis};

    use super::*;

    #[test]
    fn test_clipped_aabb() {
        let sphere = Sphere::new(P3::new(), 5.0, Arc::new(Lambertian::new()));
        assert_eq!(
            sphere.aabb(),
            AABB {
                min: P3::all(-5.0),
                max: P3::all(5.0)
            }
        );

        // Box inside the sphere
        assert_eq!(
            sphere.clipped_aabb(Axis::X, -3.0, 3.0),
            AABB {
                min: P3::all(-5.0).x(-3.0),
                max: P3::all(5.0).x(3.0),
            }
        );

        // Box intersecting the right of the sphere
        assert_eq!(
            sphere.clipped_aabb(Axis::X, 0.0, 5.2),
            AABB {
                min: P3::all(-5.0).x(0.0),
                max: P3::all(5.0),
            }
        );

        // Box to the right of the sphere
        assert!(sphere.clipped_aabb(Axis::X, 5.1, 5.2).is_empty());

        // Box to the left of the sphere
        assert!(sphere.clipped_aabb(Axis::X, -5.2, -5.1).is_empty());
    }
}
