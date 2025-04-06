use std::hint::black_box;

use criterion::Criterion;

extern crate renderer;

use renderer::buffers::{Buffer, BufferMode, FencedBuffer, FencedRawBuffer, RawBuffer};

const SIZE: usize = 1_000_000;
static MID_CHUNK: usize = SIZE / 10;
static SMALL_CHUNK: usize = 10;
static DATA: [u8; SIZE] = [1; SIZE];

fn create_buffer(mode: BufferMode) -> FencedRawBuffer {
    FencedRawBuffer::empty(SIZE, mode).expect("Failed to make buffer")
}

pub fn write(c: &mut Criterion) {
    {
        let mut buffer = create_buffer(BufferMode::Dynamic);

        chunk_test("Dynamic", &mut buffer, c);
    }

    {
        let mut immutable = create_buffer(BufferMode::Immutable);

        chunk_test("Immutable", &mut immutable, c);
    }

    {
        let mut default = create_buffer(BufferMode::Default);

        chunk_test("Default", &mut default, c);
    }

    {
        let mut pers = create_buffer(BufferMode::Persistent);

        chunk_test("Persistent", &mut pers, c);
    }

    {
        let mut pers_coh = create_buffer(BufferMode::PersistentCoherent);

        chunk_test("Persistent Coherant", &mut pers_coh, c);
    }

    {
        let mut pers = create_buffer(BufferMode::Persistent);

        chunk_map_test("Persistent", &mut pers, c);
    }

    {
        let mut pers_coh = create_buffer(BufferMode::Persistent);

        chunk_map_test("Persistent Coherant", &mut pers_coh, c);
    }
}

fn chunk_test(name: &str, buffer: &mut FencedRawBuffer, c: &mut Criterion) {
    c.bench_function(format!("{name} All").as_str(), |b| {
        b.iter(|| {
            _ = buffer.set_data_no_alloc(black_box(&DATA));
            while !buffer.signalled() {}
        })
    });

    let mut chunked = |chunk_size: usize| {
        for offset in (0..SIZE).step_by(chunk_size) {
            let segment = &DATA[offset..(offset + chunk_size)];

            _ = buffer.set_offset_data_no_alloc(black_box(offset), black_box(segment));
        }
        while !buffer.signalled() {}
    };

    c.bench_function(format!("{name} Mid Chunk").as_str(), |b| {
        b.iter(|| chunked(MID_CHUNK))
    });

    c.bench_function(format!("{name} Small Chunk").as_str(), |b| {
        b.iter(|| chunked(SMALL_CHUNK))
    });
}

fn chunk_map_test(name: &str, buffer: &mut FencedRawBuffer, c: &mut Criterion) {
    let mut chunked = |chunk_size: usize| {
        {
            let mut mapping = buffer.get_mapping();
            for offset in (0..SIZE).step_by(chunk_size) {
                let segment = &DATA[offset..(offset + chunk_size)];

                unsafe {
                    mapping.write(
                        black_box(segment.as_ptr().add(offset)),
                        black_box(chunk_size),
                        black_box(offset),
                    );
                }
            }
        }
        while !buffer.signalled() {}
    };

    c.bench_function(format!("{name} Mapping Mid Chunk").as_str(), |b| {
        b.iter(|| chunked(MID_CHUNK))
    });

    c.bench_function(format!("{name} Mapping Small Chunk").as_str(), |b| {
        b.iter(|| chunked(SMALL_CHUNK))
    });
}
