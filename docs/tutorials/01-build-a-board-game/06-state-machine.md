# Chapter 6: State Machine Deep Dive

Understand the game phase flow you've already wired.

## What This Chapter Covers

No new files. This is a conceptual chapter that explains the Bevy States pattern and how the game phases connect.

## The Four Phases

```text
Setup ──(all pieces placed)──> RedTurn
                                  │
                    (player moves) │
                                  v
                               BlueTurn
                                  │
                      (AI moves)  │
                                  v
                               RedTurn  ──(flag captured)──> GameOver
                                                                │
                                                     (press R)  │
                                                                v
                                                              Setup
```

## Bevy States Patterns

| Pattern | What It Does | Where We Use It |
| --- | --- | --- |
| `init_state::<GamePhase>()` | Inserts `State<GamePhase>` and `NextState<GamePhase>` | `main.rs` |
| `OnEnter(Phase)` | Runs once when entering a state | `init_setup_system`, `game_over_system` |
| `.run_if(in_state(Phase))` | System runs every frame while in that state | All phase-specific systems |
| `NextState::set(phase)` | Queue a transition (applied next frame) | Click handlers, AI, restart |
| `State<GamePhase>` | Read-only access to current state | Status text, zone overlays |

## System-to-Phase Mapping

| System | Phase | Trigger |
| --- | --- | --- |
| `init_setup_system` | `OnEnter(Setup)` | Phase entry |
| `setup_click_system` | `Setup` | Every frame |
| `setup_status_system` | `Setup` | Every frame |
| `player_click_system` | `RedTurn` | Every frame |
| `play_status_system` | `RedTurn` | Every frame |
| `sync_highlights_system` | `RedTurn` | Every frame |
| `ai_turn_system` | `BlueTurn` | Every frame (Ch 7) |
| `game_over_system` | `OnEnter(GameOver)` | Phase entry |
| `restart_system` | `GameOver` | Every frame |
| `tick_feedback_system` | All | Every frame |
| `sync_pieces_system` | All | Every frame (with change detection) |
| `sync_setup_zone_system` | All | Every frame (self-manages visibility) |
| `tutorial_system` | All | Every frame (Ch 8) |

## How Transitions Happen

**Setup -> RedTurn** (in `setup_click_system`):

```rust
if queue.remaining.is_empty() {
    board.auto_fill_army(Team::Blue);
    next_state.set(GamePhase::RedTurn);
}
```

**RedTurn -> BlueTurn** (in `player_click_system`):

```rust
next_state.set(GamePhase::BlueTurn);
```

**BlueTurn -> RedTurn** (in `ai_turn_system`, Chapter 7):

```rust
next_state.set(GamePhase::RedTurn);
```

**Any -> GameOver** (on flag capture):

```rust
game_result.winner = Some(loser.opponent());
next_state.set(GamePhase::GameOver);
```

**GameOver -> Setup** (in `restart_system`):

```rust
if keys.just_pressed(KeyCode::KeyR) {
    *board = StrategoBoard::new();
    // ... reset all resources ...
    next_state.set(GamePhase::Setup);
}
```

## System Ordering

The `.after()` constraints in main.rs ensure rendering sees the latest state:

```rust
rendering::sync_pieces_system
    .after(input::setup_click_system)
    .after(input::player_click_system)
    // .after(ai::ai_turn_system)  -- added in Ch 7
```

Without these, `sync_pieces_system` might run before the click handler modifies the board, causing a 1-frame delay before pieces visually update.

## Checkpoint

Same as Chapter 5 -- no new code was added.

## Next

[Chapter 7: AI Opponent](07-ai-opponent.md) -- Add a random-move AI for the Blue team.
