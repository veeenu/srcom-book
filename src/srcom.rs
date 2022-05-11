use std::collections::HashMap;

use crate::PendingRun;

use anyhow::Result;
use http::HeaderMap;
use serde::Deserialize;

lazy_static::lazy_static! {
    pub static ref GAMES: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("w6jve26j", "Dark Souls");
        m.insert("lde3woe6", "Dark Souls Remastered");
        m.insert("y65lw01e", "Dark Souls II: Scholar of the First Sin");
        m.insert("m1zky010", "Dark Souls II");
        m.insert("k6qg0xdg", "Dark Souls III");
        m.insert("m1mn8kd2", "Demon's Souls");
        m.insert("j1neogy1", "Demon's Souls (2020)");
        m.insert("9d3kqg1l", "Bloodborne");
        m.insert("nd28z0ed", "Elden Ring");
        m.insert("o1y9zk26", "Sekiro");
        m
    };
}

mod pending_runs {
    use super::*;

    #[derive(Deserialize, Debug)]
    pub(super) struct RunsResource {
        data: Vec<Run>,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Run {
        id: String,
        weblink: String,
        comment: Option<String>,
        players: PlayersResource,
        category: CategoryResource,
        times: Times,
        submitted: String,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct PlayersResource {
        data: Vec<Player>,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Player {
        weblink: String,
        names: Names,
        location: Option<Location>,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Location {
        country: Country,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Country {
        code: String,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Names {
        international: String,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Times {
        primary: String,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct CategoryResource {
        data: Category,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Category {
        name: String,
    }

    impl TryFrom<Run> for PendingRun {
        type Error = String;

        fn try_from(run: Run) -> Result<Self, Self::Error> {
            if run.players.data.is_empty() {
                return Err(format!("Run {} has no players", run.id));
            }

            let player = &run.players.data[0];
            let times = {
                let d = iso8601_duration::Duration::parse(&run.times.primary)
                    .map_err(|e| format!("{:?}", e))?
                    .to_std();

                let as_secs = d.as_secs();
                let s = as_secs % 60;
                let m = (as_secs / 60) % 60;
                let h = as_secs / 3600;

                format!("{h:02}:{m:02}:{s:02}")
            };

            Ok(PendingRun {
                id: run.id,
                weblink: run.weblink,
                comment: run.comment.unwrap_or_else(String::new),
                category: run.category.data.name,
                submitted: run.submitted,
                player_name: player.names.international.clone(),
                player_location: player
                    .location
                    .as_ref()
                    .map(|l| l.country.code.clone()),
                player_url: player.weblink.clone(),
                times,
                booked_by: None,
            })
        }
    }

    impl TryFrom<RunsResource> for Vec<PendingRun> {
        type Error = String;

        fn try_from(runs_resource: RunsResource) -> Result<Self, Self::Error> {
            runs_resource
                .data
                .into_iter()
                .map(PendingRun::try_from)
                .collect()
        }
    }
}

mod user {
    use super::*;

    #[derive(Deserialize, Debug)]
    pub(super) struct UserResource {
        data: User,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct User {
        names: Names,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Names {
        international: String,
    }

    impl UserResource {
        pub(super) fn get_name(self) -> String {
            self.data.names.international
        }
    }
}

//
// Request endpoints
//

pub struct SrcomAPI {
    client: reqwest::Client,
}

impl Default for SrcomAPI {
    fn default() -> Self {
        SrcomAPI::new()
    }
}

impl SrcomAPI {
    pub fn new() -> Self {
        SrcomAPI {
            client: reqwest::Client::new(),
        }
    }

    async fn get_pending_runs_game(&self, game_id: &str) -> Result<Vec<PendingRun>> {
        if !GAMES.contains_key(game_id) {
            return Err(anyhow::Error::msg(format!("Unknown game {}", game_id)));
        }

        let uri = format!(
            "https://www.speedrun.com/api/v1/runs?game={game_id}&embed=players,category&status=new&max=200"
        );

        let body = self
            .client
            .get(uri)
            .header("Cache-Control", "no-store, must-revalidate")
            .send()
            .await?
            .text()
            .await?;

        let runs: pending_runs::RunsResource =
            serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&body))?;

        Vec::<PendingRun>::try_from(runs).map_err(anyhow::Error::msg)
    }

    pub async fn get_pending_runs(&self) -> Result<HashMap<String, Vec<PendingRun>>> {
        futures::future::try_join_all(GAMES.keys().map(|g| async move {
            self.get_pending_runs_game(g)
                .await
                .map(|r| (g.to_string(), r))
        }))
        .await
        .map(|v| v.into_iter().collect::<HashMap<_, _>>())
    }

    pub async fn get_profile(&self, headers: &HeaderMap) -> Result<String> {
        let api_key = headers
            .get("X-API-Key")
            .ok_or_else(|| anyhow::Error::msg("API Key missing"))
            .and_then(|h| h.to_str().map_err(anyhow::Error::msg))?;

        let body = self
            .client
            .get("https://www.speedrun.com/api/v1/profile")
            .header("X-API-Key", api_key)
            .send()
            .await?
            .text()
            .await?;

        let user: user::UserResource =
            serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&body))?;

        Ok(user.get_name())
    }
}
