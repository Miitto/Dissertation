use common::BlockType;
use criterion::{Criterion, criterion_group, criterion_main};

use meshing::binary::common::*;

pub fn culled(c: &mut Criterion) {
    let empty_fn = |_x: isize, _y: isize, _z: isize| -> BlockType { BlockType::Air };
    c.bench_function("Empty", |b| {
        b.iter(|| {
            make_culled_faces(empty_fn);
        })
    });

    let full_fn = |_x: isize, _y: isize, _z: isize| -> BlockType { BlockType::Grass };
    c.bench_function("Full", |b| {
        b.iter(|| {
            make_culled_faces(full_fn);
        })
    });
}

criterion_group!(benches, culled);
criterion_main!(benches);
