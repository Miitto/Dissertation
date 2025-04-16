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
    let blank_voxels = [[[BasicVoxel::new(BlockType::Air); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
    let refs = ChunkRefs {
        chunk: &blank_voxels,
        x_neg: &blank_voxels,
        x_pos: &blank_voxels,
        y_neg: &blank_voxels,
        y_pos: &blank_voxels,
        z_neg: &blank_voxels,
        z_pos: &blank_voxels,
    };
    c.bench_function("Culled Empty", |b| {
        b.iter(|| {
            make_culled_faces(black_box(&refs));
        })
    });
    c.bench_function("Greedy Empty", |b| {
        b.iter(|| {
            make_greedy_faces(black_box(&refs));
        })
    });

    let full_voxels = [[[BasicVoxel::new(BlockType::Grass); CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];
    let refs = ChunkRefs {
        chunk: &full_voxels,
        x_neg: &full_voxels,
        x_pos: &full_voxels,
        y_neg: &full_voxels,
        y_pos: &full_voxels,
        z_neg: &full_voxels,
        z_pos: &full_voxels,
    };
    c.bench_function("Culled Full", |b| {
        b.iter(|| {
            make_culled_faces(black_box(&refs));
        })
    });
    c.bench_function("Greedy Full", |b| {
        b.iter(|| {
            make_greedy_faces(black_box(&refs));
        })
    });

    let mut args = Args::default();
    args.scene = Scene::Perlin;
    args.test = Test::Culled;
    args.radius = CHUNK_SIZE as i32 * 2;
    let blocks = test_scene(&args);
    let chunks = DashMap::new();

    chunk_data(&blocks, &args, &chunks);
    c.bench_function("Culled Perlin", |b| {
        let pos = ivec3(0, 0, 0);
        b.iter(|| {
            black_box(make_faces(
                black_box(&chunks),
                black_box(&pos),
                black_box(false),
            ))
        });
    });
    c.bench_function("Greedy Perlin", |b| {
        let pos = ivec3(0, 0, 0);
        b.iter(|| {
            black_box(make_faces(
                black_box(&chunks),
                black_box(&pos),
                black_box(true),
            ))
        });
    });

    macro_rules! serial_bench {
        ($radius:literal) => {{
            let chunks = DashMap::new();
            args.scene = Scene::Perlin;
            args.test = Test::Culled;
            args.radius = $radius * CHUNK_SIZE as i32;
            let blocks = test_scene(&args);
            chunk_data(&blocks, &args, &chunks);

            c.bench_function(stringify!(Culled $radius), |b| {
                b.iter(|| {
                    chunks.iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()), false);
                    });
                });
            });

            c.bench_function(stringify!(Greedy $radius), |b| {
                b.iter(|| {
                    chunks.iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()), true);
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
            let chunks = DashMap::new();
            args.scene = Scene::Perlin;
            args.test = Test::Culled;
            args.radius = $radius * CHUNK_SIZE as i32;
            let blocks = test_scene(&args);
            chunk_data(&blocks, &args, &chunks);

            c.bench_function(stringify!(Parallel Culled $radius), |b| {
                b.iter(|| {
                    chunks.par_iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()), false);
                    });
                });
            });

            c.bench_function(stringify!(Parallel Greedy $radius), |b| {
                b.iter(|| {
                    chunks.par_iter().for_each(|e| {
                        make_faces(&chunks, black_box(e.key()), true);
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
