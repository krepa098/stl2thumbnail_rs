use crate::parser::Parser;
use std::cell::RefCell;
use std::io::{Read, Seek};
use std::ops::Index;

// glm aliases
pub type Mat4 = glm::Mat4x4;
pub type Vec2 = glm::Vec2;
pub type Vec3 = glm::Vec3;
pub type Vec4 = glm::Vec4;

// helpers
pub fn matmul(m: &Mat4, v: &Vec3) -> Vec3 {
    (m * v.to_homogeneous()).xyz()
}

// Triangle
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triangle {
    pub vertices: [Vec3; 3],
    pub normal: Vec3,
}

impl Triangle {
    pub fn new(vertices: [Vec3; 3], normal: Vec3) -> Self {
        Self { vertices, normal }
    }
}

// Mesh
pub struct Mesh {
    triangles: Vec<Triangle>,
}

impl Mesh {
    pub fn new(triangles: Vec<Triangle>) -> Self {
        Self { triangles }
    }
}

impl Mesh {
    pub fn len(&self) -> usize {
        self.triangles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }
}

impl Index<usize> for Mesh {
    type Output = Triangle;
    fn index(&self, index: usize) -> &Self::Output {
        &self.triangles[index]
    }
}

pub struct MeshIter<'a> {
    mesh: &'a [Triangle],
    i: usize,
}

impl<'a> IntoIterator for &'a Mesh {
    type Item = Triangle;
    type IntoIter = MeshIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            mesh: &self.triangles,
            i: 0,
        }
    }
}

impl<'a> Iterator for MeshIter<'a> {
    type Item = Triangle;

    fn next(&mut self) -> Option<Self::Item> {
        let triangle = self.mesh.get(self.i).copied();
        self.i += 1;
        triangle
    }
}

// LazyMesh
pub struct LazyMesh<'a, T: Read + Seek> {
    parser: RefCell<&'a mut Parser<T>>, // inner mutability
}

impl<'a, T> LazyMesh<'a, T>
where
    T: Read + Seek,
{
    pub fn new(parser: &'a mut Parser<T>) -> Self {
        Self {
            parser: RefCell::new(parser),
        }
    }
}

pub struct LazyMeshIter<'a, T: Read + Seek> {
    parser: &'a RefCell<&'a mut Parser<T>>,
}

impl<'a, T: Read + Seek> IntoIterator for &'a LazyMesh<'a, T> {
    type Item = Triangle;
    type IntoIter = LazyMeshIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.parser.borrow_mut().rewind().unwrap();
        Self::IntoIter { parser: &self.parser }
    }
}

impl<'a, T: Read + Seek> Iterator for LazyMeshIter<'a, T> {
    type Item = Triangle;

    fn next(&mut self) -> Option<Self::Item> {
        self.parser.borrow_mut().next_triangle()
    }
}
