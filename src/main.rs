use std::{cell::RefCell, rc::Rc};

use hit::Hit;
use indicatif::ProgressIterator;
use interval::Interval;
use ray::Ray;
use vec3::Vec3;

use crate::{camera::Camera, hit_list::HitList, sphere::Sphere};

mod camera;
mod hit;
mod hit_list;
mod interval;
mod ray;
mod sphere;
mod vec3;

fn shared<T: Hit + 'static>(object: T) -> Rc<RefCell<dyn Hit>> {
    Rc::new(RefCell::new(object))
}

fn main() {
    let mut world = HitList::new();
    world.add(shared(Sphere::new(Vec3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(shared(Sphere::new(Vec3::new(0.0, -100.5, -1.0), 100.0)));

    Camera::new(16.0 / 9.0, 400).render(&world);
}
