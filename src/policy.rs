use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Team {
    Order,
    Chaos,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArenaPlayer {
    pub team: Team,
    pub is_dead: bool,
    pub is_local: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ArenaPhase {
    Preparation,
    Combat,
    Ambiguous,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LifeState {
    Alive,
    Dead,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum GameMode {
    Arena,
    Other,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClientPhase {
    Idle,
    Lobby,
    Matchmaking,
    ReadyCheck,
    ChampSelect,
    Loading,
    InGame,
    EndOfGame,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservedGameState {
    pub client_phase: ClientPhase,
    pub mode: GameMode,
    pub arena_phase: Option<ArenaPhase>,
    pub life: LifeState,
    pub confidence: u8,
}

impl Default for ObservedGameState {
    fn default() -> Self {
        Self {
            client_phase: ClientPhase::Idle,
            mode: GameMode::Unknown,
            arena_phase: None,
            life: LifeState::Unknown,
            confidence: 100,
        }
    }
}

impl ObservedGameState {
    #[cfg(test)]
    pub fn arena(arena_phase: ArenaPhase, life: LifeState) -> Self {
        Self {
            client_phase: ClientPhase::InGame,
            mode: GameMode::Arena,
            arena_phase: Some(arena_phase),
            life,
            confidence: 100,
        }
    }

    pub fn league_match(mode: GameMode) -> Self {
        Self {
            client_phase: ClientPhase::InGame,
            mode,
            arena_phase: None,
            life: LifeState::Unknown,
            confidence: 80,
        }
    }
}

pub fn classify_arena(players: &[ArenaPlayer]) -> ArenaPhase {
    let order_alive = players.iter().any(|p| !p.is_dead && p.team == Team::Order);
    let chaos_alive = players.iter().any(|p| !p.is_dead && p.team == Team::Chaos);
    match (order_alive, chaos_alive) {
        (true, true) => ArenaPhase::Combat,
        (true, false) | (false, true) => ArenaPhase::Preparation,
        (false, false) => ArenaPhase::Ambiguous,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum BandwidthAction {
    Unlimited,
    Limit { bytes_per_second: u64 },
    Pause,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RuleCondition {
    Always,
    Idle,
    Loading,
    NonArenaMatch,
    ArenaPreparation,
    ArenaCombatDead,
    ArenaCombat,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub priority: u32,
    pub condition: RuleCondition,
    pub action: BandwidthAction,
}

impl Rule {
    #[cfg(test)]
    pub fn always(id: &str, action: BandwidthAction) -> Self {
        Self {
            id: id.into(),
            name: id.into(),
            enabled: true,
            priority: 0,
            condition: RuleCondition::Always,
            action,
        }
    }

    fn matches(&self, state: &ObservedGameState) -> bool {
        match self.condition {
            RuleCondition::Always => true,
            RuleCondition::Idle => !matches!(
                state.client_phase,
                ClientPhase::Loading | ClientPhase::InGame
            ),
            RuleCondition::Loading => state.client_phase == ClientPhase::Loading,
            RuleCondition::NonArenaMatch => {
                state.client_phase == ClientPhase::InGame && state.mode != GameMode::Arena
            }
            RuleCondition::ArenaPreparation => {
                state.mode == GameMode::Arena && state.arena_phase == Some(ArenaPhase::Preparation)
            }
            RuleCondition::ArenaCombatDead => {
                state.mode == GameMode::Arena
                    && state.arena_phase == Some(ArenaPhase::Combat)
                    && state.life == LifeState::Dead
            }
            RuleCondition::ArenaCombat => {
                state.mode == GameMode::Arena && state.arena_phase != Some(ArenaPhase::Preparation)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMatch {
    pub rule_id: String,
    pub rule_name: String,
    pub action: BandwidthAction,
}

pub fn default_rules() -> Vec<Rule> {
    let definitions = [
        (
            "idle",
            "Outside a match",
            RuleCondition::Idle,
            BandwidthAction::Unlimited,
        ),
        (
            "loading",
            "Game loading",
            RuleCondition::Loading,
            BandwidthAction::Pause,
        ),
        (
            "other-match",
            "Non-Arena match",
            RuleCondition::NonArenaMatch,
            BandwidthAction::Pause,
        ),
        (
            "arena-prep",
            "Arena preparation",
            RuleCondition::ArenaPreparation,
            BandwidthAction::Unlimited,
        ),
        (
            "arena-dead",
            "Arena combat — dead",
            RuleCondition::ArenaCombatDead,
            BandwidthAction::Unlimited,
        ),
        (
            "arena-combat",
            "Arena combat",
            RuleCondition::ArenaCombat,
            BandwidthAction::Limit {
                bytes_per_second: 10_000_000,
            },
        ),
        (
            "fail-safe",
            "Unknown state",
            RuleCondition::Always,
            BandwidthAction::Pause,
        ),
    ];
    definitions
        .into_iter()
        .enumerate()
        .map(|(priority, (id, name, condition, action))| Rule {
            id: id.into(),
            name: name.into(),
            enabled: true,
            priority: priority as u32,
            condition,
            action,
        })
        .collect()
}

pub fn evaluate(rules: &[Rule], state: &ObservedGameState) -> RuleMatch {
    if let Some(rule) = rules
        .iter()
        .find(|rule| rule.enabled && rule.matches(state))
    {
        RuleMatch {
            rule_id: rule.id.clone(),
            rule_name: rule.name.clone(),
            action: rule.action.clone(),
        }
    } else {
        RuleMatch {
            rule_id: "internal-fail-safe".into(),
            rule_name: "Internal fail-safe".into(),
            action: BandwidthAction::Pause,
        }
    }
}

pub fn bytes_per_second_to_steam_kbps(bytes_per_second: u64) -> u64 {
    bytes_per_second.saturating_mul(8) / 1_000
}

pub fn effective_action(action: &BandwidthAction) -> BandwidthAction {
    action.clone()
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadGate {
    Enabled,
    Paused,
    Unknown,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PauseOwnership {
    baseline: DownloadGate,
    owned_pause: bool,
}

#[cfg(test)]
impl PauseOwnership {
    pub fn from_baseline(baseline: DownloadGate) -> Self {
        Self {
            baseline,
            owned_pause: false,
        }
    }

    pub fn after_owned_pause(mut self) -> Self {
        if self.baseline == DownloadGate::Enabled {
            self.owned_pause = true;
        }
        self
    }

    pub fn restore_gate(self) -> Option<DownloadGate> {
        self.owned_pause.then_some(DownloadGate::Enabled)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player(team: Team, dead: bool, local: bool) -> ArenaPlayer {
        ArenaPlayer {
            team,
            is_dead: dead,
            is_local: local,
        }
    }

    #[test]
    fn combat_requires_living_players_on_both_teams() {
        let players = vec![
            player(Team::Order, false, true),
            player(Team::Order, false, false),
            player(Team::Chaos, false, false),
            player(Team::Chaos, true, false),
        ];
        assert_eq!(classify_arena(&players), ArenaPhase::Combat);
    }

    #[test]
    fn dead_opponents_do_not_keep_combat_active() {
        let players = vec![
            player(Team::Order, false, true),
            player(Team::Order, false, false),
            player(Team::Chaos, true, false),
        ];
        assert_eq!(classify_arena(&players), ArenaPhase::Preparation);
    }

    #[test]
    fn death_and_revive_change_default_action_during_combat() {
        let rules = default_rules();
        let mut state = ObservedGameState::arena(ArenaPhase::Combat, LifeState::Alive);
        assert_eq!(
            evaluate(&rules, &state).action,
            BandwidthAction::Limit {
                bytes_per_second: 10_000_000
            }
        );
        state.life = LifeState::Dead;
        assert_eq!(evaluate(&rules, &state).action, BandwidthAction::Unlimited);
        state.life = LifeState::Alive;
        assert_eq!(
            evaluate(&rules, &state).action,
            BandwidthAction::Limit {
                bytes_per_second: 10_000_000
            }
        );
    }

    #[test]
    fn unknown_life_is_fail_safe_limited() {
        let rules = default_rules();
        let state = ObservedGameState::arena(ArenaPhase::Combat, LifeState::Unknown);
        assert_eq!(
            evaluate(&rules, &state).action,
            BandwidthAction::Limit {
                bytes_per_second: 10_000_000
            }
        );
    }

    #[test]
    fn non_arena_matches_pause() {
        let rules = default_rules();
        let state = ObservedGameState::league_match(GameMode::Other);
        assert_eq!(evaluate(&rules, &state).action, BandwidthAction::Pause);
    }

    #[test]
    fn rule_engine_uses_first_enabled_match() {
        let mut rules = default_rules();
        rules.insert(0, Rule::always("override", BandwidthAction::Unlimited));
        let state = ObservedGameState::league_match(GameMode::Other);
        assert_eq!(evaluate(&rules, &state).rule_id, "override");
    }

    #[test]
    fn decimal_megabytes_convert_to_steam_kilobits() {
        assert_eq!(bytes_per_second_to_steam_kbps(10_000_000), 80_000);
    }

    #[test]
    fn pause_ownership_never_resumes_a_preexisting_pause() {
        assert_eq!(
            PauseOwnership::from_baseline(DownloadGate::Paused).restore_gate(),
            None
        );
        assert_eq!(
            PauseOwnership::from_baseline(DownloadGate::Enabled)
                .after_owned_pause()
                .restore_gate(),
            Some(DownloadGate::Enabled)
        );
    }

    #[test]
    fn malformed_policy_still_fails_safe() {
        let state = ObservedGameState::league_match(GameMode::Unknown);
        let matched = evaluate(&[], &state);
        assert_eq!(matched.rule_id, "internal-fail-safe");
        assert_eq!(matched.action, BandwidthAction::Pause);
    }

    #[test]
    fn pause_is_not_silently_converted_to_a_throttle() {
        assert_eq!(
            effective_action(&BandwidthAction::Pause),
            BandwidthAction::Pause
        );
    }
}
