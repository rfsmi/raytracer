use std::{collections::HashMap, time::Instant};

use indicatif::ParallelProgressIterator;

use crate::{
    hit::Hit,
    ray::{Interval, Ray},
    util::default_struct,
    vector::{P3, V3},
};
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelBridge, ParallelIterator};

default_struct!(Config {
    aspect_ratio: f64 = 16.0 / 9.0,
    image_width: u32 = 400,
    samples_per_pixel: usize = 10,
    max_depth: usize = 10,
    vfov: f64 = 90.0,
    lookfrom: P3 = P3::new().z(-1.0),
    lookat: P3 = P3::new(),
    vup: V3 = V3::new().y(1.0),
    defocus_angle: f64 = 0.0,
    focus_dist: f64 = 10.0,
});

pub struct Camera {
    config: Config,
    image_height: u32,
    center: P3,
    pixel00_loc: P3,
    pixel_delta_u: V3,
    pixel_delta_v: V3,
    defocus_disk_u: V3,
    defocus_disk_v: V3,
}

impl Config {
    pub fn camera(self) -> Camera {
        let image_height: u32 = (self.image_width as f64 / self.aspect_ratio).max(1.0) as u32;
        let center = self.lookfrom;

        // Camera
        let theta = self.vfov.to_radians();
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * self.image_width as f64 / image_height as f64;

        // Calculate the basis vectors
        let w = (self.lookfrom - self.lookat).unit();
        let u = self.vup.cross(&w).unit();
        let v = w.cross(&u);

        // Vectors spanning the viewport
        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;

        // Distances between pixels
        let pixel_delta_u = viewport_u / self.image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        // Find location of upper left pixel
        let viewport_upper_left = center - self.focus_dist * w - (viewport_u + viewport_v) / 2.0;
        let pixel00_loc = viewport_upper_left + (pixel_delta_u + pixel_delta_v) / 2.0;

        // Calculate the camera defocus disk basis vectors
        let defocus_radius = self.focus_dist * (self.defocus_angle / 2.0).to_radians().tan();
        let defocus_disk_u = u * defocus_radius;
        let defocus_disk_v = v * defocus_radius;

        Camera {
            config: self,
            image_height,
            center,
            pixel00_loc,
            pixel_delta_u,
            pixel_delta_v,
            defocus_disk_u,
            defocus_disk_v,
        }
    }
}

fn write_colour(colour: &V3, samples_per_pixel: usize) {
    let colour = colour / samples_per_pixel as f64;
    let intensity = Interval::new(0.0, 1.0);
    let gamma_colour = V3::new()
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
    pub fn ray_colour(&self, world: &dyn Hit, r: &Ray, depth: usize) -> V3 {
        if depth == 0 {
            return V3::new();
        }
        let Some(hr) = world.hit(r, Interval::new(1e-3, f64::INFINITY)) else {
            // Blue sky
            let unit_direction = r.direction.unit();
            let a = (unit_direction.y + 1.0) / 2.0;
            return (1.0 - a) * V3::new().x(1.0).y(1.0).z(1.0) + a * V3::new().x(0.5).y(0.7).z(1.0);
        };
        let Some((attenuation, scattered)) = hr.material.scatter(r, &hr) else {
            return V3::new();
        };
        attenuation * self.ray_colour(world, &scattered, depth - 1)
    }

    fn pixel_sample_square(&self) -> V3 {
        let px = -0.5 + ::rand::random::<f64>();
        let py = -0.5 + ::rand::random::<f64>();
        (px * self.pixel_delta_u) + (py * self.pixel_delta_v)
    }

    fn get_ray(&self, i: u32, j: u32) -> Ray {
        // Get a randomly-sampled camera ray for the pixel at location i,j,
        // originating from the camera defocus disk.
        let pixel_center =
            self.pixel00_loc + (i as f64 * self.pixel_delta_u) + (j as f64 * self.pixel_delta_v);
        let pixel_sample = pixel_center + self.pixel_sample_square();

        let ray_origin = if self.config.defocus_angle <= 0.0 {
            self.center
        } else {
            // Sample defocus disk
            let p = V3::random_within_unit_disk();
            self.center + p.x * self.defocus_disk_u + p.y * self.defocus_disk_v
        };
        let ray_direction = pixel_sample - ray_origin;
        Ray::new(ray_origin, ray_direction)
    }

    pub fn render(&self, world: &dyn Hit) {
        println!("P3\n{} {}\n255", self.config.image_width, self.image_height);
        let start = Instant::now();
        let image: HashMap<_, _> = (0..self.image_height)
            .cartesian_product(0..self.config.image_width)
            .par_bridge()
            .map(move |(j, i)| {
                let colour = (0..self.config.samples_per_pixel)
                    .into_par_iter()
                    .map(|_| {
                        let r = self.get_ray(i, j);
                        self.ray_colour(world, &r, self.config.max_depth)
                    })
                    .reduce(V3::new, |a, b| a + b);
                ((i, j), colour)
            })
            .progress_count((self.image_height * self.config.image_width) as u64)
            .collect();
        eprintln!("Completed in {:.3} seconds", start.elapsed().as_secs_f32());
        for j in 0..self.image_height {
            for i in 0..self.config.image_width {
                write_colour(
                    image.get(&(i, j)).expect("there is a pixel"),
                    self.config.samples_per_pixel,
                );
            }
        }
    }
}

/*


let mut queue: VecDeque<((u32, u32), u32)> =
    (0..self.image_height)
    .flat_map(|i| (0..self.image_width).flat_map(|j| repeat((i, j), 100)));

job_queue = Pixel(0, 0).with_capacity(100);



*/
