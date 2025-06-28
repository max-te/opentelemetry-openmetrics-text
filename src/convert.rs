use std::borrow::Cow;
use std::fmt::{Display, Write};

use opentelemetry::KeyValue;
use opentelemetry_sdk::metrics::data::ResourceMetrics;

pub struct ToOpenMetrics<'a>(pub &'a ResourceMetrics);

impl<'a> ToOpenMetrics<'a> {
    pub const MIME_TYPE: &'static str =
        "application/openmetrics-text; version=1.0.0; charset=utf-8";
}

impl<'a> Display for ToOpenMetrics<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // let resource_attrs = self.0.resource().into_iter().collect::<Vec<_>>();

        for scope in self.0.scope_metrics() {
            // write_scope_info(f, scope)?;
            let scope_name = scope.scope().name();

            for metric in scope.metrics() {
                let mut name = sanitize_name(metric.name());
                let unit = convert_unit(metric.unit());
                if !unit.is_empty() {
                    name.push('_');
                    name.push_str(&unit);
                    writeln!(f, "# UNIT {name} {unit}")?;
                }
                if !metric.description().is_empty() {
                    writeln!(
                        f,
                        "# HELP {name} {}",
                        escape_label_value(metric.description())
                    )?;
                }
                match metric.data() {
                    opentelemetry_sdk::metrics::data::AggregatedMetrics::F64(metric_data) => {
                        match metric_data {
                            opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) => {
                                write_gauge(f, name, scope_name, gauge)?;
                            }
                            opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                                write_counter(f, name, scope_name, sum)?;
                            }
                            opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) => {
                                write_histogram(f, name, scope_name, histogram)?;
                            }
                            // See https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#exponential-histograms
                            // for exponential histograms
                            _ => unimplemented!(),
                        }
                    }
                    opentelemetry_sdk::metrics::data::AggregatedMetrics::U64(metric_data) => {
                        match metric_data {
                            opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) => {
                                write_gauge(f, name, scope_name, gauge)?;
                            }
                            opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                                write_counter(f, name, scope_name, sum)?;
                            }
                            opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) => {
                                write_histogram(f, name, scope_name, histogram)?;
                            }
                            _ => unimplemented!(),
                        }
                    }
                    opentelemetry_sdk::metrics::data::AggregatedMetrics::I64(metric_data) => {
                        match metric_data {
                            opentelemetry_sdk::metrics::data::MetricData::Gauge(gauge) => {
                                write_gauge(f, name, scope_name, gauge)?;
                            }
                            opentelemetry_sdk::metrics::data::MetricData::Sum(sum) => {
                                write_counter(f, name, scope_name, sum)?;
                            }
                            opentelemetry_sdk::metrics::data::MetricData::Histogram(histogram) => {
                                write_histogram(f, name, scope_name, histogram)?;
                            }
                            _ => unimplemented!(),
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// fn write_scope_info(
//     f: &mut std::fmt::Formatter<'_>,
//     scope: &opentelemetry_sdk::metrics::data::ScopeMetrics,
// ) -> Result<(), std::fmt::Error> {
//     // https://github.com/open-telemetry/opentelemetry-specification/blob/v1.45.0/specification/compatibility/prometheus_and_openmetrics.md#instrumentation-scope-1
//     let scope_name = sanitize_name(scope.scope().name());
//     let scope_attrs = print_attrs(scope.scope().attributes());
//     writeln!(f, "# TYPE {scope_name} info")?;
//     writeln!(f, "{scope_name}_info{{{}}} 1", scope_attrs)?;
//     Ok(())
// }

fn write_histogram<T: FastDisplay + Copy>(
    f: &mut std::fmt::Formatter<'_>,
    name: String,
    scope_name: &str,
    histogram: &opentelemetry_sdk::metrics::data::Histogram<T>,
) -> Result<(), std::fmt::Error> {
    let scope_name_attrs = &[KeyValue::new("otel_scope_name", scope_name.to_owned())];
    writeln!(f, "# TYPE {name} histogram")?;
    let ts = to_timestamp(histogram.time());
    let created = to_timestamp(histogram.start_time());
    writeln!(
        f,
        "{name}_created{{{attrs}}} {created}",
        attrs = print_attrs(scope_name_attrs.iter())
    )?;
    assert_eq!(
        histogram.temporality(),
        opentelemetry_sdk::metrics::Temporality::Cumulative,
        "Only cumulative Histograms are supported"
    );
    for point in histogram.data_points() {
        let mut attrs = print_attrs(point.attributes().chain(scope_name_attrs.iter()));

        writeln!(
            f,
            "{name}_count{{{attrs}}} {value} {ts}",
            value = point.count().fast_display()
        )?;

        writeln!(
            f,
            "{name}_sum{{{attrs}}} {value} {ts}",
            value = point.sum().fast_display()
        )?;

        // Non-compliant but useful:
        if let Some(min) = point.min() {
            writeln!(
                f,
                "{name}_min{{{attrs}}} {value} {ts}",
                value = min.fast_display()
            )?;
        }
        if let Some(max) = point.max() {
            writeln!(
                f,
                "{name}_max{{{attrs}}} {value} {ts}",
                value = max.fast_display()
            )?;
        }

        if !attrs.is_empty() {
            attrs.push(',');
        }
        for (bound, count) in std::iter::zip(point.bounds(), point.bucket_counts()) {
            writeln!(
                f,
                "{name}_bucket{{{attrs}le=\"{le}\"}} {value} {ts}",
                le = bound.fast_display(),
                value = count.fast_display()
            )?;
        }
        writeln!(
            f,
            "{name}_bucket{{{attrs}le=\"+Inf\"}} {value} {ts}",
            value = point.count().fast_display()
        )?;
    }
    Ok(())
}

fn write_counter<T: FastDisplay + Copy>(
    f: &mut std::fmt::Formatter<'_>,
    name: String,
    scope_name: &str,
    sum: &opentelemetry_sdk::metrics::data::Sum<T>,
) -> Result<(), std::fmt::Error> {
    let scope_name_attrs = &[KeyValue::new("otel_scope_name", scope_name.to_owned())];
    assert_eq!(
        sum.temporality(),
        opentelemetry_sdk::metrics::Temporality::Cumulative,
        "Only cumulative sums are supported"
    );
    if sum.is_monotonic() {
        writeln!(f, "# TYPE {name} counter")?;
        let ts = to_timestamp(sum.time());
        for point in sum.data_points() {
            let attrs = print_attrs(point.attributes().chain(scope_name_attrs));
            writeln!(
                f,
                "{name}_total{{{attrs}}} {value} {ts}",
                value = point.value().fast_display(),
            )?;
        }
        Ok(())
    } else {
        writeln!(f, "# TYPE {name} gauge")?;
        let ts = to_timestamp(sum.time());
        for point in sum.data_points() {
            let attrs = print_attrs(point.attributes().chain(scope_name_attrs));
            writeln!(
                f,
                "{name}{{{attrs}}} {value} {ts}",
                value = point.value().fast_display()
            )?;
        }
        Ok(())
    }
}

fn write_gauge<T: FastDisplay + Copy>(
    f: &mut std::fmt::Formatter<'_>,
    name: String,
    scope_name: &str,
    gauge: &opentelemetry_sdk::metrics::data::Gauge<T>,
) -> Result<(), std::fmt::Error> {
    let scope_name_attrs = &[KeyValue::new("otel_scope_name", scope_name.to_owned())];
    writeln!(f, "# TYPE {name} gauge")?;
    let ts = to_timestamp(gauge.time());
    for point in gauge.data_points() {
        let attrs = print_attrs(point.attributes().chain(scope_name_attrs));
        writeln!(
            f,
            "{name}{{{attrs}}} {value} {ts}",
            value = point.value().fast_display()
        )?;
    }
    Ok(())
}

fn to_timestamp(time: std::time::SystemTime) -> impl Display {
    let ts = time
        .duration_since(std::time::SystemTime::UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs_f64();
    ts.fast_display()
}

fn print_attrs<'a, I: Iterator<Item = &'a KeyValue>>(attrs: I) -> String {
    let mut result = String::new();
    for attr in attrs {
        if !result.is_empty() {
            result.push(',');
        }
        write!(
            &mut result,
            "{k}=\"{v}\"",
            k = sanitize_name(attr.key.as_str()),
            v = escape_label_value(&attr.value.as_str())
        )
        .unwrap();
    }
    result
}

fn escape_label_value<'a>(value: &'a str) -> Cow<'a, str> {
    let mut bytes = value.as_bytes();
    let first_escape = memchr::memchr3(b'\\', b'"', b'\n', bytes);
    let Some(first_escape) = first_escape else {
        return Cow::Borrowed(value);
    };
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let (head, tail) = bytes.split_at(first_escape);
    out.extend_from_slice(head);
    out.push(b'\\');
    bytes = tail;

    while let Some(next_escape) = memchr::memchr3(b'\\', b'"', b'\n', bytes) {
        out.extend_from_slice(&bytes[..next_escape]);
        out.push(b'\\');
        bytes = &bytes[next_escape + 1..];
    }
    out.extend_from_slice(bytes);

    Cow::Owned(unsafe {
        // SAFETY: The bytes are valid UTF-8 because they were obtained from a string.
        // Only valid UTF-8 characters are inserted in valid positions.
        String::from_utf8_unchecked(out)
    })
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn convert_unit(short_unit: &str) -> String {
    match short_unit {
        "" => String::new(),
        "s" => "seconds".to_owned(),
        "ms" => "milliseconds".to_owned(),
        "us" => "microseconds".to_owned(),
        "ns" => "nanoseconds".to_owned(),
        "1" => "ratio".to_owned(),
        "By" => "bytes".to_owned(),
        _ => sanitize_name(short_unit),
    }
}

trait FastDisplay {
    fn fast_display(&self) -> impl Display + Copy + use<Self>;
}

#[derive(Copy, Clone)]
struct RyuDisplay<N: ryu::Float>(N);

impl<N: ryu::Float> Display for RyuDisplay<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = ryu::Buffer::new();
        let formatted = buffer.format(self.0);
        f.write_str(formatted)
    }
}

impl FastDisplay for f64 {
    #[inline]
    fn fast_display(&self) -> impl Display + Copy + use<> {
        RyuDisplay(*self)
    }
}

#[derive(Copy, Clone)]
struct ItoaDisplay<N: itoa::Integer>(N);

impl<N: itoa::Integer> Display for ItoaDisplay<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer = itoa::Buffer::new();
        let formatted = buffer.format(self.0);
        f.write_str(formatted)
    }
}

impl FastDisplay for u64 {
    #[inline]
    fn fast_display(&self) -> impl Display + Copy + use<> {
        ItoaDisplay(*self)
    }
}

impl FastDisplay for i64 {
    #[inline]
    fn fast_display(&self) -> impl Display + Copy + use<> {
        ItoaDisplay(*self)
    }
}
