use crate::{
    config::AppSettings,
    core::{
        BandwidthAction, ClientPhase, GameMode, ObservedGameState, RuleMatch, evaluate,
        safe_effective_action,
    },
    league::{fetch_lcu_phase, find_lcu_lockfile, parse_live_state},
    steam::{default_steam_path, invoke_steam_transition},
};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Mutex, time::Duration};
use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEntry {
    pub timestamp_ms: u128,
    pub state: String,
    pub action: BandwidthAction,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSnapshot {
    pub observation: ObservedGameState,
    pub matched_rule: RuleMatch,
    pub desired_action: BandwidthAction,
    pub applied_action: Option<BandwidthAction>,
    pub baseline_action: Option<BandwidthAction>,
    pub adapter_health: String,
    pub last_transition: Option<u128>,
    pub automation_enabled: bool,
    pub steam_path: Option<PathBuf>,
    pub settings: AppSettings,
    pub activity: Vec<ActivityEntry>,
}

pub struct RuntimeState {
    pub inner: Mutex<RuntimeSnapshot>,
}

impl RuntimeState {
    pub fn new(mut settings: AppSettings) -> Self {
        if settings.steam_path.is_none() {
            settings.steam_path = default_steam_path();
        }
        apply_combat_limit(&mut settings);
        let observation = ObservedGameState::default();
        let matched_rule = evaluate(&settings.rules, &observation);
        Self {
            inner: Mutex::new(RuntimeSnapshot {
                observation,
                desired_action: matched_rule.action.clone(),
                matched_rule,
                applied_action: None,
                baseline_action: None,
                adapter_health: if settings.steam_path.is_some() {
                    "Ready"
                } else {
                    "Steam not found"
                }
                .into(),
                last_transition: None,
                automation_enabled: settings.automation_enabled,
                steam_path: settings.steam_path.clone(),
                settings,
                activity: Vec::new(),
            }),
        }
    }
}

pub fn apply_combat_limit(settings: &mut AppSettings) {
    for rule in &mut settings.rules {
        if rule.id == "arena-combat" {
            rule.action = BandwidthAction::Limit {
                bytes_per_second: settings.combat_limit_bytes_per_second,
            };
        }
    }
}

fn refresh_processes(system: &mut System) {
    let _ = system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing(),
    );
}

fn league_running(system: &System) -> bool {
    system.processes().values().any(|process| {
        process
            .name()
            .to_string_lossy()
            .eq_ignore_ascii_case("League of Legends.exe")
    })
}

fn state_label(state: &ObservedGameState) -> String {
    match (
        state.client_phase,
        state.mode,
        state.arena_phase,
        state.life,
    ) {
        (ClientPhase::Idle, _, _, _) => "League idle".into(),
        (ClientPhase::Lobby, _, _, _) => "League lobby".into(),
        (ClientPhase::Matchmaking, _, _, _) => "League matchmaking".into(),
        (ClientPhase::ReadyCheck, _, _, _) => "League ready check".into(),
        (ClientPhase::ChampSelect, _, _, _) => "League champion select".into(),
        (ClientPhase::Loading, _, _, _) => "League loading".into(),
        (ClientPhase::EndOfGame, _, _, _) => "League post-game".into(),
        (_, GameMode::Other, _, _) => "League match".into(),
        (_, GameMode::Arena, Some(phase), life) => format!("Arena {phase:?} · {life:?}"),
        _ => "League state unknown".into(),
    }
}

fn restrictiveness(action: &BandwidthAction) -> u8 {
    match action {
        BandwidthAction::Unlimited => 0,
        BandwidthAction::Limit { .. } => 1,
        BandwidthAction::Pause => 2,
    }
}

