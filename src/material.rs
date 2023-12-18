use glam::DVec3;
use rand::random;

use crate::{hit::HitRecord, ray::Ray, util::default_struct, vector};

pub trait Material: Send + Sync {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(DVec3, Ray)>;
}

default_struct!(Lambertian {
    albedo: DVec3 = DVec3::ZERO,
});

default_struct!(Metal {
    albedo: DVec3 = DVec3::ZERO,
    fuzz: f64 = 0.0,
});

default_struct!(Dielectric { ir: f64 = 1.5 });

impl Material for Lambertian {
    fn scatter(&self, _r: &Ray, hr: &HitRecord) -> Option<(DVec3, Ray)> {
        let mut scatter_direction = hr.normal + vector::random_unit();
        if vector::near_zero(scatter_direction) {
            scatter_direction = hr.normal;
        }
        // let scatter_direction = V3::random_on_hemisphere(&hr.normal);
        Some((self.albedo, hr.ray(scatter_direction)))
    }
}

impl Material for Metal {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(DVec3, Ray)> {
        let reflected =
            vector::reflect(r.direction.normalize(), hr.normal) + self.fuzz * vector::random_unit();
        if reflected.dot(hr.normal) <= 0.0 {
            return None;
        }
        Some((self.albedo, hr.ray(reflected)))
    }
}

impl Material for Dielectric {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(DVec3, Ray)> {
        fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
            // Use Schlick's approximation for reflectance.
            let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
            let r0 = r0 * r0;
            r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
        }

        let refraction_ratio = self.ir.powi(if hr.front_face { -1 } else { 1 });
        let unit_direction = r.direction.normalize();
        let cos_theta = (-unit_direction).dot(hr.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > random() {
            vector::reflect(unit_direction, hr.normal) // Must reflect
        } else {
            vector::refract(unit_direction, hr.normal, refraction_ratio)
        };
        Some((DVec3::ONE, hr.ray(direction)))
    }
}
