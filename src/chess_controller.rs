use piston::input::*;
use piston::Event;

use ChessAPI::board::*;
use ChessAPI::piece::*;

use crate::animation::AnimatePosition;
use crate::animation::Animation;
use crate::animation::AnimationTimingFunction;

pub struct ChessController {
    pub board: Board,
    pub from: Option<(usize, usize)>,
    pub moves: Vec<Move>,
    pub check: Option<(usize, usize)>,
    cursor_pos: [f64; 2],
    pub animations: Vec<AnimatePosition>,
}

impl ChessController {
    pub fn new() -> ChessController {
        ChessController {
            board: Board::new(),
            from: None,
            check: None,
            moves: Vec::new(),
            cursor_pos: [0.0, 0.0],
            animations: vec![],
        }
    }

    fn get_king_pos(&self, color: Color) -> (usize, usize) {
        (0..8)
            .map(|x| (0..8).map(move |y| (x, y)))
            .flatten()
            .find(|(x, y)| {
                if let Some(piece) = self.board.get_board()[*y][*x] {
                    return piece.color == color && piece.piece_type == PieceType::King;
                } else {
                    return false;
                }
            })
            .unwrap()
    }

    fn get_castle_move_rook_mv(
        &self,
        from: Position,
        to: Position,
    ) -> Option<((usize, usize), (usize, usize))> {
        let piece = self.board.get_board()[from.row as usize][from.col as usize].unwrap();

        if piece.piece_type != PieceType::King {
            return None;
        }

        if from.col == 4 && to.col == 6 {
            return Some(((7, from.row as usize), (5, from.row as usize)));
        }

        if from.col == 4 && to.col == 2 {
            return Some(((0, from.row as usize), (3, from.row as usize)));
        }

        None
    }

    pub fn event(&mut self, size: [u32; 2], e: &Event) {
        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos;
        }

        if !self.animations.is_empty() {
            return;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            let x = (self.cursor_pos[0]) / ((size[0] as f64) / 8.0);
            let y = (self.cursor_pos[1]) / ((size[1] as f64) / 8.0);

            let x = x as usize;
            let y = y as usize;

            let mv = self
                .moves
                .iter()
                .find(|mv| (mv.to.col, mv.to.row) == (x as i8, y as i8));

            if let Some(mv) = mv {
                self.animations = {
                    let mut mvs = vec![AnimatePosition::new()
                        .duration(0.2)
                        .timing_function(AnimationTimingFunction::Ease)
                        .start((mv.from.col as f64, mv.from.row as f64))
                        .end((mv.to.col as f64, mv.to.row as f64))];

                    if let Some((from, to)) = self.get_castle_move_rook_mv(mv.from, mv.to) {
                        mvs.push(
                            AnimatePosition::new()
                                .duration(0.2)
                                .timing_function(AnimationTimingFunction::Ease)
                                .start((from.0 as f64, from.1 as f64))
                                .end((to.0 as f64, to.1 as f64)),
                        );
                    }

                    mvs
                };

                self.board.make_move(mv).unwrap();

                if self.board.is_check() {
                    let current_turn = self.board.whose_turn();
                    self.check = Some(self.get_king_pos(current_turn));
                } else {
                    self.check = None;
                }

                self.from = None;
                self.moves = Vec::new();
            } else {
                let moves = self
                    .board
                    .generate_legal_moves()
                    .into_iter()
                    .filter(|mv| (mv.from.col, mv.from.row) == (x as i8, y as i8))
                    .collect::<Vec<Move>>();

                if moves.len() > 0 {
                    self.from = Some((x as usize, y as usize));
                    self.moves = moves;
                } else {
                    self.from = None;
                    self.moves = Vec::new();
                }
            }
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        for a in self.animations.iter_mut() {
            a.tick_dt(args.dt);
        }

        self.animations.retain(|a| !a.is_done());
    }
}
