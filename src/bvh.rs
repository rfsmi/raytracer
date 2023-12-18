use std::{collections::HashSet, iter::once};

use glam::DVec3;

use crate::{
    aabb::AABB,
    hit::{Hit, HitRecord},
    ray::{Interval, Ray},
};

#[derive(Clone, Copy, PartialEq)]
pub struct Plane {
    pub axis: DVec3,
    pub pos: f64,
}

enum Node {
    Split {
        aabb: AABB,
        axis: DVec3,
        skip: usize,
    },
    Leaf {
        aabb: AABB,
        indices: Vec<usize>,
    },
}

impl Node {
    fn aabb(&self) -> &AABB {
        match self {
            Node::Split { aabb, .. } => aabb,
            Node::Leaf { aabb, .. } => aabb,
        }
    }
}

pub struct BVH {
    nodes: Vec<Node>,
    primitives: Vec<Box<dyn Hit>>,
}

impl BVH {
    pub fn new(primitives: impl IntoIterator<Item = Box<dyn Hit>>) -> Self {
        let primitives: Vec<_> = primitives.into_iter().collect();
        let nodes = Builder::new(&primitives).build((0..primitives.len()).collect());
        Self { nodes, primitives }
    }

    pub fn hit<'a>(&'a self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord<'a>> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut stack = vec![0];
        let mut best_hr = None;
        while let Some(i) = stack.pop() {
            let node = &self.nodes[i];
            if node.aabb().ray_intersection(r, ray_t).is_none() {
                continue;
            }

            match node {
                &Node::Split { axis, skip, .. } => {
                    stack.extend(if get_axis(axis)(r.direction) < 0.0 {
                        [i + 1, i + skip]
                    } else {
                        [i + skip, i + 1]
                    });
                }
                Node::Leaf { indices, .. } => {
                    best_hr = indices
                        .iter()
                        .filter_map(|&i| {
                            let hr = self.primitives[i].hit(r, ray_t)?;
                            ray_t.max = hr.t;
                            Some(hr)
                        })
                        .last()
                        .or(best_hr);
                }
            }
        }
        best_hr
    }
}

const LEAF_COST: f64 = 1.0;
const SPLIT_COST: f64 = 1.0 / 8.0;
const N_BUCKETS: usize = 32;
const SBVH_ALPHA: f64 = 1e-6;

struct Builder<'p> {
    root_aabb: AABB,
    primitives: &'p [Box<dyn Hit>],
}

impl<'p> Builder<'p> {
    fn new(primitives: &'p [Box<dyn Hit>]) -> Self {
        Self {
            root_aabb: AABB::union(primitives.iter().map(|p| p.aabb())),
            primitives,
        }
    }

    fn build(&self, indices: Vec<usize>) -> Vec<Node> {
        let aabb = AABB::union(indices.iter().map(|&i| self.primitives[i].aabb()));

        let Some(mut split) = best_split(aabb, self.object_buckets(&indices)) else {
            return vec![Node::Leaf { aabb, indices }];
        };

        // Attempt a spatial split if the best object split has enough overlap
        let (_, lhs, rhs, _) = &split;
        let overlap = AABB::intersection([lhs.aabb, rhs.aabb]).surface_area();
        if overlap / self.root_aabb.surface_area() > SBVH_ALPHA {
            split = once(split)
                .chain(best_split(aabb, self.spatial_buckets(&indices)))
                .min_by(|(.., a), (.., b)| a.total_cmp(b))
                .unwrap();
        }

        let (axis, lhs, rhs, cost) = split;
        // Split if we must, or the cost is low enough
        if cost < LEAF_COST * indices.len() as f64 {
            let mut nodes = self.build(lhs.primitives.into_iter().collect());
            let parent = Node::Split {
                axis,
                aabb,
                skip: nodes.len() + 1,
            };
            nodes.insert(0, parent);
            nodes.extend(self.build(rhs.primitives.into_iter().collect()));
            return nodes;
        }

        // Otherwise this is just a leaf node
        vec![Node::Leaf { aabb, indices }]
    }

