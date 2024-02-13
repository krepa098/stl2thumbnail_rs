pub struct ZBuffer {
    data: Vec<f32>,
    width: u32,
    height: u32,
}

impl ZBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        let mut data = Vec::new();
        data.resize((width * height) as usize, f32::MIN);

        Self { data, width, height }
    }

    pub fn test_and_set(&mut self, x: u32, y: u32, z: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }

        if z > self.data[(y * self.width + x) as usize] {
            self.data[(y * self.width + x) as usize] = z;
            return true;
        }

        false
    }
}
