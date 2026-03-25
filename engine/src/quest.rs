//! Quest state tracking for DJ Engine.
//!
//! Tracks quest lifecycle: available -> accepted -> in_progress -> completed -> turned_in.
//! Games use [`QuestJournal`] to manage player quest state and [`QuestEvent`] to
//! react to state changes.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lifecycle state of a quest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub enum QuestStatus {
    /// Quest is available but not yet accepted.
    Available,
    /// Quest has been accepted by the player.
    Accepted,
    /// Quest objectives are in progress.
    InProgress,
    /// All objectives are complete, ready to turn in.
    Completed,
    /// Quest has been turned in and rewards collected.
    TurnedIn,
    /// Quest was abandoned by the player.
    Abandoned,
}

/// Tracked state for a single quest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct QuestState {
    pub quest_id: String,
    pub status: QuestStatus,
    /// Objective progress counters (objective_id -> current/required).
    #[reflect(ignore)]
    pub objectives: HashMap<String, ObjectiveProgress>,
}

/// Progress on a single quest objective.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObjectiveProgress {
    pub current: u32,
    pub required: u32,
}

impl ObjectiveProgress {
    pub fn new(required: u32) -> Self {
        Self {
            current: 0,
            required,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current >= self.required
    }

    pub fn increment(&mut self, amount: u32) {
        self.current = (self.current + amount).min(self.required);
    }
}

/// Resource tracking all quest states for the current player.
#[derive(Resource, Default, Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Resource)]
pub struct QuestJournal {
    #[reflect(ignore)]
    pub quests: HashMap<String, QuestState>,
}

impl QuestJournal {
    /// Accept a quest (moves from Available or new to Accepted).
    pub fn accept(&mut self, quest_id: impl Into<String>) -> bool {
        let id = quest_id.into();
        if let Some(quest) = self.quests.get_mut(&id) {
            if quest.status == QuestStatus::Available {
                quest.status = QuestStatus::Accepted;
                return true;
            }
            return false;
        }
        self.quests.insert(
            id.clone(),
            QuestState {
                quest_id: id,
                status: QuestStatus::Accepted,
                objectives: HashMap::new(),
            },
        );
        true
    }

    /// Add an objective to track for a quest.
    pub fn add_objective(
        &mut self,
        quest_id: &str,
        objective_id: impl Into<String>,
        required: u32,
    ) {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            quest
                .objectives
                .insert(objective_id.into(), ObjectiveProgress::new(required));
        }
    }

    /// Increment progress on a quest objective. Returns true if the objective is now complete.
    pub fn progress_objective(&mut self, quest_id: &str, objective_id: &str, amount: u32) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if let Some(obj) = quest.objectives.get_mut(objective_id) {
                obj.increment(amount);
                return obj.is_complete();
            }
        }
        false
    }

    /// Check if all objectives for a quest are complete.
    pub fn all_objectives_complete(&self, quest_id: &str) -> bool {
        self.quests
            .get(quest_id)
            .map(|q| q.objectives.values().all(|o| o.is_complete()))
            .unwrap_or(false)
    }

    /// Mark a quest as completed (all objectives done).
    pub fn complete(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if matches!(
                quest.status,
                QuestStatus::Accepted | QuestStatus::InProgress
            ) {
                quest.status = QuestStatus::Completed;
                return true;
            }
        }
        false
    }

    /// Turn in a completed quest (collect rewards).
    pub fn turn_in(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if quest.status == QuestStatus::Completed {
                quest.status = QuestStatus::TurnedIn;
                return true;
            }
        }
        false
    }

    /// Abandon a quest.
    pub fn abandon(&mut self, quest_id: &str) -> bool {
        if let Some(quest) = self.quests.get_mut(quest_id) {
            if matches!(
                quest.status,
                QuestStatus::Accepted | QuestStatus::InProgress
            ) {
                quest.status = QuestStatus::Abandoned;
                return true;
            }
        }
        false
    }

    /// Get the status of a quest.
    pub fn status(&self, quest_id: &str) -> Option<QuestStatus> {
        self.quests.get(quest_id).map(|q| q.status)
    }

    /// Get all quests with a given status.
    pub fn quests_with_status(&self, status: QuestStatus) -> Vec<&QuestState> {
        self.quests
            .values()
            .filter(|q| q.status == status)
            .collect()
    }

    /// Count of active quests (accepted or in progress).
    pub fn active_count(&self) -> usize {
        self.quests
            .values()
            .filter(|q| matches!(q.status, QuestStatus::Accepted | QuestStatus::InProgress))
            .count()
    }
}

