use super::mesh::*;

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub lower: Vec3,
    pub upper: Vec3,
}

impl AABB {
    pub fn from_mesh(mesh: &Mesh) -> Self {
        Self::from_iterable(mesh)
    }

    pub fn from_iterable(mesh: impl IntoIterator<Item = Triangle> + Copy) -> Self {
        let mut lower = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
        let mut upper = Vec3::new(f32::MIN, f32::MIN, f32::MIN);

        for t in mesh {
            let v = &t.vertices;

            lower.x = lower.x.min(v[0].x.min(v[1].x).min(v[2].x));
            lower.y = lower.y.min(v[0].y.min(v[1].y).min(v[2].y));
            lower.z = lower.z.min(v[0].z.min(v[1].z).min(v[2].z));

            upper.x = upper.x.max(v[0].x.max(v[1].x).max(v[2].x));
            upper.y = upper.y.max(v[0].y.max(v[1].y).max(v[2].y));
            upper.z = upper.z.max(v[0].z.max(v[1].z).max(v[2].z));
        }

        Self { lower, upper }
    }

    pub fn size(&self) -> Vec3 {
        self.upper - self.lower
    }

    pub fn center(&self) -> Vec3 {
        self.lower + self.size() * 0.5
    }

    pub fn apply_transform(&mut self, transform: &Mat4) {
        self.lower = transform_point(transform, &self.lower);
        self.upper = transform_point(transform, &self.upper);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounds() {
        let mesh = Mesh::new(vec![Triangle::new(
            [
                Vec3::new(1.0, 2.0, 3.0),
                Vec3::new(-1.0, -2.0, -3.0),
                Vec3::new(2.0, 3.0, 4.0),
            ],
            Vec3::new(0.0, 0.0, 0.0),
        )]);

        let aabb = AABB::from_mesh(&mesh);
        assert_eq!(aabb.lower, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(aabb.upper, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn test_center() {
        let mesh = Mesh::new(vec![Triangle::new(
            [
                Vec3::new(1.0, 2.0, 3.0),
                Vec3::new(-1.0, -2.0, -3.0),
                Vec3::new(2.0, 3.0, 4.0),
            ],
            Vec3::new(0.0, 0.0, 0.0),
        )]);

        let aabb = AABB::from_mesh(&mesh);
        assert_eq!(aabb.center(), Vec3::new(0.5, 0.5, 0.5));
    }
}