    pub fn object_buckets<'a>(
        &'a self,
        indices: &'a [usize],
    ) -> impl Iterator<Item = (DVec3, Vec<Bucket>)> + 'a {
        let centroid = |&i: &usize| self.primitives[i].aabb().centroid();
        let bounds = AABB::bounding_box(indices.iter().map(centroid));
        iter_bounds_axes(bounds).map(move |(axis, min_t, max_t)| {
            // Assign each primitive to its bucket
            let mut buckets = vec![Bucket::new(); N_BUCKETS];
            for &i in indices {
                let t = get_axis(axis)(self.primitives[i].aabb().centroid());
                let bucket_i = choose_bucket(&mut buckets, min_t, max_t, t);
                buckets[bucket_i].add(i, self.primitives[i].aabb());
            }
            (axis, buckets)
        })
    }

    pub fn spatial_buckets<'a>(
        &'a self,
        indices: &'a [usize],
    ) -> impl Iterator<Item = (DVec3, Vec<Bucket>)> + 'a {
        let bounds = AABB::union(indices.iter().map(|&i| self.primitives[i].aabb()));
        iter_bounds_axes(bounds).map(move |(axis, min_t, max_t)| {
            // Split each primitive into the buckets it overlaps
            let mut buckets = vec![Bucket::new(); N_BUCKETS];
            let bucket_width = get_axis(axis)(bounds.size()) / buckets.len() as f64;
            for &i in indices {
                let aabb = self.primitives[i].aabb();
                let first_bucket = choose_bucket(&buckets, min_t, max_t, get_axis(axis)(aabb.min));
                for bucket_i in first_bucket..buckets.len() {
                    let lhs = min_t + bucket_i as f64 * bucket_width;
                    let rhs = lhs + bucket_width;
                    let clipped_aabb = self.primitives[i].clipped_aabb(axis, lhs, rhs);
                    buckets[bucket_i].add(i, clipped_aabb);
                }
            }
            (axis, buckets)
        })
    }
}

fn get_axis(axis: DVec3) -> impl Fn(DVec3) -> f64 {
    move |v| {
        if axis == DVec3::X {
            v.x
        } else if axis == DVec3::Y {
            v.y
        } else if axis == DVec3::Z {
            v.z
        } else {
            panic!()
        }
    }
}

fn iter_bounds_axes(bounds: AABB) -> impl Iterator<Item = (DVec3, f64, f64)> {
    DVec3::AXES
        .into_iter()
        .map(move |axis| (axis, get_axis(axis)(bounds.min), get_axis(axis)(bounds.max)))
        .filter(|(_, min_t, max_t)| (max_t - min_t) > 0.0)
}

fn best_split(
    bounds: AABB,
    candidate_buckets: impl Iterator<Item = (DVec3, Vec<Bucket>)>,
) -> Option<(DVec3, Bucket, Bucket, f64)> {
    candidate_buckets
        .flat_map(|(axis, buckets)| {
            // TODO: This can be optimised
            // Split at each point
            (1..buckets.len()).map(move |i| {
                let (part_a, part_b) = buckets.split_at(i);
                let a = combine_buckets(part_a);
                let b = combine_buckets(part_b);
                (axis, a, b)
            })
        })
        .map(|(axis, a, b)| {
            // Determine the cost
            let cost = SPLIT_COST
                + LEAF_COST
                    * (a.primitives.len() as f64 * a.aabb.surface_area()
                        + b.primitives.len() as f64 * b.aabb.surface_area())
                    / bounds.surface_area();
            (axis, a, b, cost)
        })
        .min_by(|(.., a), (.., b)| a.total_cmp(b))
}

#[derive(Clone)]
struct Bucket {
    primitives: HashSet<usize>,
    aabb: AABB,
}

impl Bucket {
    fn new() -> Self {
        Bucket {
            primitives: HashSet::new(),
            aabb: AABB::new(),
        }
    }

    fn add(&mut self, primitive: usize, aabb: AABB) {
        self.primitives.insert(primitive);
        self.aabb.update(aabb);
    }
}

fn choose_bucket(buckets: &[Bucket], min_t: f64, max_t: f64, t: f64) -> usize {
    let f = (t - min_t) / (max_t - min_t);
    let i = (buckets.len() as f64 * f) as usize;
    if i == buckets.len() {
        i - 1
    } else {
        i
    }
}

fn combine_buckets<'a>(buckets: impl IntoIterator<Item = &'a Bucket>) -> Bucket {
    buckets.into_iter().fold(Bucket::new(), |mut a, b| {
        a.primitives.extend(&b.primitives);
        a.aabb.update(b.aabb);
        a
    })
}
