use std::fmt::Display;


pub(crate) trait FastDisplay {
    fn fast_display(&self) -> impl Display + Copy + use<Self>;
}

#[cfg(feature = "fast")]
mod fast_impl_with {
    use super::FastDisplay;
    use std::fmt::Display;

    #[derive(Copy, Clone)]
    struct RyuDisplay<N: ryu::Float>(N);

    impl<N: ryu::Float> Display for RyuDisplay<N> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut buffer = ryu::Buffer::new();
            let mut formatted = buffer.format(self.0);

            // Remove trailing .0 to match f64 Display
            let formatted_bytes = formatted.as_bytes();
            if formatted_bytes.ends_with(b".0") {
                formatted = unsafe {
                    std::str::from_utf8_unchecked(&formatted_bytes[..formatted_bytes.len() - 2])
                }
            }

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
}
#[cfg(not(feature = "fast"))]
mod fast_impl_without {
    use super::FastDisplay;
    use std::fmt::Display;

    impl FastDisplay for f64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            *self
        }
    }
    impl FastDisplay for u64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            *self
        }
    }

    impl FastDisplay for i64 {
        #[inline]
        fn fast_display(&self) -> impl Display + Copy + use<> {
            *self
        }
    }
}
