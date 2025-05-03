pub mod directions;
pub mod tests;

use directions::Dir;

pub use clap::Parser;
use tests::{Scene, Test};
#[derive(clap::Parser, Debug, Clone, Copy)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Scene to use
    #[arg(short, long, default_value = "perlin")]
    pub scene: Scene,

    /// Test type
    #[arg(short, long, default_value = "culled")]
    pub test: Test,

    /// Radius
    #[arg(short, long, default_value = "32")]
    pub radius: i32,

    /// Height
    #[arg(short, long, default_value = "20")]
    pub depth: i32,

    /// Frustum Culling
    #[arg(short, long, default_value = "false")]
    pub frustum_cull: bool,

    /// Combine Draw calls
    #[arg(short, long, default_value = "false")]
    pub combine: bool,

    /// Use Vertex Pulling
    #[arg(short, long, default_value = "false")]
    pub vertex_pull: bool,

    /// Profile
    #[arg(short, long, default_value = "false")]
    pub profile: bool,

    /// Auto test
    #[arg(short, long, default_value = "false")]
    pub auto_test: bool,
}

impl Args {
    pub const fn default() -> Self {
        Self {
            scene: Scene::Single,
            test: Test::Culled,
            radius: 32,
            depth: 20,
            frustum_cull: false,
            combine: false,
            vertex_pull: false,
            profile: false,
            auto_test: false,
        }
    }
}

impl std::fmt::Display for Args {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let radius = if self.scene == Scene::Perlin {
            format!(" {}", self.radius)
        } else {
            String::new()
        };
        let mut flags = String::new();
        if self.frustum_cull || self.combine || self.vertex_pull {
            flags.push(' ');
        }
        if self.frustum_cull {
            flags.push('F');
        }
        if self.vertex_pull {
            flags.push('V');
        }
        if self.combine {
            flags.push('C');
        }
        write!(f, "{:?}{}, {:?}{}", self.scene, radius, self.test, flags)
    }
}

use glam::IVec3;

pub fn seperate_global_pos(pos: &IVec3) -> (IVec3, IVec3) {
    const CHUNK_SIZE: i32 = 30;
    let mut chunk_pos = pos / CHUNK_SIZE;
    let mut in_chunk_pos = pos.abs() % CHUNK_SIZE;

    if pos.x < 0 {
        chunk_pos.x -= 1;
        in_chunk_pos.x = (CHUNK_SIZE - 1) - in_chunk_pos.x;
    }
    if pos.y < 0 {
        chunk_pos.y -= 1;
        in_chunk_pos.y = (CHUNK_SIZE - 1) - in_chunk_pos.y;
    }
    if pos.z < 0 {
        chunk_pos.z -= 1;
        in_chunk_pos.z = (CHUNK_SIZE - 1) - in_chunk_pos.z;
    }

    (chunk_pos, in_chunk_pos)
}

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
    pub const fn new(block_type: BlockType) -> Self {
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
    Invalid,
    Air,
    Grass,
    Stone,
    Snow, // etc.
}

impl BlockType {
    pub fn is_solid(&self) -> bool {
        ![BlockType::Air, BlockType::Invalid].contains(self)
    }
}

impl From<BlockType> for u32 {
    fn from(value: BlockType) -> u32 {
        match value {
            BlockType::Invalid => u32::MAX,
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
            u32::MAX => Ok(BlockType::Invalid),
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
        const CHUNK_SIZE: u8 = 30;
        if x >= CHUNK_SIZE || y >= CHUNK_SIZE || z >= CHUNK_SIZE {
            panic!("Invalid position: ({}, {}, {})", x, y, z);
        }

        if width >= CHUNK_SIZE || height >= CHUNK_SIZE {
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
