use rayon::prelude::*;
use std::hint::black_box;

use common::{
    Args, BasicVoxel, BlockType,
    tests::{Scene, Test, test_scene},
};
use criterion::{Criterion, criterion_group, criterion_main};

use dashmap::DashMap;
use glam::ivec3;
use meshing::binary::{common::*, culled::chunk_data};

pub fn culled(c: &mut Criterion) {
    {
        let blank_voxels =
            [[[BasicVoxel::new(BlockType::Air); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let refs = ChunkRefs {
            chunk: &blank_voxels,
            x_neg: &blank_voxels,
            x_pos: &blank_voxels,
            y_neg: &blank_voxels,
            y_pos: &blank_voxels,
            z_neg: &blank_voxels,
            z_pos: &blank_voxels,
        };
        let depths = build_depths(&refs);

        c.bench_function("Single Culled Empty", |b| {
            b.iter(|| {
                make_culled_faces(black_box(&refs), black_box(&depths));
            })
        });
        c.bench_function("Single Greedy Empty", |b| {
            b.iter(|| {
                make_greedy_faces(black_box(&refs), black_box(&depths));
            })
        });
    }
    {
        let full_voxels =
            [[[BasicVoxel::new(BlockType::Grass); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
        let refs = ChunkRefs {
            chunk: &full_voxels,
            x_neg: &full_voxels,
            x_pos: &full_voxels,
            y_neg: &full_voxels,
            y_pos: &full_voxels,
            z_neg: &full_voxels,
            z_pos: &full_voxels,
        };
        let depths = build_depths(&refs);
        c.bench_function("Single Culled Full", |b| {
            b.iter(|| {
                make_culled_faces(black_box(&refs), black_box(&depths));
            })
        });
        c.bench_function("Single Greedy Full", |b| {
            b.iter(|| {
                make_greedy_faces(black_box(&refs), black_box(&depths));
            })
        });
    }
    {
        let mut args = Args::default();
        args.scene = Scene::Perlin;
        args.test = Test::Culled;
        args.radius = CHUNK_SIZE as i32 * 2;
        let blocks = test_scene(&args);
        let chunks = DashMap::new();

        chunk_data(&blocks, &args, &chunks);
        let chunk = chunks.get(&ivec3(0, 0, 0)).unwrap();
        chunk.voxels().build_depths(&chunks, &ivec3(0, 0, 0));
        let voxels = chunk.voxels();
        let mask = voxels.depth_mask.read().unwrap();
        let depths = mask.as_ref().unwrap();

        c.bench_function("Single Culled Perlin", |b| {
            let pos = ivec3(0, 0, 0);
            b.iter(|| {
                black_box(make_faces(
                    black_box(&chunks),
                    black_box(&pos),
                    black_box(depths),
                    black_box(false),
                ))
            });
        });
        c.bench_function("Single Greedy Perlin", |b| {
            let pos = ivec3(0, 0, 0);
            b.iter(|| {
                black_box(make_faces(
                    black_box(&chunks),
                    black_box(&pos),
                    black_box(depths),
                    black_box(true),
                ))
            });
        });
    }

    macro_rules! serial_bench {
        ($radius:literal) => {{
            let mut args = Args::default();
            let chunks = DashMap::new();
            args.scene = Scene::Perlin;
            args.test = Test::Culled;
            args.radius = $radius * CHUNK_SIZE as i32;
            let blocks = test_scene(&args);
            chunk_data(&blocks, &args, &chunks);let chunk = chunks.get(&ivec3(0, 0, 0)).unwrap();
            chunk
                .voxels()
                .build_depths(&chunks, &ivec3(0, 0, 0));
            let voxels = chunk.voxels();
            let mask = voxels.depth_mask.read().unwrap();
            let depths = mask.as_ref().unwrap();

            c.bench_function(stringify!(Culled $radius), |b| {
                b.iter(|| {
                    chunks.iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()),black_box(depths), false);
                    });
                });
            });

            c.bench_function(stringify!(Greedy $radius), |b| {
                b.iter(|| {
                    chunks.iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()),black_box(depths), true);
                    });
                });
            });
        }};
    }

    serial_bench!(1);
    serial_bench!(2);
    serial_bench!(4);
    serial_bench!(8);

    macro_rules! parallel_bench {
        ($radius:literal) => {{
            let mut args = Args::default();
            let chunks = DashMap::new();
            args.scene = Scene::Perlin;
            args.test = Test::Culled;
            args.radius = $radius * CHUNK_SIZE as i32;
            let blocks = test_scene(&args);
            chunk_data(&blocks, &args, &chunks);let chunk = chunks.get(&ivec3(0, 0, 0)).unwrap();
            chunk
                .voxels()
                .build_depths(&chunks, &ivec3(0, 0, 0));
            let voxels = chunk.voxels();
            let mask = voxels.depth_mask.read().unwrap();
            let depths = mask.as_ref().unwrap();

            c.bench_function(stringify!(Parallel Culled $radius), |b| {
                b.iter(|| {
                    chunks.par_iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()), black_box(depths), false);
                    });
                });
            });

            c.bench_function(stringify!(Parallel Greedy $radius), |b| {
                b.iter(|| {
                    chunks.par_iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()),black_box(depths), true);
                    });
                });
            });
        }};
    }

    parallel_bench!(1);
    parallel_bench!(2);
    parallel_bench!(4);
    parallel_bench!(8);
}

criterion_group!(culled_bench, culled);
criterion_main!(culled_bench);
