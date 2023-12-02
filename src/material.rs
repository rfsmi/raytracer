use rand::random;

use crate::{hit::HitRecord, ray::Ray, util::default_struct, vector::V3};

pub trait Material: Send + Sync {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(V3, Ray)>;
}

default_struct!(Lambertian {
    albedo: V3 = V3::new(),
});

default_struct!(Metal {
    albedo: V3 = V3::new(),
    fuzz: f64 = 0.0,
});

default_struct!(Dielectric { ir: f64 = 1.5 });

impl Material for Lambertian {
    fn scatter(&self, _r: &Ray, hr: &HitRecord) -> Option<(V3, Ray)> {
        let mut scatter_direction = hr.normal + V3::random_unit();
        if scatter_direction.near_zero() {
            scatter_direction = hr.normal;
        }
        // let scatter_direction = V3::random_on_hemisphere(&hr.normal);
        Some((self.albedo, hr.ray(scatter_direction)))
    }
}

impl Material for Metal {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(V3, Ray)> {
        let reflected = r.direction.unit().reflect(&hr.normal) + self.fuzz * V3::random_unit();
        if reflected.dot(&hr.normal) <= 0.0 {
            return None;
        }
        Some((self.albedo, hr.ray(reflected)))
    }
}

impl Material for Dielectric {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(V3, Ray)> {
        fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
            // Use Schlick's approximation for reflectance.
            let r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
            let r0 = r0 * r0;
            r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
        }

        let refraction_ratio = self.ir.powi(if hr.front_face { -1 } else { 1 });
        let unit_direction = r.direction.unit();
        let cos_theta = (-unit_direction).dot(&hr.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = refraction_ratio * sin_theta > 1.0;
        let direction = if cannot_refract || reflectance(cos_theta, refraction_ratio) > random() {
            unit_direction.reflect(&hr.normal) // Must reflect
        } else {
            unit_direction.refract(&hr.normal, refraction_ratio)
        };
        Some((V3::new().x(1.0).y(1.0).z(1.0), hr.ray(direction)))
    }
}
