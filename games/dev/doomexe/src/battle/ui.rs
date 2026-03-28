use super::systems::{BattleEnemy, BattlePlayer};
use bevy::prelude::*;
use dj_engine::combat::AttackCooldown;
use dj_engine::data::components::CombatStatsComponent;
use dj_engine::inventory::Inventory;

#[derive(Component)]
pub struct BattleUIRoot;

#[derive(Component)]
pub struct BattleHudText;

pub fn setup_battle_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexEnd,
                ..default()
            },
            BackgroundColor(Color::NONE),
            BattleUIRoot,
        ))
        .with_children(|parent| {
            // HUD text showing HP
            parent.spawn((
                Text::new("Battle! Press Space to attack."),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                BattleHudText,
            ));
        });
}

pub fn update_battle_hud(
    player_query: Query<(&CombatStatsComponent, &AttackCooldown), With<BattlePlayer>>,
    enemy_query: Query<&CombatStatsComponent, With<BattleEnemy>>,
    inventory: Res<Inventory>,
    mut text_query: Query<&mut Text, With<BattleHudText>>,
) {
    let Ok((player_stats, cooldown)) = player_query.single() else {
        return;
    };
    let Ok(mut text) = text_query.single_mut() else {
        return;
    };

    let enemy_hp = enemy_query
        .iter()
        .next()
        .map(|s| format!("{}/{}", s.hp, s.max_hp))
        .unwrap_or_else(|| "defeated".into());

    let potions = inventory.count_item("health_potion");
    let gold = inventory.currency_balance("gold");

    let cd_bar = if cooldown.ready() {
        "READY".to_string()
    } else {
        let filled = (cooldown.timer.fraction() * 8.0) as usize;
        format!("[{}{}]", "#".repeat(filled), "-".repeat(8 - filled))
    };

    **text = format!(
        "BATTLE  |  You: {}/{}  |  Glitch: {}  |  Atk: {}  |  Potions: {} (Q)  |  Gold: {}",
        player_stats.hp, player_stats.max_hp, enemy_hp, cd_bar, potions, gold
    );
}

pub fn cleanup_battle_ui(mut commands: Commands, query: Query<Entity, With<BattleUIRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
