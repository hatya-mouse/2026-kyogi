use serde::Deserialize;

/// The configuration for the worker.
#[derive(Deserialize, Debug)]
pub(crate) struct WorkerConfig {
    /// The port number to start the worker on.
    pub port: u16,
}
