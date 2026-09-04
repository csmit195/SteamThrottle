use crate::policy::{
    ArenaPlayer, ClientPhase, GameMode, LifeState, ObservedGameState, Team, classify_arena,
};
use serde::Deserialize;
use std::{fs, path::PathBuf};
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct LcuLockfile {
    pub port: u16,
    pub password: String,
    pub protocol: String,
}

pub fn parse_lockfile(contents: &str) -> Option<LcuLockfile> {
    let parts = contents.trim().split(':').collect::<Vec<_>>();
    if parts.len() != 5 {
        return None;
    }
    Some(LcuLockfile {
        port: parts[2].parse().ok()?,
        password: parts[3].to_owned(),
        protocol: parts[4].to_ascii_lowercase(),
    })
}

pub fn find_lcu_lockfile(system: &System) -> Option<PathBuf> {
    let from_process = system.processes().values().find_map(|process| {
        if !process
            .name()
            .to_string_lossy()
            .eq_ignore_ascii_case("LeagueClientUx.exe")
        {
            return None;
        }
        let path = process.exe()?.parent()?.join("lockfile");
        path.is_file().then_some(path)
    });
    from_process.or_else(|| {
        let common = PathBuf::from(r"C:\Riot Games\League of Legends\lockfile");
        common.is_file().then_some(common)
    })
}

pub fn parse_gameflow_phase(json: &str) -> ClientPhase {
    match json.trim().trim_matches('"') {
        "Lobby" => ClientPhase::Lobby,
        "Matchmaking" => ClientPhase::Matchmaking,
        "ReadyCheck" => ClientPhase::ReadyCheck,
        "ChampSelect" => ClientPhase::ChampSelect,
        "GameStart" | "Reconnect" => ClientPhase::Loading,
        "InProgress" => ClientPhase::InGame,
        "WaitingForStats" | "PreEndOfGame" | "EndOfGame" => ClientPhase::EndOfGame,
        _ => ClientPhase::Idle,
    }
}

pub async fn fetch_lcu_phase(client: &reqwest::Client, path: &PathBuf) -> Option<ClientPhase> {
    let lockfile = parse_lockfile(&fs::read_to_string(path).ok()?)?;
    let url = format!(
        "{}://127.0.0.1:{}/lol-gameflow/v1/gameflow-phase",
        lockfile.protocol, lockfile.port
    );
    let body = client
        .get(url)
        .basic_auth("riot", Some(lockfile.password))
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;
    Some(parse_gameflow_phase(&body))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LiveData {
    active_player: ActivePlayer,
    game_data: GameData,
    all_players: Vec<LivePlayer>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ActivePlayer {
    #[serde(default)]
    summoner_name: String,
    #[serde(default)]
    riot_id_game_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameData {
    game_mode: String,
    #[serde(default)]
    map_number: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LivePlayer {
    #[serde(default)]
    summoner_name: String,
    #[serde(default)]
    riot_id_game_name: String,
    team: String,
    is_dead: bool,
}

pub fn parse_live_state(json: &str) -> Result<ObservedGameState, serde_json::Error> {
    let data: LiveData = serde_json::from_str(json)?;
    if data.game_data.game_mode != "CHERRY" && data.game_data.map_number != 30 {
        return Ok(ObservedGameState::league_match(GameMode::Other));
    }

    let local_name = if data.active_player.riot_id_game_name.is_empty() {
        &data.active_player.summoner_name
    } else {
        &data.active_player.riot_id_game_name
    };
    let mut life = LifeState::Unknown;
    let players = data
        .all_players
        .into_iter()
        .filter_map(|player| {
            let name = if player.riot_id_game_name.is_empty() {
                &player.summoner_name
            } else {
                &player.riot_id_game_name
            };
            let is_local = !local_name.is_empty() && name.eq_ignore_ascii_case(local_name);
            if is_local {
                life = if player.is_dead {
                    LifeState::Dead
                } else {
                    LifeState::Alive
                };
            }
            let team = match player.team.as_str() {
                "ORDER" => Team::Order,
                "CHAOS" => Team::Chaos,
                _ => return None,
            };
            Some(ArenaPlayer {
                team,
                is_dead: player.is_dead,
                is_local,
            })
        })
        .collect::<Vec<_>>();

    Ok(ObservedGameState {
        client_phase: ClientPhase::InGame,
        mode: GameMode::Arena,
        arena_phase: Some(classify_arena(&players)),
        life,
        confidence: 100,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::ArenaPhase;

    #[test]
    fn parses_arena_combat_and_local_life() {
        let json = r#"{
          "activePlayer":{"summonerName":"Local"},
          "gameData":{"gameMode":"CHERRY","mapNumber":30},
          "allPlayers":[
            {"summonerName":"Local","team":"ORDER","isDead":false},
            {"summonerName":"Ally","team":"ORDER","isDead":false},
            {"summonerName":"Enemy","team":"CHAOS","isDead":false},
            {"summonerName":"Out","team":"CHAOS","isDead":true}
          ]
        }"#;
        let state = parse_live_state(json).unwrap();
        assert_eq!(state.mode, GameMode::Arena);
        assert_eq!(state.arena_phase, Some(ArenaPhase::Combat));
        assert_eq!(state.life, LifeState::Alive);
    }

    #[test]
    fn eliminated_players_do_not_confuse_preparation() {
        let json = r#"{
          "activePlayer":{"summonerName":"Local"},
          "gameData":{"gameMode":"CHERRY","mapNumber":30},
          "allPlayers":[
            {"summonerName":"Local","team":"ORDER","isDead":false},
            {"summonerName":"Ally","team":"ORDER","isDead":false},
            {"summonerName":"Eliminated","team":"CHAOS","isDead":true}
          ]
        }"#;
        let state = parse_live_state(json).unwrap();
        assert_eq!(state.arena_phase, Some(ArenaPhase::Preparation));
    }

    #[test]
    fn any_other_mode_is_a_non_arena_match() {
        let json = r#"{
          "activePlayer":{"summonerName":"Local"},
          "gameData":{"gameMode":"CLASSIC","mapNumber":11},
          "allPlayers":[]
        }"#;
        assert_eq!(parse_live_state(json).unwrap().mode, GameMode::Other);
    }

    #[test]
    fn maps_lcu_gameflow_without_exposing_credentials() {
        let lockfile = parse_lockfile("LeagueClient:1234:54321:secret:HTTPS").unwrap();
        assert_eq!(lockfile.port, 54321);
        assert_eq!(lockfile.password, "secret");
        assert_eq!(
            parse_gameflow_phase("\"ChampSelect\""),
            ClientPhase::ChampSelect
        );
        assert_eq!(
            parse_gameflow_phase("\"Matchmaking\""),
            ClientPhase::Matchmaking
        );
        assert_eq!(
            parse_gameflow_phase("\"ReadyCheck\""),
            ClientPhase::ReadyCheck
        );
    }
}
