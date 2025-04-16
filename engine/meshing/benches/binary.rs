use std::hint::black_box;

use common::{
    Args, BlockType,
    tests::{Scene, Test, test_scene},
};
use criterion::{Criterion, criterion_group, criterion_main};

use dashmap::DashMap;
use glam::{IVec3, ivec3};
use meshing::binary::{
    common::*,
    culled::{Chunk, chunk_data},
};

pub fn culled(c: &mut Criterion) {
    let chunks = (-1..=1)
        .flat_map(|x| (-1..=1).flat_map(move |y| (-1..=1).map(move |z| ivec3(x, y, z))))
        .map(|pos| (pos, Chunk::fill(BlockType::Air, false, false, false)))
        .collect::<DashMap<_, _>>();
    let pos = IVec3::new(0, 0, 0);
    let chunk = chunks.get(&pos).unwrap();

    let empty_fn =
        |x: isize, y: isize, z: isize| -> BlockType { chunk.get_at(x, y, z, &pos, &chunks) };

    c.bench_function("Culled Empty", |b| {
        b.iter(|| {
            make_culled_faces(black_box(empty_fn));
        })
    });
    c.bench_function("Greedy Empty", |b| {
        b.iter(|| {
            make_greedy_faces(black_box(empty_fn));
        })
    });

    let chunks = (-1..=1)
        .flat_map(|x| (-1..=1).flat_map(move |y| (-1..=1).map(move |z| ivec3(x, y, z))))
        .map(|pos| (pos, Chunk::fill(BlockType::Grass, false, false, false)))
        .collect::<DashMap<_, _>>();
    let pos = IVec3::new(0, 0, 0);
    let chunk = chunks.get(&pos).unwrap();

    let full_fn =
        |x: isize, y: isize, z: isize| -> BlockType { chunk.get_at(x, y, z, &pos, &chunks) };
    c.bench_function("Culled Full", |b| {
        b.iter(|| {
            make_culled_faces(black_box(full_fn));
        })
    });
    c.bench_function("Greedy Full", |b| {
        b.iter(|| {
            make_greedy_faces(black_box(full_fn));
        })
    });

    let mut args = Args::default();
    args.scene = Scene::Perlin;
    args.test = Test::Culled;

    let blocks = test_scene(&args);
    let chunks = DashMap::new();

    chunk_data(&blocks, &args, &chunks);
    let pos = IVec3::new(0, 0, 0);
    let chunk = chunks.get(&pos).unwrap();
    let perlin_fn =
        |x: isize, y: isize, z: isize| -> BlockType { chunk.get_at(x, y, z, &pos, &chunks) };

    c.bench_function("Culled Perlin", |b| {
        b.iter(|| {
            make_culled_faces(black_box(perlin_fn));
        })
    });
    c.bench_function("Greedy Perlin", |b| {
        b.iter(|| {
            make_greedy_faces(black_box(perlin_fn));
        })
    });
}

criterion_group!(benches, culled);
criterion_main!(benches);
