use serde::Deserialize;
use shared::net::WorkerId;
use std::{collections::HashMap, net::Ipv4Addr};

/// The configuration for the manager.
#[derive(Deserialize, Debug)]
pub(crate) struct ManagerConfig {
    /// The port number to start the manager on.
    pub port: u16,
    /// IDs and data of the workers.
    pub workers: HashMap<WorkerId, WorkerInfo>,
}

/// Information of a worker such as IP address.
#[derive(Deserialize, Debug)]
pub(crate) struct WorkerInfo {
    /// IP address of the worker.
    pub address: Ipv4Addr,
    /// Port number of the worker.
    pub port: u16,
}