/// Events fired when quest state changes.
#[derive(Message, Debug, Clone, PartialEq)]
pub enum QuestEvent {
    Accepted {
        quest_id: String,
    },
    ObjectiveProgress {
        quest_id: String,
        objective_id: String,
        current: u32,
        required: u32,
    },
    Completed {
        quest_id: String,
    },
    TurnedIn {
        quest_id: String,
    },
    Abandoned {
        quest_id: String,
    },
}

/// Plugin providing quest tracking.
pub struct QuestPlugin;

impl Plugin for QuestPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<QuestJournal>()
            .register_type::<QuestJournal>()
            .register_type::<QuestStatus>()
            .add_message::<QuestEvent>();

        use crate::contracts::{AppContractExt, ContractEntry, PluginContract};
        app.register_contract(PluginContract {
            name: "QuestPlugin".into(),
            description: "Quest lifecycle tracking and objective progress".into(),
            resources: vec![ContractEntry::of::<QuestJournal>(
                "Player quest journal state",
            )],
            components: vec![],
            events: vec![ContractEntry::of::<QuestEvent>("Quest state change events")],
            system_sets: vec![],
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accept_new_quest() {
        let mut journal = QuestJournal::default();
        assert!(journal.accept("quest_01"));
        assert_eq!(journal.status("quest_01"), Some(QuestStatus::Accepted));
        assert_eq!(journal.active_count(), 1);
    }

    #[test]
    fn test_cannot_accept_twice() {
        let mut journal = QuestJournal::default();
        journal.accept("quest_01");
        // Second accept returns false (already accepted, not Available)
        assert!(!journal.accept("quest_01"));
    }

    #[test]
    fn test_objective_progress() {
        let mut journal = QuestJournal::default();
        journal.accept("quest_01");
        journal.add_objective("quest_01", "kill_wolves", 5);

        assert!(!journal.progress_objective("quest_01", "kill_wolves", 3));
        assert!(!journal.all_objectives_complete("quest_01"));

        assert!(journal.progress_objective("quest_01", "kill_wolves", 2));
        assert!(journal.all_objectives_complete("quest_01"));
    }

    #[test]
    fn test_objective_progress_clamps_to_required() {
        let mut journal = QuestJournal::default();
        journal.accept("quest_01");
        journal.add_objective("quest_01", "collect_herbs", 3);

        journal.progress_objective("quest_01", "collect_herbs", 100);
        let quest = journal.quests.get("quest_01").unwrap();
        assert_eq!(quest.objectives["collect_herbs"].current, 3);
    }

    #[test]
    fn test_complete_and_turn_in() {
        let mut journal = QuestJournal::default();
        journal.accept("quest_01");
        assert!(journal.complete("quest_01"));
        assert_eq!(journal.status("quest_01"), Some(QuestStatus::Completed));

        assert!(journal.turn_in("quest_01"));
        assert_eq!(journal.status("quest_01"), Some(QuestStatus::TurnedIn));
        assert_eq!(journal.active_count(), 0);
    }

    #[test]
    fn test_cannot_turn_in_incomplete() {
        let mut journal = QuestJournal::default();
        journal.accept("quest_01");
        assert!(!journal.turn_in("quest_01")); // not completed yet
    }

    #[test]
    fn test_abandon_quest() {
        let mut journal = QuestJournal::default();
        journal.accept("quest_01");
        assert!(journal.abandon("quest_01"));
        assert_eq!(journal.status("quest_01"), Some(QuestStatus::Abandoned));
        assert_eq!(journal.active_count(), 0);
    }

    #[test]
    fn test_quests_with_status() {
        let mut journal = QuestJournal::default();
        journal.accept("quest_01");
        journal.accept("quest_02");
        journal.accept("quest_03");
        journal.complete("quest_02");

        let accepted = journal.quests_with_status(QuestStatus::Accepted);
        assert_eq!(accepted.len(), 2);

        let completed = journal.quests_with_status(QuestStatus::Completed);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].quest_id, "quest_02");
    }
}
