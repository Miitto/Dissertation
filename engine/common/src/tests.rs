use dashmap::DashMap;
use rayon::prelude::*;

use bracket_noise::prelude::{FastNoise, FractalType, NoiseType};
use glam::{IVec3, ivec3};

use crate::{Args, BlockType};

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
#[allow(dead_code)]
pub enum Scene {
    Single,
    Cube,
    Perlin,
}

impl Scene {
    pub const fn all() -> [Scene; 3] {
        [Self::Single, Self::Cube, Self::Perlin]
    }
}

#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum Test {
    Tri,
    Basic,
    Instanced,
    Culled,
    Greedy,
    Raymarch,
    Flat,
    Svt64,
}

impl Test {
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Basic, Self::Instanced, Self::Culled, Self::Greedy]
            .iter()
            .copied()
    }
}

pub fn test_scene(args: &Args) -> DashMap<IVec3, BlockType> {
    println!("Creating test scene");
    let scene = match args.scene {
        Scene::Single => {
            let map = DashMap::new();
            map.insert(ivec3(0, 30, -5), BlockType::Grass);
            map
        }
        Scene::Cube => {
            let map = DashMap::new();
            (0..32).for_each(|x| {
                (0..32).for_each(|y| {
                    (32..64).for_each(|z| {
                        map.insert(ivec3(x, y, z), BlockType::Grass);
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

            let tuples: Vec<(i32, i32)> = (-radius..radius)
                .flat_map(|x| (-radius..radius).map(move |z| (x, z)))
                .collect();

            tuples
                .into_par_iter()
                .flat_map(|(x, z)| {
                    let noise = noise.get_noise(x as f32 / NOISE_SCALE, z as f32 / NOISE_SCALE);

                    let height = ((noise + 0.7) * input_height as f32).ceil() as i32;

                    (0..=height).into_par_iter().map(move |y| {
                        let block_type = if y > input_height - 3 {
                            BlockType::Snow
                        } else if y > input_height / 2 {
                            BlockType::Grass
                        } else {
                            BlockType::Stone
                        };

                        (ivec3(x, y, z), block_type)
                    })
                })
                .collect()
        }
    };

    println!("Finished generating scsene");
    scene
}
