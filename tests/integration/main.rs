use opentelemetry::{KeyValue, global};
use opentelemetry_openmetrics::convert::ToOpenMetrics;
use opentelemetry_openmetrics::testsupport::TestMetricsReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::reader::MetricReader;

#[test]
fn test_conversion() {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    global::set_meter_provider(meter_provider);

    let meter = global::meter("meter.1");
    let gauge = meter
        .f64_gauge("f64.gauge")
        .with_description("A \"gauge\"\nFor testing")
        .build();
    gauge.record(4.2, &[]);
    gauge.record(4.22, &[KeyValue::new("kk", "vv")]);
    gauge.record(4.23, &[]);

    let counter = meter
        .f64_counter("f64.counter")
        .with_unit("seconds")
        .build();
    counter.add(12.5, &[]);

    let hist = meter.f64_histogram("histo").build();
    hist.record(0.0, &[]);
    hist.record(1.3, &[]);
    hist.record(1.4, &[]);
    hist.record(13.0, &[]);
    hist.record(4.22, &[KeyValue::new("kk", "vv")]);

    let mut metrics = ResourceMetrics::default();
    reader.collect(&mut metrics).unwrap();

    dbg!(&metrics);

    eprintln!("{}", ToOpenMetrics(&metrics));

    todo!();
}
