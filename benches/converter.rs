use std::hint::black_box;
use std::rc::Rc;

use opentelemetry_openmetrics::convert::WriteOpenMetrics;
use tango_bench::{IntoBenchmarks, benchmark_fn, tango_benchmarks, tango_main};
use testsupport::resource_metrics::make_large_test_metrics;

pub fn benchmarks() -> impl IntoBenchmarks {
    let metrics = Rc::new(make_large_test_metrics());

    [benchmark_fn("display", move |b| {
        let met = metrics.clone();
        let mut buffer = String::new();
        b.iter(move || {
            buffer.clear();
            met.write_as_openmetrics(black_box(&mut buffer))
        })
    })]
}

tango_benchmarks!(benchmarks());
tango_main!();
