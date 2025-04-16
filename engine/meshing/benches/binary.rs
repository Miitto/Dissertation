use std::hint::black_box;

use common::{
    Args, BasicVoxel, BlockType,
    tests::{Scene, Test, test_scene},
};
use criterion::{Criterion, criterion_group, criterion_main};

use dashmap::DashMap;
use meshing::binary::{
    common::*,
    culled::{chunk_data, mesh_chunks},
};

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
    c.bench_function("Empty", |b| {
        b.iter(|| {
            make_culled_faces(black_box(&refs));
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
    c.bench_function("Full", |b| {
        b.iter(|| {
            make_culled_faces(black_box(&refs));
        })
    });

    let mut args = Args::default();
    args.scene = Scene::Perlin;
    args.test = Test::Culled;

    let blocks = test_scene(&args);

    c.bench_function("Blocks to Chunks", |b| {
        b.iter(|| {
            let chunks = DashMap::new();
            chunk_data(&blocks, &args, &chunks);
        });
    });

    macro_rules! culled_bench {
        ($radius:literal) => {
            let chunks = DashMap::new();
            args.scene = Scene::Perlin;
            args.test = Test::Culled;
            args.radius = $radius;
            chunk_data(&blocks, &args, &chunks);

            c.bench_function(stringify!(Parallel Culled $radius), |b| {
                b.iter(|| {
                    for chunk in chunks.iter() {
                        let chunk = chunk.value();
                        chunk.invalidate();
                    }
                    mesh_chunks(&chunks);
                });
            });
        };
    }

    culled_bench!(32);
    culled_bench!(64);
    culled_bench!(128);
    culled_bench!(256);
    culled_bench!(512);
    culled_bench!(1024);
}

criterion_group!(benches, culled);
criterion_main!(benches);
