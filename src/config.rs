//! Configuration helpers

/// Helper macro to check is boolean choice is enabled by the configuration, defaulting to yes.
///
/// The current implementation works around the limitation of const functions in rust at the
/// time of writing.
macro_rules! is_enabled {
    ($env_var: tt) => {
        match option_env!($env_var) {
            Some(env_var) => match env_var.as_bytes() {
                b"false" => false,
                _ => true,
            },
            None => true,
        }
    };
}

pub(crate) use is_enabled;
