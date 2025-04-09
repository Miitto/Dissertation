use std::{cell::RefCell, hint::black_box};

use common::{
    Args, BlockType, seperate_global_pos,
    tests::{Scene, test_scene},
};
use criterion::{Criterion, criterion_group, criterion_main};

use meshing::binary::{common::*, culled::Chunk};

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

    let args = Args::default();

    let blocks = test_scene(&args);
    let mut chunks = std::collections::HashMap::new();

    blocks.iter().for_each(|r| {
        let pos = r.key();
        let block = r.value();
        let (chunk_pos, in_chunk_pos) = seperate_global_pos(pos);

        let chunk = chunks.entry(chunk_pos).or_insert(RefCell::new(Chunk::fill(
            BlockType::Air,
            false,
            args.frustum_cull,
        )));

        chunk.borrow_mut().set(in_chunk_pos, *block);
    });

    let mut chunk_iter = chunks.iter();
    let (mut pos, mut chunk) = chunk_iter.next().unwrap();

    let perlin_fn = |x: isize, y: isize, z: isize| -> BlockType {
        chunk.borrow().get_at(x, y, z, pos, &chunks)
    };

    c.bench_function("Get fn", |b| {
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
        b.iter(|| {
            make_culled_faces(black_box(perlin_fn));
            if let Some(next) = chunk_iter.next() {
                pos = next.0;
                chunk = next.1;
            } else {
                chunk_iter = chunks.iter();
                let (next_pos, next_chunk) = chunk_iter.next().unwrap();
                pos = next_pos;
                chunk = next_chunk;
            }
        })
    });
}

criterion_group!(benches, culled);
criterion_main!(benches);
