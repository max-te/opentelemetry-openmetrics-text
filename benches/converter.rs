use std::hint::black_box;
use std::rc::Rc;

use opentelemetry::KeyValue;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_openmetrics::convert::ToOpenMetrics;
use opentelemetry_openmetrics::testsupport::TestMetricsReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::reader::MetricReader;
use tango_bench::{IntoBenchmarks, benchmark_fn, tango_benchmarks, tango_main};

pub fn benchmarks() -> impl IntoBenchmarks {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let meter = meter_provider.meter("meter.1");

    let gauge = meter
        .f64_gauge("f64.gauge")
        .with_description("A \"gauge\"\nFor testing")
        .build();
    for i in 0..100 {
        gauge.record(4.22, &[KeyValue::new("foo.bar", format!("a{i}"))]);
    }

    let counter = meter
        .f64_counter("f64.counter")
        .with_unit("seconds")
        .build();
    counter.add(12.5, &[]);
    for i in 0..1000 {
        counter.add(
            4.22 * i as f64,
            &[KeyValue::new("high-low", format!("v\n{i}"))],
        );
    }

    let hist = meter.f64_histogram("histo").build();
    hist.record(0.0, &[]);
    hist.record(1.3, &[]);
    hist.record(1.4, &[]);
    hist.record(13.0, &[]);

    for i in 0..1000 {
        hist.record(
            4.22 / i as f64,
            &[
                KeyValue::new("x.y.z", format!("v{i}")),
                KeyValue::new("z.z.z", "fixed"),
                KeyValue::new("z.y.z", format!("0{}0", i + 1)),
            ],
        );
    }
    let mut metrics = ResourceMetrics::default();
    reader.collect(&mut metrics).unwrap();
    let metrics = Rc::new(metrics);

    [benchmark_fn("display", move |b| {
        let met = metrics.clone();
        let mut buffer = String::new();
        b.iter(move || {
            use std::fmt::Write;
            buffer.clear();
            write!(&mut buffer, "{}", black_box(ToOpenMetrics(&met)))
        })
    })]
}

tango_benchmarks!(benchmarks());
tango_main!();
