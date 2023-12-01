use indicatif::ProgressIterator;

use crate::{hit::Hit, interval::Interval, ray::Ray, util::default_struct, vec3::Vec3};

default_struct!(Config {
    aspect_ratio: f64 = 16.0 / 9.0,
    image_width: u32 = 400,
    samples_per_pixel: usize = 10,
    max_depth: usize = 10,
});

impl Config {
    pub fn camera(self) -> Camera {
        let image_height: u32 = (self.image_width as f64 / self.aspect_ratio).max(1.0) as u32;
        let center = Vec3::new();

        // Camera
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * self.image_width as f64 / image_height as f64;
        let camera_center = Vec3::new();

        // Vectors spanning the viewport
        let viewport_u = Vec3::new().x(viewport_width);
        let viewport_v = Vec3::new().y(-viewport_height);

        // Distances between pixels
        let pixel_delta_u = viewport_u / self.image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Find location of upper left pixel
        let viewport_upper_left =
            camera_center - Vec3::new().z(1.0) * focal_length - (viewport_u + viewport_v) / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) / 2.0;
        Camera {
            config: self,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
        }
    }
}

pub struct Camera {
    config: Config,
    image_height: u32,
    center: Vec3,
    pixel00_loc: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

fn write_colour(mut colour: Vec3, samples_per_pixel: usize) {
    colour /= samples_per_pixel as f64;
    let intensity = Interval::new(0.0, 1.0);
    let gamma_colour = Vec3::new()
        .x(colour.x.sqrt())
        .y(colour.y.sqrt())
        .z(colour.z.sqrt());
    println!(
        "{} {} {}",
        (255.0 * intensity.clamp(gamma_colour.x)) as u8,
        (255.0 * intensity.clamp(gamma_colour.y)) as u8,
        (255.0 * intensity.clamp(gamma_colour.z)) as u8,
    );
}

impl Camera {
    pub fn ray_colour(&self, world: &dyn Hit, r: &Ray, depth: usize) -> Vec3 {
        if depth == 0 {
            return Vec3::new();
        }
        let Some(hr) = world.hit(r, Interval::new(1e-3, f64::INFINITY)) else {
            // Blue sky
            let unit_direction = r.direction.unit();
            let a = (unit_direction.y + 1.0) / 2.0;
            return (1.0 - a) * Vec3::new().x(1.0).y(1.0).z(1.0)
                + a * Vec3::new().x(0.5).y(0.7).z(1.0);
        };
        let Some((attenuation, scattered)) = hr.material.scatter(r, &hr) else {
            return Vec3::new();
        };
        attenuation * self.ray_colour(world, &scattered, depth - 1)
    }

    fn pixel_sample_square(&self) -> Vec3 {
        let px = -0.5 + ::rand::random::<f64>();
        let py = -0.5 + ::rand::random::<f64>();
        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }

    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let pixel_center =
            self.pixel00_loc + (i as f64 * self.pixel_delta_u) + (j as f64 * self.pixel_delta_v);
        let pixel_sample = pixel_center + self.pixel_sample_square();

        let ray_direction = pixel_sample - self.center;
        Ray::new(self.center, ray_direction)
    }

    pub fn render(&self, world: &dyn Hit) {
        println!("P3\n{} {}\n255", self.config.image_width, self.image_height);
        for j in (0..self.image_height).progress() {
            for i in 0..self.config.image_width {
                let mut colour = Vec3::new();
                for _ in 0..self.config.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    colour += self.ray_colour(world, &r, self.config.max_depth);
                }
                write_colour(colour, self.config.samples_per_pixel);
            }
        }
    }

    pub fn config(&self) -> Config {
        self.config
    }
}
