use std::{cell::RefCell, rc::Rc};

use indicatif::ProgressIterator;

use crate::{
    hit::Hit, hit_list::HitList, interval::Interval, ray::Ray, sphere::Sphere, vec3::Vec3,
};

pub struct Camera {
    aspect_ratio: f64,
    image_width: u32,
    image_height: u32,
    center: Vec3,
    pixel00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

fn make_shared<T: Hit + 'static>(object: T) -> Rc<RefCell<dyn Hit>> {
    Rc::new(RefCell::new(object))
}

impl Camera {
    pub fn new(aspect_ratio: f64, image_width: u32) -> Self {
        let image_height: u32 = (image_width as f64 / aspect_ratio).max(1.0) as u32;
        let center = Vec3::new(0.0, 0.0, 0.0);

        // Camera
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * image_width as f64 / image_height as f64;
        let camera_center = Vec3::default();

        // Vectors spanning the viewport
        let viewport_u = Vec3::U * viewport_width;
        let viewport_v = Vec3::V * -viewport_height;

        // Distances between pixels
        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Find location of upper left pixel
        let viewport_upper_left =
            camera_center - Vec3::W * focal_length - (viewport_u + viewport_v) / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) / 2.0;
        Self {
            aspect_ratio,
            image_width,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    pub fn ray_colour(&self, r: &Ray, world: &dyn Hit) -> Vec3 {
        if let Some(hr) = world.hit(r, Interval::new(0.0, f64::INFINITY)) {
            return (hr.normal + Vec3::UVW) / 2.0;
        }

        let unit_direction = r.direction.unit();
        let a = (unit_direction.y + 1.0) / 2.0;
        (1.0 - a) * Vec3::new(1.0, 1.0, 1.0) + a * Vec3::new(0.5, 0.7, 1.0)
    }

    pub fn render(&self, world: &dyn Hit) {
        println!("P3\n{} {}\n255", self.image_width, self.image_height);
        for j in (0..self.image_height).progress() {
            for i in 0..self.image_width {
                let pixel_center = self.pixel00_loc
                    + (i as f64 * self.pixel_delta_u)
                    + (j as f64 * self.pixel_delta_v);
                let ray_direction = pixel_center - self.center;
                let ray = Ray::new(self.center, ray_direction);

                let colour = self.ray_colour(&ray, world);
                let r = (255.0 * colour.x) as u8;
                let g = (255.0 * colour.y) as u8;
                let b = (255.0 * colour.z) as u8;
                println!("{r} {g} {b}")
            }
        }
    }
}
