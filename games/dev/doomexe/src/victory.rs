//! Victory/credits screen shown after defeating the lich.
//!
//! Displays stats, narrator quip, and gold particles.

use crate::state::GameState;
use bevy::prelude::*;
use dj_engine::input::{ActionState, InputAction};
use dj_engine::particles::{ParticleConfig, ParticleEvent};
use dj_engine::prelude::{Inventory, QuestJournal, StoryFlags};

#[derive(Component)]
struct VictoryScreen;

/// Tracks how many enemies were killed across all areas.
#[derive(Resource, Default)]
pub struct DemoStats {
    pub enemies_killed: u32,
    pub gold_earned: u32,
    pub areas_cleared: u32,
    pub potions_used: u32,
}

pub struct VictoryPlugin;

impl Plugin for VictoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoStats>()
            .add_systems(OnEnter(GameState::Victory), setup_victory)
            .add_systems(
                Update,
                (victory_input, victory_particles).run_if(in_state(GameState::Victory)),
            )
            .add_systems(OnExit(GameState::Victory), teardown_victory);
    }
}

fn setup_victory(
    mut commands: Commands,
    inventory: Res<Inventory>,
    journal: Res<QuestJournal>,
    flags: Res<StoryFlags>,
    mut stats: ResMut<DemoStats>,
) {
    // Count cleared areas from flags
    let mut areas = 0;
    if flags
        .0
        .get("QuestTurnedIn_cellar")
        .copied()
        .unwrap_or(false)
    {
        areas += 1;
    }
    if flags.0.get("QuestTurnedIn_grove").copied().unwrap_or(false) {
        areas += 1;
    }
    if flags.0.get("QuestTurnedIn_crypt").copied().unwrap_or(false) {
        areas += 1;
    }
    stats.areas_cleared = areas;
    stats.gold_earned = inventory.currency_balance("gold") as u32;

    let quests_completed = ["clear_the_cellar", "purify_grove", "cleanse_the_crypt"]
        .iter()
        .filter(|q| journal.status(q).is_some())
        .count();

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.01, 0.05)),
            VictoryScreen,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("THE HAMSTER WAS RIGHT"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("Demo Complete"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Stats
            let stat_lines = [
                format!("Areas Cleared: {} / 3", areas),
                format!("Quests Completed: {} / 3", quests_completed),
                format!("Gold Collected: {}", inventory.currency_balance("gold")),
                format!("Items in Inventory: {}", inventory.used_slots()),
            ];

            for line in &stat_lines {
                parent.spawn((
                    Text::new(line.clone()),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    Node {
                        margin: UiRect::bottom(Val::Px(6.0)),
                        ..default()
                    },
                ));
            }

            // Narrator quip
            parent.spawn((
                Text::new(
                    "\"Not bad for a meat creature. I suppose you've earned my grudging respect.\"",
                ),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.4, 0.2)),
                Node {
                    margin: UiRect::new(Val::Px(0.0), Val::Px(0.0), Val::Px(30.0), Val::Px(5.0)),
                    ..default()
                },
            ));

            parent.spawn((
                Text::new("— The Hamster Narrator"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.35, 0.2)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Return prompt
            parent.spawn((
                Text::new("[Press any key to return to title]"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.5)),
            ));
        });

    info!(
        "Victory screen: {} areas, {} gold",
        areas, stats.gold_earned
    );
}

fn victory_input(actions: Res<ActionState>, mut next_state: ResMut<NextState<GameState>>) {
    if actions.just_pressed(InputAction::Confirm)
        || actions.just_pressed(InputAction::Cancel)
        || actions.just_pressed(InputAction::Up)
        || actions.just_pressed(InputAction::Down)
    {
        next_state.set(GameState::TitleScreen);
    }
}

/// Spawn gold particles periodically during victory screen.
fn victory_particles(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut particle_events: MessageWriter<ParticleEvent>,
) {
    *timer += time.delta_secs();
    if *timer >= 0.8 {
        *timer = 0.0;
        particle_events.write(ParticleEvent {
            position: Vec3::ZERO,
            config: ParticleConfig::gold_sparkle(),
        });
    }
}

fn teardown_victory(mut commands: Commands, query: Query<Entity, With<VictoryScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_stats_default() {
        let stats = DemoStats::default();
        assert_eq!(stats.enemies_killed, 0);
        assert_eq!(stats.gold_earned, 0);
        assert_eq!(stats.areas_cleared, 0);
        assert_eq!(stats.potions_used, 0);
    }

    #[test]
    fn test_demo_stats_tracking() {
        let mut stats = DemoStats::default();
        stats.enemies_killed = 13; // 5 rats + 4 grove + 3 skeletons + 1 lich
        stats.areas_cleared = 3;
        stats.gold_earned = 375; // 50 + 75 + 150 + starting
        assert_eq!(stats.enemies_killed, 13);
        assert_eq!(stats.areas_cleared, 3);
    }
}
