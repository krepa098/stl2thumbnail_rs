use crate::mesh::*;
use anyhow::bail;
use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use scan_fmt::scan_fmt;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};

const HEADER_SIZE: u64 = 80;
const TRIANGLE_SIZE: u64 = 50;

#[derive(Debug, Clone, Copy)]
pub enum StlType {
    Binary,
    Ascii,
}

pub struct Parser<T>
where
    T: Read + Seek,
{
    reader: BufReader<T>,
    stl_type: StlType,
    header_length: u64,
    recalculate_normals: bool,
}

impl<T: Read + Seek> Parser<T> {
    pub fn from_buf(inner: T, recalculate_normals: bool) -> Result<Self> {
        let mut reader = BufReader::new(inner);

        let stl_type = deduce_stl_type(&mut reader)?;
        reader.rewind()?;

        // figure out header size
        let mut header_length = 0;
        match stl_type {
            StlType::Binary => {
                header_length = HEADER_SIZE + 4; // header size + triangle count (u32)
            }
            StlType::Ascii => {
                while let Ok(line) = read_ascii_line(&mut reader) {
                    if line.starts_with("solid") {
                        header_length = reader.stream_position()?;
                        break;
                    }
                }
            }
        };

        Ok(Self {
            reader,
            stl_type,
            header_length,
            recalculate_normals,
        })
    }

    pub fn rewind(&mut self) -> Result<()> {
        self.reader.seek(SeekFrom::Start(self.header_length))?;
        Ok(())
    }

    pub fn next_triangle(&mut self) -> Option<Triangle> {
        let mut triangle = match self.stl_type {
            StlType::Ascii => read_ascii_triangle(&mut self.reader).ok(),
            StlType::Binary => read_triangle(&mut self.reader).ok(),
        };

        // calculate normal from vertices using right hand rule is case it is missing
        if let Some(triangle) = &mut triangle {
            if self.recalculate_normals
                || triangle.normal == Vec3::new(0.0, 0.0, 0.0)
                || triangle.normal == Vec3::new(f32::NAN, f32::NAN, f32::NAN)
            {
                triangle.normal = (triangle.vertices[1] - triangle.vertices[0])
                    .cross(&(triangle.vertices[2] - triangle.vertices[0]))
                    .normalize();
            }
        }

        triangle
    }

    pub fn triangle_count(&mut self) -> Result<u64> {
        self.rewind()?;

        match self.stl_type {
            StlType::Binary => {
                // for binary files we can quickly read the first u32
                // after the header which contains the triangle count
                self.reader.seek(SeekFrom::Current(-4))?;
                let count = self.reader.read_u32::<LittleEndian>()? as u64;
                Ok(count)
            }
            StlType::Ascii => {
                // we have no other choice as parsing the hole file
                let mut count = 0;
                while self.next_triangle().is_some() {
                    count += 1;
                }
                Ok(count)
            }
        }
    }

    pub fn read_all(&mut self) -> Result<Mesh> {
        self.rewind()?;
        let mut triangles = vec![];

        while let Some(triangle) = self.next_triangle() {
            triangles.push(triangle);
        }

        Ok(Mesh::new(triangles))
    }
}

impl Parser<fs::File> {
    pub fn from_file(filename: &str, recalculate_normals: bool) -> Result<Self> {
        let file = fs::File::open(filename)?;
        (&file).seek(SeekFrom::Start(0))?;

        Self::from_buf(file, recalculate_normals)
    }
}

fn deduce_stl_type<T: BufRead + io::Seek>(reader: &mut T) -> Result<StlType> {
    // check if file starts with 'solid', though
    // some malformed binary files also start with 'solid'
    let mut data = vec![0u8; 5];
    reader.read_exact(&mut data)?;

    let starts_with_solid = data
        .iter()
        .zip("solid".chars())
        .all(|(a, b)| a.to_ascii_lowercase() == b as u8);

    // we thus read a couple more bytes to check if they are ascii characters
    let mut data = vec![0u8; 128];
    reader.read_exact(&mut data)?;
    let following_bytes_are_ascii = data.iter().all(|d| d.is_ascii());

    if starts_with_solid && following_bytes_are_ascii {
        return Ok(StlType::Ascii);
    }

    // skip header
    reader.rewind()?;
    reader.seek(SeekFrom::Start(HEADER_SIZE))?;

    // check if it is a valid binary file by confirming that the
    // specified triangle count matches the size of the file
    // (the header does not include a magic number)
    let triangles = reader.read_u32::<LittleEndian>()? as u64;
    let filesize = reader.seek(SeekFrom::End(0))?;
    if triangles * TRIANGLE_SIZE + HEADER_SIZE + std::mem::size_of::<u32>() as u64 == filesize {
        return Ok(StlType::Binary);
    }

    // not a valid stl file
    bail!("not an STL file")
}

fn read_ascii_line<T: BufRead>(reader: &mut T) -> Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    Ok(line.trim_start().to_ascii_lowercase())
}

