Raytracer
---------

Based on the book Ray Tracing in One Weekend [[1]](#1). Currently only renders
spheres.

All rendering is done on CPU with Rayon for parallelisation. A BVH is used for
acceleration.

A SBVH [[2]](#2) implementation is WIP.

<a id="1">[1]</a>https://raytracing.github.io/books/RayTracingInOneWeekend.html  
<a id="2">[2]</a>https://www.nvidia.in/docs/IO/77714/sbvh.pdf