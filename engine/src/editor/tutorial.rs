//! Interactive tutorial overlay system.
//!
//! Displays a step-by-step guided tutorial with panel highlighting.
//! Tutorial content is loaded from an embedded JSON file.

use super::types::{EditorUiState, EditorView, COLOR_PRIMARY, COLOR_SECONDARY};
use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, Id, LayerId, Order, Painter, Pos2, Rect, RichText};
use serde::Deserialize;

const FIRST_GAME_TUTORIAL_JSON: &str = include_str!("../../template/tutorials/first_game.json");

// ── Data types (deserializable from JSON) ────────────────────────

#[derive(Deserialize, Clone, Debug)]
pub struct TutorialDef {
    pub name: String,
    pub steps: Vec<TutorialStep>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TutorialStep {
    pub title: String,
    pub body: String,
    pub target: TutorialTarget,
    pub completion: CompletionCondition,
}

#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TutorialTarget {
    TopPanel,
    LeftPanel,
    RightPanel,
    CentralPanel,
    FullScreen,
}

#[derive(Deserialize, Clone, Debug)]
pub enum CompletionCondition {
    Manual,
    ViewChanged(String),
    TabChanged(String),
    EntityPlaced,
    NodeSelected,
}

// ── Runtime state ────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct TutorialState {
    pub active: Option<ActiveTutorial>,
}

pub struct ActiveTutorial {
    pub def: TutorialDef,
    pub current_step: usize,
    pub panel_rects: PanelRects,
}

#[derive(Default, Clone, Debug)]
pub struct PanelRects {
    pub top: Option<Rect>,
    pub left: Option<Rect>,
    pub right: Option<Rect>,
    pub central: Option<Rect>,
}

impl PanelRects {
    fn rect_for_target(&self, target: &TutorialTarget) -> Option<Rect> {
        match target {
            TutorialTarget::TopPanel => self.top,
            TutorialTarget::LeftPanel => self.left,
            TutorialTarget::RightPanel => self.right,
            TutorialTarget::CentralPanel => self.central,
            TutorialTarget::FullScreen => None,
        }
    }
}

// ── Public API ───────────────────────────────────────────────────

/// Load and start the built-in "First Game" tutorial.
pub fn start_first_game_tutorial(state: &mut TutorialState) {
    match serde_json::from_str::<TutorialDef>(FIRST_GAME_TUTORIAL_JSON) {
        Ok(def) => {
            state.active = Some(ActiveTutorial {
                def,
                current_step: 0,
                panel_rects: PanelRects::default(),
            });
        }
        Err(e) => {
            warn!("Failed to load tutorial: {e}");
        }
    }
}

/// Update stored panel rects from the egui panel responses.
/// Call this after all panels have been drawn.
pub fn update_panel_rects<TopR, LeftR, RightR, CentralR>(
    world: &mut World,
    top: &egui::InnerResponse<TopR>,
    left: &egui::InnerResponse<LeftR>,
    right: &egui::InnerResponse<RightR>,
    central: &egui::InnerResponse<CentralR>,
) {
    let Some(mut state) = world.get_resource_mut::<TutorialState>() else {
        return;
    };
    let Some(active) = state.active.as_mut() else {
        return;
    };
    active.panel_rects = PanelRects {
        top: Some(top.response.rect),
        left: Some(left.response.rect),
        right: Some(right.response.rect),
        central: Some(central.response.rect),
    };
}

