use crate::PendingRun;

use anyhow::Result;
use serde::Deserialize;

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
        location: Location,
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

    impl TryFrom<Run> for PendingRun {
        type Error = String;

        fn try_from(run: Run) -> Result<Self, Self::Error> {
            if run.players.data.len() == 0 {
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
                submitted: run.submitted,
                player_name: player.names.international.clone(),
                player_location: player.location.country.code.clone(),
                player_url: player.weblink.clone(),
                times: times,
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

mod moderators {
    use super::*;

    #[derive(Deserialize, Debug)]
    pub(super) struct ModsResource {
        data: GameData,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct GameData {
        moderators: ModsData,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct ModsData {
        data: Vec<Moderator>,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Moderator {
        names: Names,
    }

    #[derive(Deserialize, Debug)]
    pub(super) struct Names {
        international: String,
    }

    impl ModsResource {
        pub(super) fn names(self) -> Vec<String> {
            self.data
                .moderators
                .data
                .into_iter()
                .map(|moderator| moderator.names.international)
                .collect::<Vec<_>>()
        }
    }
}

//
// Request endpoints
//

pub async fn get_pending_runs() -> Result<Vec<PendingRun>> {
    const URI: &str =
        "https://www.speedrun.com/api/v1/runs?game=k6qg0xdg&embed=players&status=new&max=200";

    let body = reqwest::Client::new()
        .get(URI)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .await?
        // .json::<pending_runs::RunsResource>()
        .text()
        .await?;

    let runs: pending_runs::RunsResource =
        serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&body))?;

    Vec::<PendingRun>::try_from(runs).map_err(|e| anyhow::Error::msg(e))
}

#[tokio::test]
async fn test_get_pending_runs() {
    let pending_runs = get_pending_runs().await;
    println!("{:#?}", pending_runs);
}

pub async fn get_mods() -> Result<Vec<String>> {
    const URI: &str = "https://www.speedrun.com/api/v1/games/k6qg0xdg?embed=moderators";

    let body = reqwest::Client::new()
        .get(URI)
        .header("Cache-Control", "no-cache")
        .header("Pragma", "no-cache")
        .send()
        .await?
        .text()
        .await?;

    serde_path_to_error::deserialize(&mut serde_json::Deserializer::from_str(&body))
        .map(|mods: moderators::ModsResource| mods.names())
        .map_err(|e| anyhow::Error::msg(e))
}

#[tokio::test]
async fn test_get_mods() {
    let mods = get_mods().await;
    println!("{:#?}", mods);
}
