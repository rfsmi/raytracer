use std::{cell::RefCell, rc::Rc};

use hit::Hit;
use indicatif::ProgressIterator;
use interval::Interval;
use ray::Ray;
use vec3::Vec3;

use crate::{hit_list::HitList, sphere::Sphere};

mod hit;
mod hit_list;
mod interval;
mod ray;
mod sphere;
mod vec3;

fn make_shared<T: Hit + 'static>(object: T) -> Rc<RefCell<dyn Hit>> {
    Rc::new(RefCell::new(object))
}

fn ray_colour(r: &Ray, world: &dyn Hit) -> Vec3 {
    if let Some(hr) = world.hit(r, Interval::new(0.0, f64::INFINITY)) {
        return (hr.normal + Vec3::UVW) / 2.0;
    }

    let unit_direction = r.direction.unit();
    let a = (unit_direction.y + 1.0) / 2.0;
    (1.0 - a) * Vec3::new(1.0, 1.0, 1.0) + a * Vec3::new(0.5, 0.7, 1.0)
}

fn print_colour(colour: &Vec3) {
    let r = (255.0 * colour.x) as u8;
    let g = (255.0 * colour.y) as u8;
    let b = (255.0 * colour.z) as u8;
    println!("{r} {g} {b}")
}

fn main() {
    // Image
    let aspect_ratio = 16.0 / 9.0;
    let image_width: u32 = 400;
    let image_height: u32 = (image_width as f64 / aspect_ratio).max(1.0) as u32;

    // World
    let mut world = HitList::new();
    world.add(make_shared(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(make_shared(Sphere::new(
        Vec3::new(0.0, -100.5, -1.0),
        100.0,
    )));

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

    // Render
    println!("P3\n{} {}\n255", image_width, image_height);
    for j in (0..image_height).progress() {
        for i in 0..image_width {
            let pixel_center =
                pixel00_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let ray = Ray::new(camera_center, ray_direction);

            let color = ray_colour(&ray, &world);
            print_colour(&color);
        }
    }
}