/// Draw the tutorial overlay. Call at the end of `editor_ui_system`.
pub fn draw_tutorial_overlay(ctx: &egui::Context, world: &mut World) {
    // Extract all needed data from the immutable borrow, then release it.
    enum Snapshot {
        Inactive,
        PastEnd,
        Active {
            step: TutorialStep,
            current_step: usize,
            total_steps: usize,
            panel_rects: PanelRects,
        },
    }

    let snapshot = match world.get_resource::<TutorialState>() {
        None => return,
        Some(state) => match &state.active {
            None => Snapshot::Inactive,
            Some(active) => match active.def.steps.get(active.current_step) {
                None => Snapshot::PastEnd,
                Some(step) => Snapshot::Active {
                    step: step.clone(),
                    current_step: active.current_step,
                    total_steps: active.def.steps.len(),
                    panel_rects: active.panel_rects.clone(),
                },
            },
        },
    };

    let (step, current_step, total_steps, target_rect) = match snapshot {
        Snapshot::Inactive => return,
        Snapshot::PastEnd => {
            world.resource_mut::<TutorialState>().active = None;
            return;
        }
        Snapshot::Active {
            step,
            current_step,
            total_steps,
            panel_rects,
        } => {
            let target_rect = panel_rects.rect_for_target(&step.target);
            (step, current_step, total_steps, target_rect)
        }
    };

    // Check auto-completion before drawing.
    if check_completion(world, &step.completion) {
        advance_step(world, 1);
        return;
    }

    let screen_rect = ctx.input(|i| i.viewport_rect());

    // Draw dim overlay with cutout for the target panel.
    draw_dim_overlay(ctx, screen_rect, target_rect);

    // Draw instruction window.
    let window_pos = instruction_window_pos(screen_rect, target_rect, &step.target);
    let is_last = current_step + 1 >= total_steps;

    let mut should_advance = false;
    let mut should_back = false;
    let mut should_skip = false;

    egui::Window::new("Tutorial")
        .id(Id::new("tutorial_instruction"))
        .collapsible(false)
        .resizable(false)
        .fixed_pos(window_pos)
        .default_width(340.0)
        .order(Order::Foreground)
        .show(ctx, |ui| {
            ui.label(
                RichText::new(&step.title)
                    .strong()
                    .size(16.0)
                    .color(COLOR_PRIMARY),
            );
            ui.add_space(6.0);
            ui.label(&step.body);
            ui.add_space(10.0);

            // Progress.
            ui.label(
                RichText::new(format!("Step {} of {}", current_step + 1, total_steps))
                    .small()
                    .color(Color32::GRAY),
            );
            ui.add_space(6.0);

            ui.horizontal(|ui| {
                if current_step > 0 && ui.button("Back").clicked() {
                    should_back = true;
                }

                let next_label = if is_last { "Done" } else { "Next" };
                if matches!(step.completion, CompletionCondition::Manual) {
                    if ui
                        .button(RichText::new(next_label).strong().color(COLOR_PRIMARY))
                        .clicked()
                    {
                        should_advance = true;
                    }
                } else {
                    ui.label(
                        RichText::new("(auto-advances)")
                            .small()
                            .italics()
                            .color(Color32::GRAY),
                    );
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .small_button(RichText::new("Skip Tutorial").color(COLOR_SECONDARY))
                        .clicked()
                    {
                        should_skip = true;
                    }
                });
            });
        });

    if should_skip {
        world.resource_mut::<TutorialState>().active = None;
    } else if should_advance {
        advance_step(world, 1);
    } else if should_back {
        advance_step(world, -1);
    }
}

// ── Internal helpers ─────────────────────────────────────────────

fn advance_step(world: &mut World, delta: i32) {
    let mut state = world.resource_mut::<TutorialState>();
    let Some(active) = state.active.as_mut() else {
        return;
    };
    let new = active.current_step as i32 + delta;
    if new < 0 {
        active.current_step = 0;
    } else if new >= active.def.steps.len() as i32 {
        // Tutorial complete.
        drop(state);
        world.resource_mut::<TutorialState>().active = None;
    } else {
        active.current_step = new as usize;
    }
}

fn check_completion(world: &World, condition: &CompletionCondition) -> bool {
    let Some(ui_state) = world.get_resource::<EditorUiState>() else {
        return false;
    };

    match condition {
        CompletionCondition::Manual => false,
        CompletionCondition::ViewChanged(target) => {
            let current = match ui_state.current_view {
                EditorView::Level => "Level",
                EditorView::StoryGraph => "StoryGraph",
            };
            current == target
        }
        CompletionCondition::TabChanged(target) => {
            let current = format!("{:?}", ui_state.browser_tab);
            current == *target
        }
        CompletionCondition::EntityPlaced => !ui_state.selected_entities.is_empty(),
        CompletionCondition::NodeSelected => ui_state.selected_node_id.is_some(),
    }
}

