use criterion::{criterion_group, criterion_main, Criterion};
use runner::{build_complex_filter, interpreted, jit::build_module, read_data};

fn criterion_benchmark(c: &mut Criterion) {
    let users = read_data();
    let filters = build_complex_filter();
    let jit_fn = unsafe { build_module(&filters) };

    c.bench_function("Interpreted", |b| {
        b.iter(|| interpreted::filter_vec_with_filters(&users, &filters))
    });

    c.bench_function("JIT", |b| b.iter(|| unsafe { jit_fn.execute(&users) }));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
