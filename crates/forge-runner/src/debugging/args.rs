use crate::debugging::TraceVerbosity;
use crate::debugging::component::Component;
use clap::Args;
use debugging::Components;

#[derive(Args, Debug, Clone, Default, Eq, PartialEq)]
#[group(required = false, multiple = false)]
pub struct TraceArgs {
    /// Trace verbosity level
    #[arg(long)]
    trace_verbosity: Option<TraceVerbosity>,

    /// Components to include in the trace.
    #[arg(long, num_args = 1.., value_delimiter = ' ')]
    trace_components: Option<Vec<Component>>,
}

impl TraceArgs {
    /// Returns the [`Option<Components>`] based on the provided arguments.
    #[must_use]
    pub fn to_components(&self) -> Option<Components> {
        match (&self.trace_components, &self.trace_verbosity) {
            (None, Some(verbosity)) => Some(build_components(verbosity.to_components_vec())),
            (Some(components), None) => Some(build_components(components)),
            (None, None) => None,
            (Some(_), Some(_)) => {
                unreachable!("this case is impossible, as it is handled by clap")
            }
        }
    }
}

fn build_components<'a>(iter: impl IntoIterator<Item = &'a Component>) -> Components {
    Components::new(iter.into_iter().map(debugging::Component::from).collect())
}