fn draw_dim_overlay(ctx: &egui::Context, screen: Rect, cutout: Option<Rect>) {
    let dim_color = Color32::from_black_alpha(180);
    let layer_id = LayerId::new(Order::Foreground, Id::new("tutorial_dim"));
    let painter = Painter::new(ctx.clone(), layer_id, screen);

    match cutout {
        Some(cutout) => {
            // Draw four strips around the cutout.
            // Top strip.
            painter.rect_filled(
                Rect::from_min_max(screen.min, Pos2::new(screen.max.x, cutout.min.y)),
                0.0,
                dim_color,
            );
            // Bottom strip.
            painter.rect_filled(
                Rect::from_min_max(Pos2::new(screen.min.x, cutout.max.y), screen.max),
                0.0,
                dim_color,
            );
            // Left strip (between top and bottom).
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(screen.min.x, cutout.min.y),
                    Pos2::new(cutout.min.x, cutout.max.y),
                ),
                0.0,
                dim_color,
            );
            // Right strip (between top and bottom).
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(cutout.max.x, cutout.min.y),
                    Pos2::new(screen.max.x, cutout.max.y),
                ),
                0.0,
                dim_color,
            );

            // Highlight border around the cutout.
            painter.rect_stroke(
                cutout,
                2.0,
                egui::Stroke::new(2.0, COLOR_PRIMARY),
                egui::StrokeKind::Outside,
            );
        }
        None => {
            // FullScreen — dim everything lightly.
            painter.rect_filled(screen, 0.0, Color32::from_black_alpha(120));
        }
    }
}

fn instruction_window_pos(
    screen: Rect,
    target: Option<Rect>,
    target_kind: &TutorialTarget,
) -> Pos2 {
    let margin = 20.0;
    let window_width = 360.0;

    match target_kind {
        TutorialTarget::FullScreen => {
            // Center on screen.
            Pos2::new(
                screen.center().x - window_width / 2.0,
                screen.center().y - 100.0,
            )
        }
        TutorialTarget::TopPanel => {
            // Below the top panel.
            let panel = target.unwrap_or(screen);
            Pos2::new(screen.center().x - window_width / 2.0, panel.max.y + margin)
        }
        TutorialTarget::LeftPanel => {
            // To the right of the left panel.
            let panel = target.unwrap_or(screen);
            Pos2::new(panel.max.x + margin, panel.min.y + margin)
        }
        TutorialTarget::RightPanel => {
            // To the left of the right panel.
            let panel = target.unwrap_or(screen);
            Pos2::new(panel.min.x - window_width - margin, panel.min.y + margin)
        }
        TutorialTarget::CentralPanel => {
            // Overlaid on the central panel, slightly offset.
            let panel = target.unwrap_or(screen);
            Pos2::new(panel.min.x + margin, panel.min.y + margin)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_tutorial_def() {
        let def: TutorialDef =
            serde_json::from_str(FIRST_GAME_TUTORIAL_JSON).expect("tutorial JSON should parse");
        assert_eq!(def.name, "Make Your First Game");
        assert_eq!(def.steps.len(), 8);
        assert_eq!(def.steps[0].target, TutorialTarget::FullScreen);
        assert!(matches!(
            def.steps[0].completion,
            CompletionCondition::Manual
        ));
    }

    #[test]
    fn test_deserialize_view_changed_condition() {
        let def: TutorialDef = serde_json::from_str(FIRST_GAME_TUTORIAL_JSON).unwrap();
        let step5 = &def.steps[4]; // "Switch to Story Graph"
        assert!(matches!(
            &step5.completion,
            CompletionCondition::ViewChanged(v) if v == "StoryGraph"
        ));
    }

    #[test]
    fn test_panel_rects_for_target() {
        let rects = PanelRects {
            top: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(800.0, 50.0))),
            left: Some(Rect::from_min_size(
                Pos2::new(0.0, 50.0),
                egui::vec2(250.0, 600.0),
            )),
            right: None,
            central: None,
        };
        assert!(rects.rect_for_target(&TutorialTarget::TopPanel).is_some());
        assert!(rects.rect_for_target(&TutorialTarget::LeftPanel).is_some());
        assert!(rects.rect_for_target(&TutorialTarget::RightPanel).is_none());
        assert!(rects.rect_for_target(&TutorialTarget::FullScreen).is_none());
    }

    #[test]
    fn test_instruction_window_pos_fullscreen_is_centered() {
        let screen = Rect::from_min_size(Pos2::ZERO, egui::vec2(1280.0, 720.0));
        let pos = instruction_window_pos(screen, None, &TutorialTarget::FullScreen);
        // Should be roughly centered.
        assert!(pos.x > 200.0 && pos.x < 700.0);
        assert!(pos.y > 100.0 && pos.y < 500.0);
    }
}
