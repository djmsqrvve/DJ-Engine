//! Game rules: movement validation and combat resolution.

use crate::board::{CellTerrain, StrategoBoard, BOARD_HEIGHT, BOARD_WIDTH};
use crate::pieces::{PieceRank, PlacedPiece, Team};

/// Result of attacking a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CombatResult {
    AttackerWins,
    DefenderWins,
    BothDie,
    FlagCaptured(Team), // The team whose flag was captured (loser).
}

/// Resolve combat between attacker and defender.
pub fn resolve_combat(attacker: &PlacedPiece, defender: &PlacedPiece) -> CombatResult {
    if defender.rank == PieceRank::Flag {
        return CombatResult::FlagCaptured(defender.team);
    }
    if attacker.rank == PieceRank::Flag {
        // Flag can't attack, but if somehow it does, it loses.
        return CombatResult::DefenderWins;
    }

    let attack_str = attacker.rank.strength();
    let defend_str = defender.rank.strength();

    match attack_str.cmp(&defend_str) {
        std::cmp::Ordering::Greater => CombatResult::AttackerWins,
        std::cmp::Ordering::Less => CombatResult::DefenderWins,
        std::cmp::Ordering::Equal => CombatResult::BothDie,
    }
}

/// Check if a move from (fx, fy) to (tx, ty) is valid for the given team.
pub fn is_valid_move(
    board: &StrategoBoard,
    team: Team,
    from_x: usize,
    from_y: usize,
    to_x: usize,
    to_y: usize,
) -> bool {
    let Some(from_cell) = board.get(from_x, from_y) else {
        return false;
    };
    let Some(to_cell) = board.get(to_x, to_y) else {
        return false;
    };

    // Must have a piece of the right team.
    let Some(piece) = &from_cell.piece else {
        return false;
    };
    if piece.team != team {
        return false;
    }
    if !piece.rank.can_move() {
        return false;
    }

    // Target must not be a lake.
    if to_cell.terrain == CellTerrain::Lake {
        return false;
    }

    // Target must not have a friendly piece.
    if let Some(target_piece) = &to_cell.piece {
        if target_piece.team == team {
            return false;
        }
    }

    // Movement must be orthogonal.
    let dx = (to_x as i32 - from_x as i32).unsigned_abs() as usize;
    let dy = (to_y as i32 - from_y as i32).unsigned_abs() as usize;

    if dx == 0 && dy == 0 {
        return false;
    }
    if dx > 0 && dy > 0 {
        return false; // No diagonal movement.
    }

    let distance = dx + dy;

    // Scouts can move any distance in a straight line (but can't jump over pieces/lakes).
    if piece.rank == PieceRank::Scout {
        if distance > 1 {
            return is_clear_path(board, from_x, from_y, to_x, to_y);
        }
        return true;
    }

    // All other pieces move exactly 1 cell.
    distance == 1
}

/// Check that the straight-line path between two cells is free of pieces and lakes.
/// Does NOT check the destination cell (it may have an enemy piece to attack).
fn is_clear_path(
    board: &StrategoBoard,
    from_x: usize,
    from_y: usize,
    to_x: usize,
    to_y: usize,
) -> bool {
    let step_x = (to_x as i32 - from_x as i32).signum();
    let step_y = (to_y as i32 - from_y as i32).signum();

    let mut cx = from_x as i32 + step_x;
    let mut cy = from_y as i32 + step_y;

    while (cx as usize, cy as usize) != (to_x, to_y) {
        let x = cx as usize;
        let y = cy as usize;
        let Some(cell) = board.get(x, y) else {
            return false;
        };
        if cell.terrain == CellTerrain::Lake || cell.piece.is_some() {
            return false;
        }
        cx += step_x;
        cy += step_y;
    }

    true
}

