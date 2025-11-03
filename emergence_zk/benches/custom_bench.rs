use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use rayon::prelude::*;

const FIB_MAX: u64 = 300;

use std::f64::consts::PI;

// Simulates a CPU-intensive computation with controllable complexity
fn expensive_computation(n: u64) -> f64 {
    let mut result = n as f64;

    // Perform many floating point operations
    for i in 1..1000 {
        result = (result * PI).sin().abs();
        result += (i as f64).sqrt();
        result = result.ln_1p().exp();
    }

    // Add some branching
    if result > 100.0 {
        result = result.sqrt();
    }

    result
}

fn loop_over_expensive(x: u64) -> Vec<f64> {
    let vec = (1..x).collect::<Vec<u64>>();
    vec.iter().map(|e| expensive_computation(*e)).collect()
}

fn loop_over_expensive_rayon(x: u64) -> Vec<f64> {
    let vec = (1..x).collect::<Vec<u64>>();
    vec.par_iter().map(|e| expensive_computation(*e)).collect()
}
fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("sequential", |b| {
        b.iter(|| loop_over_expensive(black_box(FIB_MAX)))
    });
    c.bench_function("rayon", |b| {
        b.iter(|| loop_over_expensive_rayon(black_box(FIB_MAX)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
