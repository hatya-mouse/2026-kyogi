//! Defines the data types that is used across the manager crate.

mod answer;
mod day_data;
mod init_data;

pub(super) use answer::{ApiActionPlanAnswer, ApiAgentKindAnswer, PostPlanResponse};
pub(super) use day_data::{ApiDayData, ApiOtherAgentsData, ApiTrafficData};
pub(super) use init_data::{ApiInitialData, ApiMap, ApiSpot};

pub(super) struct Api {
    /// Client to send request to the server.
    client: reqwest::Client,
    /// The URL of the server.
    server_url: String,
    /// Token for the server.
    token: String,
}

impl Api {
    pub(super) fn new(server_url: String, token: String) -> reqwest::Result<Api> {
        Ok(Api {
            client: reqwest::Client::builder().build()?,
            server_url,
            token,
        })
    }

    /// Gets the setting from the game server.
    pub(super) async fn get_initial(&self) -> reqwest::Result<ApiInitialData> {
        let url = format!("{}/setting", self.server_url.trim_end_matches('/'));

        self.client
            .get(url)
            .header("Procon-Token", self.token.as_str())
            .send()
            .await?
            .error_for_status()?
            .json::<ApiInitialData>()
            .await
    }

    /// Submit the kind of agents to the game server.
    pub(super) async fn post_agents(&self, agents: &ApiAgentKindAnswer) -> reqwest::Result<()> {
        self.client
            .post(self.server_url.clone())
            .header("Procon-Token", self.token.as_str())
            .json(agents)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Gets the status of the current day from the game server.
    pub(super) async fn get_day(&self) -> reqwest::Result<ApiDayData> {
        self.client
            .get(self.server_url.clone())
            .header("Procon-Token", self.token.as_str())
            .send()
            .await?
            .error_for_status()?
            .json::<ApiDayData>()
            .await
    }

    /// Submit the kind of agents to the game server.
    pub(super) async fn post_plan(
        &self,
        agents: &ApiActionPlanAnswer,
    ) -> reqwest::Result<PostPlanResponse> {
        self.client
            .post(self.server_url.clone())
            .header("Procon-Token", self.token.as_str())
            .json(agents)
            .send()
            .await?
            .error_for_status()?
            .json::<PostPlanResponse>()
            .await
    }
}