/// Get all valid moves for a piece at (x, y).
pub fn valid_moves(board: &StrategoBoard, team: Team, x: usize, y: usize) -> Vec<(usize, usize)> {
    let Some(cell) = board.get(x, y) else {
        return Vec::new();
    };
    let Some(piece) = &cell.piece else {
        return Vec::new();
    };
    if piece.team != team || !piece.rank.can_move() {
        return Vec::new();
    }

    let mut moves = Vec::new();

    if piece.rank == PieceRank::Scout {
        // Check all 4 directions for multi-cell moves.
        for (dx, dy) in &[(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
            let mut nx = x as i32 + dx;
            let mut ny = y as i32 + dy;
            while nx >= 0 && nx < BOARD_WIDTH as i32 && ny >= 0 && ny < BOARD_HEIGHT as i32 {
                let tx = nx as usize;
                let ty = ny as usize;
                let Some(target) = board.get(tx, ty) else {
                    break;
                };
                if target.terrain == CellTerrain::Lake {
                    break;
                }
                if let Some(target_piece) = &target.piece {
                    // Can attack enemy, but can't go further.
                    if target_piece.team != team {
                        moves.push((tx, ty));
                    }
                    break;
                }
                moves.push((tx, ty));
                nx += dx;
                ny += dy;
            }
        }
    } else {
        // Normal piece: check 4 adjacent cells.
        for (nx, ny) in board.grid.neighbors(x, y) {
            if is_valid_move(board, team, x, y, nx, ny) {
                moves.push((nx, ny));
            }
        }
    }

    moves
}

/// Execute a move on the board. Returns the combat result if an attack occurred.
pub fn execute_move(
    board: &mut StrategoBoard,
    from_x: usize,
    from_y: usize,
    to_x: usize,
    to_y: usize,
) -> Option<CombatResult> {
    let from_cell = board.get(from_x, from_y)?.clone();
    let attacker = from_cell.piece?;

    let to_cell = board.get(to_x, to_y)?;
    let combat = if let Some(defender) = &to_cell.piece {
        Some(resolve_combat(&attacker, defender))
    } else {
        None
    };

    // Clear source.
    board.get_mut(from_x, from_y).unwrap().piece = None;

    match combat {
        Some(CombatResult::AttackerWins) | Some(CombatResult::FlagCaptured(_)) => {
            let cell = board.get_mut(to_x, to_y).unwrap();
            cell.piece = Some(PlacedPiece {
                revealed: true,
                ..attacker
            });
        }
        Some(CombatResult::DefenderWins) => {
            // Attacker removed, defender stays (but is now revealed).
            if let Some(defender) = &mut board.get_mut(to_x, to_y).unwrap().piece {
                defender.revealed = true;
            }
        }
        Some(CombatResult::BothDie) => {
            board.get_mut(to_x, to_y).unwrap().piece = None;
        }
        None => {
            // Simple move to empty cell.
            board.get_mut(to_x, to_y).unwrap().piece = Some(attacker);
        }
    }

    combat
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_piece(rank: PieceRank, team: Team) -> PlacedPiece {
        PlacedPiece {
            rank,
            team,
            revealed: false,
        }
    }

    #[test]
    fn combat_higher_rank_wins() {
        let result = resolve_combat(
            &make_piece(PieceRank::Marshal, Team::Red),
            &make_piece(PieceRank::General, Team::Blue),
        );
        assert_eq!(result, CombatResult::AttackerWins);
    }

    #[test]
    fn combat_lower_rank_loses() {
        let result = resolve_combat(
            &make_piece(PieceRank::Sergeant, Team::Red),
            &make_piece(PieceRank::Colonel, Team::Blue),
        );
        assert_eq!(result, CombatResult::DefenderWins);
    }

    #[test]
    fn combat_equal_rank_both_die() {
        let result = resolve_combat(
            &make_piece(PieceRank::Captain, Team::Red),
            &make_piece(PieceRank::Captain, Team::Blue),
        );
        assert_eq!(result, CombatResult::BothDie);
    }

    #[test]
    fn combat_capturing_flag_wins_game() {
        let result = resolve_combat(
            &make_piece(PieceRank::Scout, Team::Red),
            &make_piece(PieceRank::Flag, Team::Blue),
        );
        assert_eq!(result, CombatResult::FlagCaptured(Team::Blue));
    }

    #[test]
    fn flag_cannot_move() {
        let mut board = StrategoBoard::new();
        board.get_mut(0, 0).unwrap().piece = Some(make_piece(PieceRank::Flag, Team::Red));
        assert!(!is_valid_move(&board, Team::Red, 0, 0, 1, 0));
    }

    #[test]
    fn cannot_move_to_lake() {
        let mut board = StrategoBoard::new();
        board.get_mut(1, 4).unwrap().piece = Some(make_piece(PieceRank::Captain, Team::Red));
        assert!(!is_valid_move(&board, Team::Red, 1, 4, 2, 4)); // Lake at (2,4)
    }

    #[test]
    fn cannot_move_onto_friendly() {
        let mut board = StrategoBoard::new();
        board.get_mut(0, 0).unwrap().piece = Some(make_piece(PieceRank::Captain, Team::Red));
        board.get_mut(1, 0).unwrap().piece = Some(make_piece(PieceRank::Major, Team::Red));
        assert!(!is_valid_move(&board, Team::Red, 0, 0, 1, 0));
    }

    #[test]
    fn can_attack_enemy() {
        let mut board = StrategoBoard::new();
        board.get_mut(0, 0).unwrap().piece = Some(make_piece(PieceRank::Captain, Team::Red));
        board.get_mut(1, 0).unwrap().piece = Some(make_piece(PieceRank::Sergeant, Team::Blue));
        assert!(is_valid_move(&board, Team::Red, 0, 0, 1, 0));
    }

    #[test]
    fn scout_can_move_multiple_cells() {
        let mut board = StrategoBoard::new();
        board.get_mut(0, 0).unwrap().piece = Some(make_piece(PieceRank::Scout, Team::Red));
        let moves = valid_moves(&board, Team::Red, 0, 0);
        // Scout at (0,0) can move right along row 0 and down along col 0.
        assert!(moves.contains(&(5, 0)));
        assert!(moves.contains(&(0, 3)));
    }

    #[test]
    fn scout_blocked_by_lake() {
        let mut board = StrategoBoard::new();
        board.get_mut(2, 3).unwrap().piece = Some(make_piece(PieceRank::Scout, Team::Red));
        let moves = valid_moves(&board, Team::Red, 2, 3);
        // Moving down from (2,3) hits lake at (2,4), so (2,4) and beyond are blocked.
        assert!(!moves.contains(&(2, 4)));
        assert!(!moves.contains(&(2, 5)));
    }

    #[test]
    fn execute_move_captures_piece() {
        let mut board = StrategoBoard::new();
        board.get_mut(0, 0).unwrap().piece = Some(make_piece(PieceRank::Marshal, Team::Red));
        board.get_mut(1, 0).unwrap().piece = Some(make_piece(PieceRank::Sergeant, Team::Blue));

        let result = execute_move(&mut board, 0, 0, 1, 0);
        assert_eq!(result, Some(CombatResult::AttackerWins));
        assert!(board.get(0, 0).unwrap().piece.is_none());
        assert_eq!(
            board.get(1, 0).unwrap().piece.unwrap().rank,
            PieceRank::Marshal
        );
        assert!(board.get(1, 0).unwrap().piece.unwrap().revealed);
    }
}