fn read_ascii_triangle<T: BufRead>(reader: &mut T) -> Result<Triangle> {
    let mut vertices = [
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 0.0),
    ];

    let (nx, ny, nz) = scan_fmt!(&read_ascii_line(reader)?, "facet normal {f} {f} {f}", f32, f32, f32)?;

    read_ascii_line(reader)?; // "outer loop"

    for v in &mut vertices {
        let (vx, vy, vz) = scan_fmt!(&read_ascii_line(reader)?, "vertex {f} {f} {f}", f32, f32, f32)?;
        v.x = vx;
        v.y = vy;
        v.z = vz;
    }

    read_ascii_line(reader)?; // "endloop"
    read_ascii_line(reader)?; // "endfacet"

    Ok(Triangle::new(vertices, Vec3::new(nx, ny, nz)))
}

fn read_vec3<T: io::Read>(reader: &mut T) -> Result<Vec3> {
    Ok(Vec3::new(
        reader.read_f32::<LittleEndian>()?,
        reader.read_f32::<LittleEndian>()?,
        reader.read_f32::<LittleEndian>()?,
    ))
}

fn read_triangle<T: io::Read>(reader: &mut T) -> Result<Triangle> {
    let n = read_vec3(reader)?;
    let v1 = read_vec3(reader)?;
    let v2 = read_vec3(reader)?;
    let v3 = read_vec3(reader)?;

    reader.read_u16::<LittleEndian>()?; // attributes

    Ok(Triangle::new([v1, v2, v3], n))
}

#[cfg(test)]
mod test {
    use crate::mesh::*;
    use crate::parser::Parser;
    use std::io::Cursor;

    const TRI_BIN: &[u8] = include_bytes!("test_models/triangle.stl");
    const TRI_ASCII: &[u8] = include_bytes!("test_models/triangle_ascii.stl");
    const TRI_ASCII_BROKEN: &[u8] = include_bytes!("test_models/triangle_ascii_broken.stl");

    #[test]
    fn parser_ascii_test() {
        let reader = Cursor::new(TRI_ASCII);
        let mut parser = Parser::from_buf(reader, false).unwrap();
        let triangles = parser.read_all().unwrap();

        assert_eq!(triangles[0].normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(triangles[0].vertices[0], Vec3::new(-1.0, -1.0, 0.0));
        assert_eq!(triangles[0].vertices[1], Vec3::new(1.0, -1.0, 0.0));
        assert_eq!(triangles[0].vertices[2], Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn parser_ascii_broken_test() {
        let reader = Cursor::new(TRI_ASCII_BROKEN);
        let mut parser = Parser::from_buf(reader, false).unwrap();
        let triangles = parser.read_all().unwrap();

        assert_eq!(triangles.len(), 0);
    }

    #[test]
    fn parser_bin_test() {
        let reader = Cursor::new(TRI_BIN);
        let mut parser = Parser::from_buf(reader, false).unwrap();
        let mesh = parser.read_all().unwrap();

        assert_eq!(mesh[0].normal, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(mesh[0].vertices[0], Vec3::new(-1.0, -1.0, 0.0));
        assert_eq!(mesh[0].vertices[1], Vec3::new(1.0, -1.0, 0.0));
        assert_eq!(mesh[0].vertices[2], Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn mesh_lazy_ascii() {
        let reader = Cursor::new(TRI_ASCII);
        let mut parser = Parser::from_buf(reader, false).unwrap();

        assert_eq!(parser.triangle_count().unwrap(), 2);

        let lazy_mesh = LazyMesh::new(&mut parser);

        let triangles = (&lazy_mesh).into_iter().collect::<Vec<Triangle>>();
        assert_eq!(
            triangles[0],
            Triangle {
                vertices: [
                    Vec3::new(-1.0, -1.0, 0.0),
                    Vec3::new(1.0, -1.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0)
                ],
                normal: Vec3::new(0.0, 0.0, 1.0),
            }
        );
        assert_eq!(
            triangles[1],
            Triangle {
                vertices: [
                    Vec3::new(-1.0, -1.0, 1.0),
                    Vec3::new(1.0, -1.0, 1.0),
                    Vec3::new(0.0, 1.0, 1.0)
                ],
                normal: Vec3::new(0.0, 0.0, 1.0),
            }
        );
    }

    #[test]
    fn mesh_lazy_bin() {
        let reader = Cursor::new(TRI_BIN);
        let mut parser = Parser::from_buf(reader, false).unwrap();

        assert_eq!(parser.triangle_count().unwrap(), 1);

        let lazy_mesh = LazyMesh::new(&mut parser);

        let triangles = (&lazy_mesh).into_iter().collect::<Vec<Triangle>>();
        assert_eq!(
            triangles[0],
            Triangle {
                vertices: [
                    Vec3::new(-1.0, -1.0, 0.0),
                    Vec3::new(1.0, -1.0, 0.0),
                    Vec3::new(0.0, 1.0, 0.0)
                ],
                normal: Vec3::new(0.0, 0.0, 1.0),
            }
        );
    }
}
