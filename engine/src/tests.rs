use std::collections::HashMap;

use bracket_noise::prelude::{FastNoise, FractalType, NoiseType};
use glam::{IVec3, ivec3};

use crate::{Args, common::BlockType};

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
#[allow(dead_code)]
pub enum Scene {
    Single,
    Cube,
    Plane,
    Perlin,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Test {
    Tri,
    Basic,
    BasicInstanced,
    Chunk,
    Culled,
    Greedy,
    Raymarch,
    Svt64,
}

pub fn test_scene(args: &Args) -> HashMap<IVec3, BlockType> {
    match args.scene {
        Scene::Single => {
            let mut map = HashMap::new();
            map.insert(ivec3(0, 0, 0), BlockType::Grass);
            map
        }
        Scene::Cube => {
            let mut map = HashMap::new();
            (0..32).for_each(|x| {
                (0..32).for_each(|y| {
                    (0..32).for_each(|z| {
                        map.insert(ivec3(x, y, z), BlockType::Grass);
                    })
                })
            });

            map
        }
        Scene::Plane => {
            let radius = args.radius;
            let height = args.depth;

            let mut map = HashMap::new();

            (-radius..radius).for_each(|x| {
                (0..height).for_each(|y| {
                    (-radius..radius).for_each(|z| {
                        let block = if y == height - 1 {
                            BlockType::Grass
                        } else {
                            BlockType::Stone
                        };
                        map.insert(ivec3(x, y, z), block);
                    })
                })
            });

            map
        }
        Scene::Perlin => {
            let mut noise = FastNoise::seeded(1234);
            noise.set_noise_type(NoiseType::PerlinFractal);
            noise.set_fractal_type(FractalType::FBM);
            noise.set_fractal_octaves(5);
            noise.set_fractal_gain(0.5);
            noise.set_fractal_lacunarity(2.0);
            noise.set_frequency(2.0);

            const NOISE_SCALE: f32 = 160.0;

            let radius = args.radius;
            let input_height = args.depth;

            let mut blocks = HashMap::new();

            for x in -radius..radius {
                for z in -radius..radius {
                    let noise = noise.get_noise(x as f32 / NOISE_SCALE, z as f32 / NOISE_SCALE);

                    let height = ((noise + 0.7) * input_height as f32).ceil() as i32;

                    for y in 0..=height {
                        let block_type = if y > input_height - 3 {
                            BlockType::Snow
                        } else if y > input_height / 2 {
                            BlockType::Grass
                        } else {
                            BlockType::Stone
                        };

                        blocks.insert(ivec3(x, y, z), block_type);
                    }
                }
            }

            blocks
        }
    }
}
