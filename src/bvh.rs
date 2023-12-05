use crate::{
    aabb::AABB,
    hit::{Hit, HitRecord},
    ray::{Interval, Ray},
    vector::{P3, V3},
};

#[derive(Clone, Copy)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    const ALL: &'static [Axis] = &[Axis::X, Axis::Y, Axis::Z];

    fn v3(self, v: V3) -> f64 {
        match self {
            Axis::X => v.x,
            Axis::Y => v.y,
            Axis::Z => v.z,
        }
    }

    fn p3(self, p: P3) -> f64 {
        match self {
            Axis::X => p.x,
            Axis::Y => p.y,
            Axis::Z => p.z,
        }
    }
}

const PRIMITIVE_TEST_COST: f64 = 1.0;
const BVH_SPLIT_COST: f64 = 1.0 / 8.0;

enum Node {
    ObjectSplit { aabb: AABB, axis: Axis, skip: usize },
    Leaf { aabb: AABB, indices: Vec<usize> },
}

impl Node {
    fn build(primitives: &[Box<dyn Hit>], mut indices: Vec<usize>) -> Vec<Node> {
        let current_primitives = indices.iter().map(|&i| &primitives[i]);
        let aabb = AABB::union(current_primitives.clone().map(|o| o.aabb()));

        if let Some((axis, split_i, cost)) = object_split::find_best(aabb, current_primitives) {
            // Split if we must, or the cost is low enough
            if cost < PRIMITIVE_TEST_COST * indices.len() as f64 {
                indices.sort_by(|&a, &b| {
                    axis.p3(primitives[a].aabb().centroid().unwrap())
                        .total_cmp(&axis.p3(primitives[b].aabb().centroid().unwrap()))
                });
                let rest = indices.split_off(split_i);
                return Self::object_split(aabb, axis, primitives, indices, rest);
            }
        };

        // Otherwise this is just a leaf node
        vec![Self::Leaf { aabb, indices }]
    }

    fn object_split(
        aabb: AABB,
        axis: Axis,
        primitives: &[Box<dyn Hit>],
        a: Vec<usize>,
        b: Vec<usize>,
    ) -> Vec<Node> {
        let mut nodes = Self::build(primitives, a);
        let parent = Node::ObjectSplit {
            axis,
            aabb,
            skip: nodes.len() + 1,
        };
        nodes.insert(0, parent);
        nodes.extend(Self::build(primitives, b));
        nodes
    }

    fn aabb(&self) -> &AABB {
        match self {
            Node::ObjectSplit { aabb, .. } => aabb,
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
        Self {
            nodes: Node::build(&primitives, (0..primitives.len()).collect()),
            primitives,
        }
    }
}

impl Hit for BVH {
    fn hit<'a>(&'a self, r: &Ray, mut ray_t: Interval) -> Option<HitRecord<'a>> {
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
                Node::ObjectSplit { axis, skip, .. } => {
                    stack.extend(if axis.v3(r.direction).is_sign_negative() {
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

    fn aabb(&self) -> AABB {
        if self.nodes.len() == 0 {
            AABB::Empty
        } else {
            *self.nodes[0].aabb()
        }
    }
}

mod object_split {
    use super::*;

    const N_BUCKETS: usize = 12;

    #[derive(Clone, Copy)]
    struct Bucket {
        count: usize,
        aabb: AABB,
    }

    impl Bucket {
        fn new() -> Self {
            Bucket {
                count: 0,
                aabb: AABB::new(),
            }
        }

        fn add(&mut self, aabb: AABB) {
            self.count += 1;
            self.aabb.extend(aabb);
        }
    }

    fn combine_buckets<'a>(buckets: impl IntoIterator<Item = &'a Bucket>) -> Bucket {
        buckets.into_iter().fold(Bucket::new(), |a, b| Bucket {
            count: a.count + b.count,
            aabb: AABB::union([a.aabb, b.aabb]),
        })
    }

    pub fn find_best<'a>(
        bounds: AABB,
        primitives: impl Iterator<Item = &'a Box<dyn Hit>> + Clone,
    ) -> Option<(Axis, usize, f64)> {
        let centroids = primitives.clone().filter_map(|o| o.aabb().centroid());
        let centroid_bounds = AABB::bounding_box(centroids);
        Axis::ALL
            .iter()
            // All centroids are at the same point
            .filter(|axis| axis.v3(centroid_bounds.size()) > 0.0)
            .flat_map(move |&axis| {
                // Assign each primitive to its bucket
                let mut buckets = [Bucket::new(); N_BUCKETS];
                let bounds_min = axis.p3(centroid_bounds.min().unwrap());
                let bounds_max = axis.p3(centroid_bounds.max().unwrap());
                for primitive in primitives.clone() {
                    let centroid = axis.p3(primitive.aabb().centroid().unwrap());
                    let f = (centroid - bounds_min) / (bounds_max - bounds_min);
                    let mut i = (N_BUCKETS as f64 * f) as usize;
                    if i == N_BUCKETS {
                        i -= 1;
                    }
                    buckets[i].add(primitive.aabb());
                }
                // Split at each point
                (1..N_BUCKETS).map(move |i| {
                    let (part_a, part_b) = buckets.split_at(i);
                    let a = combine_buckets(part_a);
                    let b = combine_buckets(part_b);
                    (axis, a, b)
                })
            })
            .map(|(axis, a, b)| {
                // Determine the cost
                let cost = BVH_SPLIT_COST
                    + PRIMITIVE_TEST_COST
                        * (a.count as f64 * a.aabb.surface_area()
                            + b.count as f64 * b.aabb.surface_area())
                        / bounds.surface_area();
                (axis, a.count, cost)
            })
            .min_by(|(_, _, a), (_, _, b)| a.total_cmp(b))
    }
}
