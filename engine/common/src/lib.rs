pub mod tests;

pub use clap::Parser;
use tests::{Scene, Test};
#[derive(clap::Parser, Debug, Clone, Copy)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Scene to use
    #[arg(short, long, default_value = "single")]
    pub scene: Scene,

    /// Test type
    #[arg(short, long, default_value = "basic")]
    pub test: Test,

    /// Radius
    #[arg(short, long, default_value = "8")]
    pub radius: i32,

    /// Height
    #[arg(short, long, default_value = "20")]
    pub depth: i32,

    /// Frustum Culling
    #[arg(short, long, default_value = "false")]
    pub frustum_cull: bool,

    /// Combine Draw calls using SSBO
    #[arg(short, long, default_value = "false")]
    pub combine: bool,

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
            test: Test::Basic,
            radius: 8,
            depth: 20,
            frustum_cull: false,
            combine: false,
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
        if self.frustum_cull || self.combine {
            flags.push(' ');
        }
        if self.frustum_cull {
            flags.push_str("F");
        }
        if self.combine {
            flags.push_str("C");
        }
        write!(f, "{:?}{}, {:?}{}", self.scene, radius, self.test, flags)
    }
}

use glam::{IVec3, Vec3, ivec3, vec3};
use renderer::{Dir, camera::Camera};

pub fn seperate_global_pos(pos: &IVec3) -> (IVec3, IVec3) {
    let mut chunk_pos = pos / 32;
    let mut in_chunk_pos = pos.abs() % 32;

    if pos.x < 0 {
        chunk_pos.x -= 1;
        in_chunk_pos.x = 31 - in_chunk_pos.x;
    }
    if pos.y < 0 {
        chunk_pos.y -= 1;
        in_chunk_pos.y = 31 - in_chunk_pos.y;
    }
    if pos.z < 0 {
        chunk_pos.z -= 1;
        in_chunk_pos.z = 31 - in_chunk_pos.z;
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

pub fn get_looked_at_block(
    camera: &dyn Camera,
    get_block_fn: impl Fn(&IVec3) -> BlockType,
) -> Option<IVec3> {
    return None;
    renderer::profiler::event!("Greedy Get Looked At Block");

    //http://www.cse.yorku.ca/~amana/research/grid.pdf
    let forward = camera.forward();

    let mut current = camera.transform().position;
    let end = current + (forward * 6.0);

    let delta = vec3(
        (end.x - current.x).abs(),
        (end.y - current.y).abs(),
        (end.z - current.z).abs(),
    );

    let step = vec3(forward.x.signum(), forward.y.signum(), forward.z.signum());

    let hypotenuse = delta.length();
    let hypotenuse_half = hypotenuse / 2.0;

    let mut t_max = vec3(
        hypotenuse_half / delta.x,
        hypotenuse_half / delta.y,
        hypotenuse_half / delta.z,
    );

    let t_delta = vec3(
        hypotenuse / delta.x,
        hypotenuse / delta.y,
        hypotenuse / delta.z,
    );

    macro_rules! inc_x {
        () => {
            t_max.x += t_delta.x;
            current.x += step.x;
        };
    }

    macro_rules! inc_y {
        () => {
            t_max.y += t_delta.y;
            current.y += step.y;
        };
    }

    macro_rules! inc_z {
        () => {
            t_max.z += t_delta.z;
            current.z += step.z;
        };
    }

    let compare = |current: &Vec3, end: &Vec3| {
        let x = if step.x < 0.0 {
            current.x >= end.x
        } else {
            current.x <= end.x
        };

        let y = if step.y < 0.0 {
            current.y >= end.y
        } else {
            current.y <= end.y
        };

        let z = if step.z < 0.0 {
            current.z >= end.z
        } else {
            current.z <= end.z
        };

        x && y && z
    };

    while compare(&current, &end) {
        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                inc_x!();
            } else if t_max.x > t_max.z {
                inc_z!();
            } else {
                inc_x!();
                inc_z!();
            }
        } else if t_max.x > t_max.y {
            if t_max.y < t_max.z {
                inc_y!();
            } else if t_max.y > t_max.z {
                inc_z!();
            } else {
                inc_y!();
                inc_z!();
            }
        } else if t_max.y < t_max.z {
            inc_x!();
            inc_y!();
        } else if t_max.y > t_max.z {
            inc_z!();
        } else {
            inc_x!();
            inc_y!();
            inc_z!();
        }

        let pos = ivec3(
            current.x.floor() as i32,
            current.y.floor() as i32,
            current.z.floor() as i32,
        );

        if get_block_fn(&pos).is_solid() {
            return Some(pos);
        }
    }

    None
}
