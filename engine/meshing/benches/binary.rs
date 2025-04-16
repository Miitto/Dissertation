use std::{cell::RefCell, hint::black_box};

use common::{
    Args, BlockType, seperate_global_pos,
    tests::{Scene, Test, test_scene},
};
use criterion::{Criterion, criterion_group, criterion_main};

use dashmap::{DashMap, iter::Iter};
use glam::IVec3;
use meshing::binary::{
    common::*,
    culled::{Chunk, chunk_data, mesh_chunks},
};

pub fn culled(c: &mut Criterion) {
    let empty_fn = |_x: isize, _y: isize, _z: isize| -> BlockType { BlockType::Air };
    c.bench_function("Empty", |b| {
        b.iter(|| {
            make_culled_faces(black_box(empty_fn));
        })
    });

    let full_fn = |_x: isize, _y: isize, _z: isize| -> BlockType { BlockType::Grass };
    c.bench_function("Full", |b| {
        b.iter(|| {
            make_culled_faces(black_box(full_fn));
        })
    });

    let mut args = Args::default();
    args.scene = Scene::Perlin;
    args.test = Test::Culled;

    let blocks = test_scene(&args);
    let chunks = DashMap::new();

    chunk_data(&blocks, &args, &chunks);

    let mut chunk_iter = chunks.iter();

    let mut next_item = || {
        if let Some(e) = chunk_iter.next() {
            e
        } else {
            chunk_iter = chunks.iter();
            chunk_iter.next().unwrap()
        }
    };

    c.bench_function("Get fn", |b| {
        let e = next_item();
        let pos = e.key();
        let chunk = e.value();
        let perlin_fn =
            |x: isize, y: isize, z: isize| -> BlockType { chunk.get_at(x, y, z, pos, &chunks) };
        b.iter(|| {
            for x in -1..=32 {
                for y in -1..=32 {
                    for z in -1..=32 {
                        let _ = perlin_fn(black_box(x), black_box(y), black_box(z));
                    }
                }
            }
        })
    });

    c.bench_function("Perlin", |b| {
        let e = next_item();
        let pos = e.key();
        let chunk = e.value();

        let perlin_fn =
            |x: isize, y: isize, z: isize| -> BlockType { chunk.get_at(x, y, z, pos, &chunks) };
        b.iter(|| {
            make_culled_faces(black_box(perlin_fn));
        })
    });

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
