//! Head mesh — OBJ loading, spatial hash, collision queries.
//!
//! Ported from `hcr_s2\src\head.rs` with the same Möller–Trumbore
//! ray-triangle intersection and ray-parity containment test.

use crate::types::Vec3;

/// Pre-loaded head mesh with spatial hash for fast collision queries.
pub struct HeadMesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
    pub center: Vec3,
    /// Spatial hash: grid_size per axis, cells are lists of triangle indices.
    pub hash_grid: SpatialHash,
}

#[derive(Clone)]
pub struct SpatialHash {
    pub cell_size: f32,
    pub min: Vec3,
    pub dims: [u32; 3],
    pub cells: Vec<Vec<usize>>,
}

impl SpatialHash {
    pub fn new(vertices: &[Vec3], triangles: &[[usize; 3]], cell_size: f32) -> Self {
        let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
        for v in vertices {
            min.x = min.x.min(v.x); min.y = min.y.min(v.y); min.z = min.z.min(v.z);
            max.x = max.x.max(v.x); max.y = max.y.max(v.y); max.z = max.z.max(v.z);
        }
        let dims = [
            ((max.x - min.x) / cell_size).ceil() as u32 + 1,
            ((max.y - min.y) / cell_size).ceil() as u32 + 1,
            ((max.z - min.z) / cell_size).ceil() as u32 + 1,
        ];
        let cell_count = (dims[0] * dims[1] * dims[2]) as usize;
        let mut cells: Vec<Vec<usize>> = Vec::with_capacity(cell_count);
        for _ in 0..cell_count { cells.push(Vec::new()); }
        for (ti, tri) in triangles.iter().enumerate() {
            let v0 = vertices[tri[0]];
            let v1 = vertices[tri[1]];
            let v2 = vertices[tri[2]];
            let tmin = Vec3::new(
                v0.x.min(v1.x).min(v2.x),
                v0.y.min(v1.y).min(v2.y),
                v0.z.min(v1.z).min(v2.z),
            );
            let tmax = Vec3::new(
                v0.x.max(v1.x).max(v2.x),
                v0.y.max(v1.y).max(v2.y),
                v0.z.max(v1.z).max(v2.z),
            );
            let ix0 = ((tmin.x - min.x) / cell_size) as u32;
            let iy0 = ((tmin.y - min.y) / cell_size) as u32;
            let iz0 = ((tmin.z - min.z) / cell_size) as u32;
            let ix1 = ((tmax.x - min.x) / cell_size) as u32;
            let iy1 = ((tmax.y - min.y) / cell_size) as u32;
            let iz1 = ((tmax.z - min.z) / cell_size) as u32;
            for ix in ix0..=ix1.min(dims[0]-1) {
                for iy in iy0..=iy1.min(dims[1]-1) {
                    for iz in iz0..=iz1.min(dims[2]-1) {
                        let idx = (ix + iy * dims[0] + iz * dims[0] * dims[1]) as usize;
                        cells[idx].push(ti);
                    }
                }
            }
        }
        Self { cell_size, min, dims, cells }
    }

    fn cell_index(&self, p: Vec3) -> Option<usize> {
        let ix = ((p.x - self.min.x) / self.cell_size) as i32;
        let iy = ((p.y - self.min.y) / self.cell_size) as i32;
        let iz = ((p.z - self.min.z) / self.cell_size) as i32;
        if ix < 0 || iy < 0 || iz < 0
            || ix >= self.dims[0] as i32 || iy >= self.dims[1] as i32 || iz >= self.dims[2] as i32 {
            return None;
        }
        Some((ix as u32 + iy as u32 * self.dims[0] + iz as u32 * self.dims[0] * self.dims[1]) as usize)
    }

    pub fn candidate_triangles(&self, p: Vec3) -> &[usize] {
        match self.cell_index(p) {
            Some(idx) => &self.cells[idx],
            None => &[],
        }
    }
}

