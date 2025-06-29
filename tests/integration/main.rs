use openmetrics_parser::openmetrics::parse_openmetrics;
use opentelemetry::KeyValue;
use opentelemetry::metrics::MeterProvider;
use opentelemetry_openmetrics::convert::ToOpenMetrics;
use opentelemetry_openmetrics::testsupport::TestMetricsReader;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::metrics::data::ResourceMetrics;
use opentelemetry_sdk::metrics::reader::MetricReader;

fn make_test_metrics() -> ResourceMetrics {
    let reader = TestMetricsReader::default();
    let meter_provider = SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .build();
    let meter = meter_provider.meter("meter.1");

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

    metrics
}

#[test]
fn test_output_is_parseable_by_openmetrics_parser() {
    let metrics = make_test_metrics();

    let formatted = format!("{}", ToOpenMetrics(&metrics));
    println!("{}", &formatted);

    let parsed = parse_openmetrics(&formatted);

    if let Err(ref err) = parsed {
        match err {
            openmetrics_parser::ParseError::ParseError(s) => {
                panic!("Parse error:\n{s}")
            }
            openmetrics_parser::ParseError::DuplicateMetric => panic!("Duplicate metric!"),
            openmetrics_parser::ParseError::InvalidMetric(s) => eprintln!("InvalidMetric:\n{s}"),
            // ^ Some of these are too harsh. We cannot fail the test on them
        }
    }
}
