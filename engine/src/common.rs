use renderer::Dir;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstanceData(u32);

impl InstanceData {
    pub fn new(x: u8, y: u8, z: u8, direction: Dir) -> Self {
        if x > 31 || y > 31 || z > 31 {
            panic!("Invalid position: ({}, {}, {})", x, y, z);
        }

        let x = x as u32;
        let y = y as u32;
        let z = z as u32;

        let d = usize::from(direction) as u32;

        Self((d << 15) | (x << 10) | (y << 5) | z)
    }

    pub fn as_int(&self) -> u32 {
        self.0
    }

    pub fn x(&self) -> u8 {
        ((self.0 >> 10) & 0b11111) as u8
    }

    pub fn y(&self) -> u8 {
        ((self.0 >> 5) & 0b11111) as u8
    }

    pub fn z(&self) -> u8 {
        (self.0 & 0b11111) as u8
    }

    pub fn dir(&self) -> Dir {
        Dir::from(((self.0 >> 15) & 0b111) as usize)
    }

    pub fn rotate_on_dir(&self) -> Self {
        let x = self.x();
        let y = self.y();
        let z = self.z();
        let dir = self.dir();

        match dir {
            Dir::Up | Dir::Down => Self::new(x, z, y, dir),
            Dir::Forward | Dir::Backward => Self::new(y, x, 31 - z, dir),
            Dir::Left | Dir::Right => Self::new(z, y, x, dir),
        }
    }
}

impl From<u32> for InstanceData {
    fn from(data: u32) -> Self {
        Self(data)
    }
}

impl From<InstanceData> for u32 {
    fn from(data: InstanceData) -> Self {
        data.0
    }
}
