use crate::{hit::HitRecord, ray::Ray, util::default_struct, vec3::Vec3};

pub trait Material {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(Vec3, Ray)>;
}

default_struct!(Lambertian {
    albedo: Vec3 = Vec3::new(),
});

default_struct!(Metal {
    albedo: Vec3 = Vec3::new(),
    fuzz: f64 = 0.0,
});

impl Material for Lambertian {
    fn scatter(&self, _r: &Ray, hr: &HitRecord) -> Option<(Vec3, Ray)> {
        let mut scatter_direction = hr.normal + Vec3::random_unit();
        if scatter_direction.near_zero() {
            scatter_direction = hr.normal;
        }
        // let scatter_direction = Vec3::random_on_hemisphere(&hr.normal);
        let scattered = Ray::new(hr.p, scatter_direction);
        Some((self.albedo, scattered))
    }
}

impl Material for Metal {
    fn scatter(&self, r: &Ray, hr: &HitRecord) -> Option<(Vec3, Ray)> {
        let reflected = r.direction.unit().reflect(&hr.normal) + self.fuzz * Vec3::random_unit();
        if reflected.dot(&hr.normal) <= 0.0 {
            return None;
        }
        let scattered = Ray::new(hr.p, reflected);
        Some((self.albedo, scattered))
    }
}