impl HeadMesh {
    /// Load an OBJ file and build the head mesh.
    pub fn load_obj(path: &str, center: Vec3) -> Self {
        let text = std::fs::read_to_string(path).expect("Cannot read head OBJ");
        let mut verts: Vec<Vec3> = Vec::new();
        let mut tris: Vec<[usize; 3]> = Vec::new();

        for line in text.lines() {
            let l = line.trim();
            if l.starts_with("v ") {
                let p: Vec<f32> = l[2..].split_whitespace()
                    .filter_map(|s| s.parse().ok()).collect();
                if p.len() >= 3 {
                    verts.push(Vec3::new(p[0], p[1], p[2]) - center);
                }
            } else if l.starts_with("f ") {
                let idx: Vec<usize> = l[2..].split_whitespace()
                    .filter_map(|s| s.split('/').next()?.parse::<usize>().ok())
                    .map(|i| if i > 0 { i - 1 } else { 0 }).collect();
                if idx.len() == 3 {
                    tris.push([idx[0], idx[1], idx[2]]);
                } else if idx.len() >= 4 {
                    tris.push([idx[0], idx[1], idx[2]]);
                    tris.push([idx[0], idx[2], idx[3]]);
                }
            }
        }
        println!("Head: {} verts, {} tris", verts.len(), tris.len());
        let hash_grid = SpatialHash::new(&verts, &tris, 0.02); // 2cm cells
        Self { vertices: verts, triangles: tris, center: Vec3::ZERO, hash_grid }
    }

    /// Ray-triangle intersection (Möller–Trumbore).
    pub fn ray_triangle(&self, origin: Vec3, dir: Vec3, tri_idx: usize) -> Option<(f32, Vec3)> {
        let [i0, i1, i2] = self.triangles[tri_idx];
        let v0 = self.vertices[i0];
        let v1 = self.vertices[i1];
        let v2 = self.vertices[i2];

        let e1 = v1 - v0;
        let e2 = v2 - v0;
        let h = dir.cross(e2);
        let a = e1.dot(h);

        if a.abs() < 1e-8 { return None; }

        let f = 1.0 / a;
        let s = origin - v0;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 { return None; }

        let q = s.cross(e1);
        let v = f * dir.dot(q);
        if v < 0.0 || u + v > 1.0 { return None; }

        let t = f * e2.dot(q);
        if t > 1e-6 {
            Some((t, origin + dir * t))
        } else {
            None
        }
    }

    /// Raycast from head center along (theta, phi) to find the head surface.
    pub fn raycast(&self, theta: f32, phi: f32) -> Option<(Vec3, Vec3)> {
        let dir = Vec3::new(
            theta.sin() * phi.cos(),
            theta.sin() * phi.sin(),
            theta.cos(),
        );
        let origin = self.center;
        let mut closest_t = f32::MAX;
        let mut closest = None;

        for &ti in self.hash_grid.candidate_triangles(origin + dir * 0.1).iter()
            .chain(self.hash_grid.candidate_triangles(origin + dir * 0.5).iter())
        {
            if let Some((t, pt)) = self.ray_triangle(origin, dir, ti) {
                if t < closest_t {
                    // Compute normal
                    let [i0, i1, i2] = self.triangles[ti];
                    let n = (self.vertices[i1] - self.vertices[i0])
                        .cross(self.vertices[i2] - self.vertices[i0]).normalize();
                    closest_t = t;
                    closest = Some((pt, n));
                }
            }
        }
        closest
    }

    /// Raycast from an arbitrary point toward the head center to find the surface.
    pub fn raycast_from_point(&self, point: Vec3) -> Option<(Vec3, Vec3)> {
        let dir = (self.center - point).normalize();
        let origin = point;
        let mut closest_t = f32::MAX;
        let mut closest = None;

        for &ti in self.hash_grid.candidate_triangles(point).iter()
            .chain(self.hash_grid.candidate_triangles(self.center).iter())
        {
            if let Some((t, pt)) = self.ray_triangle(origin, dir, ti) {
                if t < closest_t && t > 0.0 {
                    let [i0, i1, i2] = self.triangles[ti];
                    let n = (self.vertices[i1] - self.vertices[i0])
                        .cross(self.vertices[i2] - self.vertices[i0]).normalize();
                    closest_t = t;
                    closest = Some((pt, n));
                }
            }
        }
        closest
    }

    /// Point-in-mesh test via ray parity (count intersections along +X ray).
    pub fn contains(&self, point: Vec3) -> bool {
        let dir = Vec3::new(1.0, 0.0, 0.0);
        let mut count = 0;
        let candidates: Vec<usize> = self.hash_grid.candidate_triangles(point).to_vec();
        for &ti in &candidates {
            if let Some((t, _)) = self.ray_triangle(point, dir, ti) {
                if t > 0.0 { count += 1; }
            }
        }
        count % 2 == 1
    }
}
