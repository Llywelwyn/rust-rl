use criterion::{ black_box, criterion_group, criterion_main, Criterion };
use bracket_lib::prelude::*;

/// Benchmarks methods from rltk used to desaturate non-visible tiles.
// Greyscale is significantly faster, but generally looks worse - the
// third alternative is directly setting the desaturated value, if it
// is known in advance.
fn nonvisible_benchmark(c: &mut Criterion) {
    let bg = black_box(RGB::from_f32(0.4, 0.0, 0.0));

    c.bench_function("rgb -> greyscale", |b| b.iter(|| bg.to_greyscale()));
    c.bench_function("rgb -> desaturate", |b| b.iter(|| bg.desaturate()));
}

criterion_group!(benches, nonvisible_benchmark);
criterion_main!(benches);
