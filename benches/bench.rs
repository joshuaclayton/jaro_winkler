use criterion::{black_box, criterion_group, criterion_main, Criterion};

macro_rules! run_bench {
    ($name:expr, $val:expr, $c:expr) => {
        $c.bench_function(&format!("JW {}", $name), |b| {
            b.iter(|| jaro_winkler::jaro_winkler(black_box($val.0), black_box($val.1)))
        });

        $c.bench_function(&format!("JW {} flipped", $name), |b| {
            b.iter(|| jaro_winkler::jaro_winkler(black_box($val.1), black_box($val.0)))
        });

        $c.bench_function(&format!("strsim {}", $name), |b| {
            b.iter(|| strsim::jaro_winkler(black_box($val.0), black_box($val.1)))
        });

        $c.bench_function(&format!("strsim {} flipped", $name), |b| {
            b.iter(|| strsim::jaro_winkler(black_box($val.1), black_box($val.0)))
        });

        $c.bench_function(&format!("eddie {}", $name), |b| {
            b.iter(|| eddie::JaroWinkler::new().similarity(black_box($val.0), black_box($val.1)))
        });

        $c.bench_function(&format!("eddie {} flipped", $name), |b| {
            b.iter(|| eddie::JaroWinkler::new().similarity(black_box($val.1), black_box($val.0)))
        });
    };
}

static STANDARD: (&str, &str) = ("wonderful", "wonderment");
static SHORT: (&str, &str) = ("hello", "hell");
static DIFFERENT: (&str, &str) = ("hello hi what is going on", "hell");
static LONG: (&str, &str) = (
    "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s",
    "wonderment double double"
);

static BOTH_LONG: (&str, &str) = (
    "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s",
    "; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s Doc-tests jaro running 0 tests test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s"
);

fn criterion_benchmark(c: &mut Criterion) {
    run_bench!("standard", STANDARD, c);
    run_bench!("short", SHORT, c);
    run_bench!("different", DIFFERENT, c);
    run_bench!("long", LONG, c);
    run_bench!("both long", BOTH_LONG, c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
