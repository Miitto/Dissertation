#[derive(Clone, Copy, Debug)]
pub enum BlockType {
    Air,
    Grass,
    // etc.
}

pub struct Pos(u16);

impl Pos {
    pub fn new(x: u8, y: u8, z: u8) -> Self {
        if x > 31 || y > 31 || z > 31 {
            panic!("Invalid position: ({}, {}, {})", x, y, z);
        }

        let x = x as u16;
        let y = y as u16;
        let z = z as u16;

        Self((x << 10) | (y << 5) | z)
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
}

pub struct Voxel {
    position: Pos,
}

shaders::program!(ChunkVoxel, 330, {
    #vertex vert
    #fragment frag

    uniform mat4 projectionMatrix;
    uniform mat4 viewMatrix;
    uniform mat4 modelMatrix;

    struct vIn {
        uint position;
    }

    struct v2f {
        vec4 color;
    }

    v2f vert(vIn input) {
        v2f o;
        o.color = vec4(1.0, 0.0, 0.0, 1.0);

        mat4 vp = projectionMatrix * viewMatrix;
        mat4 mvp = vp * modelMatrix;

        uint x = input.position >> 10;
        uint y = (input.position >> 5) & 0b11111;
        uint z = input.position & 0b11111;

        gl_Position = mvp * vec4(0.0, 0.0, 0.0, 1.0);
    }

    vec4 frag(v2f input) {
        return input.color;
    }
});
