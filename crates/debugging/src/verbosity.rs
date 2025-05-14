/// Verbosity that can be used to create verbosity-gated containers.
#[derive(Debug, Clone, Copy, Ord, PartialOrd, Eq, PartialEq)]
pub enum Verbosity {
    /// Lowest verbosity level. Will always compute any container.
    Minimal,
    /// Standard verbosity level. Will compute [`Standard`] containers.
    Standard,
    /// Detailed verbosity level. Will compute [`Detailed`] containers.
    Detailed,
}

impl Verbosity {
    /// Creates a new [`Standard`] verbosity container based on this verbosity level.
    pub fn standard<T>(self, supply_fn: impl FnOnce() -> T) -> Standard<T> {
        Standard::new(self, supply_fn)
    }

    /// Creates a new [`Detailed`] verbosity container based on this verbosity level.
    pub fn detailed<T>(self, supply_fn: impl FnOnce() -> T) -> Detailed<T> {
        Detailed::new(self, supply_fn)
    }
}

/// Macro to define a verbosity-gated container.
///
/// Assumes that the struct name provided (e.g., `Standard`) exactly matches
/// a variant in the [`Verbosity`] enum (e.g., [`Verbosity::Standard`]).
macro_rules! impl_verbosity_container {
    ($struct_name:ident) => {
        #[doc = concat!("Container for a value that is computed only if the verbosity level is [`Verbosity::", stringify!($struct_name), "`] or greater.")]
        #[derive(Debug, Clone)]
        pub struct $struct_name<T> {
            value: Option<T>,
        }

        impl<T> $struct_name<T> {
            #[doc = concat!("Creates a new [`", stringify!($struct_name), "`] instance from a given `supply_fn` and [`Verbosity`] level.")]
            #[doc = ""]
            #[doc = concat!("If the verbosity is less than [`Verbosity::", stringify!($struct_name), "`], the value won't be computed.")]
            #[must_use]
            pub fn new(verbosity: Verbosity, supply_fn: impl FnOnce() -> T) -> Self {
                $struct_name {
                    value: (verbosity >= Verbosity::$struct_name).then(supply_fn),
                }
            }

            #[doc = "Returns the reference value if it was computed based on the verbosity level, otherwise returns `None`."]
            #[must_use]
            pub fn as_option(&self) -> Option<&T> {
                self.value.as_ref()
            }
        }
    };
}

impl_verbosity_container!(Detailed);
impl_verbosity_container!(Standard);
