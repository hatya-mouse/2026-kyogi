//! Defines the data types that is used across the manager crate.

mod answer;
mod day_data;
mod init_data;

pub(super) use answer::{ApiActionPlanAnswer, ApiAgentKindAnswer};
pub(super) use day_data::{ApiAgentData, ApiDayData, ApiOtherAgentsData, ApiTrafficData};
pub(super) use init_data::{ApiInitialData, ApiMap, ApiSpot};
