use renderer::Dir;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InstanceData(u32);

pub trait Voxel {
    fn get_type(&self) -> BlockType;
    fn set_type(&mut self, block_type: BlockType);
}

#[derive(Debug, Clone, Copy)]
pub struct BasicVoxel {
    block_type: BlockType,
}

impl BasicVoxel {
    pub fn new(block_type: BlockType) -> Self {
        Self { block_type }
    }
}

impl Voxel for BasicVoxel {
    fn get_type(&self) -> BlockType {
        self.block_type
    }

    fn set_type(&mut self, block_type: BlockType) {
        self.block_type = block_type;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default, Hash, Eq)]
pub enum BlockType {
    #[default]
    Air,
    Grass,
    Stone,
    Snow, // etc.
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        *self != BlockType::Air
    }
}

impl From<BlockType> for u32 {
    fn from(value: BlockType) -> u32 {
        match value {
            BlockType::Air => 0,
            BlockType::Grass => 1,
            BlockType::Stone => 2,
            BlockType::Snow => 3,
        }
    }
}

impl TryFrom<u32> for BlockType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(BlockType::Air),
            1 => Ok(BlockType::Grass),
            2 => Ok(BlockType::Stone),
            3 => Ok(BlockType::Snow),
            _ => Err(()),
        }
    }
}

impl InstanceData {
    pub fn new(
        x: u8,
        y: u8,
        z: u8,
        direction: Dir,
        width: u8,
        height: u8,
        block_type: BlockType,
    ) -> Self {
        if x > 31 || y > 31 || z > 31 {
            panic!("Invalid position: ({}, {}, {})", x, y, z);
        }

        if width >= 32 || height >= 32 {
            panic!("Invalid width or height: ({}, {})", width, height);
        }

        let x = x as u32;
        let y = y as u32;
        let z = z as u32;

        let d = usize::from(direction) as u32;

        let width = width as u32;
        let height = height as u32;

        let x_mask = (x & 0b11111) << 10;
        let y_mask = (y & 0b11111) << 5;
        let z_mask = z & 0b11111;

        let d_mask = (d & 0b111) << 15;

        let w_mask = (width & 0b11111) << 18;
        let h_mask = (height & 0b11111) << 23;

        let block_type: u32 = block_type.into();

        let block_type_mask = block_type << 28;

        Self(x_mask | y_mask | z_mask | d_mask | w_mask | h_mask | block_type_mask)
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

    pub fn width(&self) -> u8 {
        ((self.0 >> 18) & 0b11111) as u8
    }

    pub fn height(&self) -> u8 {
        ((self.0 >> 23) & 0b11111) as u8
    }

    pub fn block_type(&self) -> BlockType {
        (self.0 >> 28).try_into().unwrap()
    }

    pub fn rotate_on_dir(&self) -> Self {
        let x = self.x();
        let y = self.y();
        let z = self.z();
        let dir = self.dir();
        let width = self.width();
        let height = self.height();
        let block_type = self.block_type();

        match dir {
            Dir::Up | Dir::Down => Self::new(x, z, y, dir, width, height, block_type),
            Dir::Forward | Dir::Backward => Self::new(x, y, z, dir, width, height, block_type),
            Dir::Left | Dir::Right => Self::new(z, y, x, dir, width, height, block_type),
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

impl std::fmt::Display for InstanceData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "InstanceData {{ x: {}, y: {}, z: {}, dir: {:?}, width: {}, height: {} }}\n{:032b}",
            self.x(),
            self.y(),
            self.z(),
            self.dir(),
            self.width() + 1,
            self.height() + 1,
            self.0
        )
    }
}
