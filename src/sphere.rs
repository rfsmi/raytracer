use std::sync::Arc;

use crate::{
    aabb::AABB,
    bvh::Plane,
    hit::{Hit, HitRecord},
    material::Material,
    ray::{Interval, Ray},
    vector::P3,
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

    fn split_aabb(&self, plane: Plane) -> (AABB, AABB) {
        let d = (plane.pos - self.center) * plane.axis;
        let mut h = (self.radius * self.radius - d.length_squared()).sqrt();
        if h.is_nan() {
            h = -self.radius;
        }
        let mut lhs = AABB {
            min: self.center - plane.axis * self.radius - plane.axis.others() * h,
            max: self.center + d + plane.axis.others() * h,
        };
        let mut rhs = AABB {
            min: self.center + d - plane.axis.others() * h,
            max: self.center + plane.axis * self.radius + plane.axis.others() * h,
        };
        let center_box = AABB::bounding_box([
            self.center - plane.axis.others() * self.radius,
            self.center + plane.axis.others() * self.radius,
        ]);
        if plane.axis.value(d) < 0.0 {
            rhs.update(center_box);
        } else {
            lhs.update(center_box);
        }
        let aabb = self.aabb();
        (
            AABB::intersection([aabb, lhs]),
            AABB::intersection([aabb, rhs]),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{material::Lambertian, vector::Axis};

    use super::*;

    #[test]
    fn test_split_aabb() {
        let sphere = Sphere::new(P3::new(), 5.0, Arc::new(Lambertian::new()));
        assert_eq!(
            sphere.aabb(),
            AABB {
                min: P3::all(-5.0),
                max: P3::all(5.0)
            }
        );

        // Split inside the sphere
        assert_eq!(
            sphere.split_aabb(Plane {
                axis: Axis::X,
                pos: 3.0
            }),
            (
                AABB {
                    min: P3::all(-5.0),
                    max: P3::all(5.0).x(3.0)
                },
                AABB {
                    min: P3::all(-4.0).x(3.0),
                    max: P3::all(4.0).x(5.0)
                }
            )
        );

        // Split to the right of the sphere
        let (lhs, rhs) = sphere.split_aabb(Plane {
            axis: Axis::X,
            pos: 5.1,
        });
        assert_eq!(lhs, sphere.aabb());
        assert!(rhs.is_empty());

        // Split to the left of the sphere
        let (lhs, rhs) = sphere.split_aabb(Plane {
            axis: Axis::X,
            pos: -5.1,
        });
        assert!(lhs.is_empty());
        assert_eq!(rhs, sphere.aabb());
    }
}