pub fn spawn_monitor(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let client = match reqwest::Client::builder()
            .timeout(Duration::from_millis(220))
            .danger_accept_invalid_certs(true)
            .no_proxy()
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                if let Some(state) = app.try_state::<RuntimeState>() {
                    state.inner.lock().unwrap().adapter_health =
                        format!("Telemetry client error: {error}");
                }
                return;
            }
        };
        let mut system = System::new();
        let mut relaxed_candidate: Option<BandwidthAction> = None;
        let mut relaxed_samples = 0_u8;

        loop {
            refresh_processes(&mut system);
            let running = league_running(&system);
            let observation = if !running {
                let mut state = ObservedGameState::default();
                if let Some(path) = find_lcu_lockfile(&system)
                    && let Some(phase) = fetch_lcu_phase(&client, &path).await
                {
                    state.client_phase = phase;
                }
                state
            } else {
                match client
                    .get("https://127.0.0.1:2999/liveclientdata/allgamedata")
                    .send()
                    .await
                    .and_then(|response| response.error_for_status())
                {
                    Ok(response) => match response.text().await {
                        Ok(body) => parse_live_state(&body)
                            .unwrap_or_else(|_| ObservedGameState::league_match(GameMode::Unknown)),
                        Err(_) => ObservedGameState::league_match(GameMode::Unknown),
                    },
                    Err(_) => ObservedGameState {
                        client_phase: ClientPhase::Loading,
                        mode: GameMode::Unknown,
                        arena_phase: None,
                        life: crate::core::LifeState::Unknown,
                        confidence: 60,
                    },
                }
            };

            let (should_apply, steam_path, desired, automation) = {
                let state = app.state::<RuntimeState>();
                let mut snapshot = state.inner.lock().unwrap();
                let matched = evaluate(&snapshot.settings.rules, &observation);
                let desired = matched.action.clone();
                // Steam exposes no reliable getter for its global pause gate. Until an
                // active transfer proves the gate was enabled, emulate Pause safely.
                let effective =
                    safe_effective_action(&desired, snapshot.settings.hard_pause_enabled);
                let current = snapshot.applied_action.clone();
                let changed =
                    snapshot.observation != observation || snapshot.desired_action != desired;
                snapshot.observation = observation.clone();
                snapshot.matched_rule = matched.clone();
                snapshot.desired_action = desired.clone();

                let immediate = current
                    .as_ref()
                    .is_none_or(|old| restrictiveness(&effective) >= restrictiveness(old));
                let stable_relaxation = if immediate {
                    relaxed_candidate = None;
                    relaxed_samples = 0;
                    true
                } else if relaxed_candidate.as_ref() == Some(&effective) {
                    relaxed_samples += 1;
                    relaxed_samples >= 2
                } else {
                    relaxed_candidate = Some(effective.clone());
                    relaxed_samples = 1;
                    false
                };
                let should_apply = snapshot.automation_enabled
                    && current.as_ref() != Some(&effective)
                    && stable_relaxation;
                if changed {
                    let timestamp_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();
                    let entry = ActivityEntry {
                        timestamp_ms,
                        state: state_label(&observation),
                        action: desired.clone(),
                        reason: matched.rule_name,
                    };
                    snapshot.activity.insert(0, entry);
                    snapshot.activity.truncate(40);
                    snapshot.last_transition = Some(timestamp_ms);
                }
                (
                    should_apply,
                    snapshot.steam_path.clone(),
                    effective,
                    snapshot.automation_enabled,
                )
            };

            if should_apply {
                let previous = app
                    .state::<RuntimeState>()
                    .inner
                    .lock()
                    .unwrap()
                    .applied_action
                    .clone();
                let result: Result<(), String> = steam_path
                    .as_deref()
                    .ok_or_else(|| "Steam executable not found".to_string())
                    .and_then(|path| {
                        invoke_steam_transition(path, previous.as_ref(), &desired)
                            .map_err(|error| error.to_string())
                    });
                let state = app.state::<RuntimeState>();
                let mut snapshot = state.inner.lock().unwrap();
                match result {
                    Ok(()) => {
                        snapshot.applied_action = Some(desired);
                        snapshot.adapter_health = "Connected".into();
                    }
                    Err(error) => snapshot.adapter_health = error,
                }
            }
            if automation {
                let snapshot = app.state::<RuntimeState>().inner.lock().unwrap().clone();
                let _ = app.emit("state-changed", snapshot);
            }
            tokio::time::sleep(if running {
                Duration::from_millis(250)
            } else {
                Duration::from_secs(2)
            })
            .await;
        }
    });
}
