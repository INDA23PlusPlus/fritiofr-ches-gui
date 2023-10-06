use serde::{Deserialize, Serialize};
use std::net::TcpStream;

use chess_network_protocol::ClientToServer;
use chess_network_protocol::ServerToClient;
use piston::input::*;
use piston::Event;

use ChessAPI::board::*;
use ChessAPI::piece::*;

use crate::animation::AnimatePosition;
use crate::animation::AnimateValue;
use crate::animation::Animation;
use crate::animation::AnimationTimingFunction;

pub struct ChessController {
    pub board: Board,
    pub from: Option<(usize, usize)>,
    pub last_from: Option<(usize, usize)>,
    pub last_to: Option<(usize, usize)>,
    pub moves: Vec<Move>,
    pub check: Option<(usize, usize)>,
    pub cursor_pos: [f64; 2],
    pub animations: Vec<AnimatePosition>,
    pub promotion_move: Option<Move>,
    client: ClientHandle,
    pub promotion_dialog: bool,
    pub promotion_color: Color,
    pub promotion_animation: AnimateValue,
    // 0 - Stalemate
    // 1 - Black wins
    // 2 - White wins
    pub end_state_show: bool,
    pub end_state: Option<usize>,
    pub end_state_animation: AnimateValue,
}

impl ChessController {
    pub fn new() -> ChessController {
        let tcp_stream = TcpStream::connect("0.0.0.0:8080").unwrap();

        let handshake = chess_network_protocol::ClientToServerHandshake {
            server_color: chess_network_protocol::Color::Black,
        };

        serde_json::to_writer(&tcp_stream, &handshake).unwrap();
        let client = ClientHandle { stream: tcp_stream };

        ChessController {
            board: Board::new(),
            from: None,
            last_from: None,
            last_to: None,
            check: None,
            moves: Vec::new(),
            cursor_pos: [0.0, 0.0],
            animations: vec![],
            client,

            promotion_move: None,
            promotion_animation: AnimateValue::new()
                .duration(0.1)
                .timing_function(AnimationTimingFunction::Ease)
                .finish(),
            promotion_color: Color::White,
            promotion_dialog: false,

            end_state_show: true,
            end_state: None,
            end_state_animation: AnimateValue::new()
                .duration(0.1)
                .timing_function(AnimationTimingFunction::Ease)
                .finish(),
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
        if self.board.whose_turn() == Color::Black {
            let mv = self.client.listen_move();
            self.make_move(&Move {
                from: Position {
                    row: mv.start_x as i8,
                    col: mv.start_y as i8,
                },
                to: Position {
                    row: mv.end_x as i8,
                    col: mv.end_y as i8,
                },
                promotion: match mv.promotion {
                    chess_network_protocol::Piece::BlackQueen => Some(PieceType::Queen),
                    chess_network_protocol::Piece::BlackRook => Some(PieceType::Rook),
                    chess_network_protocol::Piece::BlackBishop => Some(PieceType::Bishop),
                    chess_network_protocol::Piece::BlackKnight => Some(PieceType::Knight),
                    _ => None,
                },
            });
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::R => {
                    *self = ChessController::new();
                    return;
                }
                _ => {}
            }
        }

        if let Some(pos) = e.mouse_cursor_args() {
            self.cursor_pos = pos;
        }

        if !self.animations.is_empty() {
            return;
        }

        if let Some(Button::Mouse(MouseButton::Left)) = e.press_args() {
            if self.end_state.is_some() {
                self.end_state_show = !self.end_state_show;
                self.end_state_animation.reset();
                return;
            }

            if self.promotion_dialog {
                let width = 600.0;
                let size = width / 8.0;
                let gap = 10.0;

                for i in 0..4 {
                    let x = width / 2.0 - (size + gap) * 2.0 + gap / 2.0 + (size + gap) * i as f64;
                    let y = width / 2.0 - size / 2.0;

                    let [m_x, m_y] = self.cursor_pos;

                    if m_x > x && m_x < x + size && m_y > y && m_y < y + size {
                        let mut mv = self.promotion_move.clone().unwrap();
                        mv.promotion = Some(match i {
                            0 => PieceType::Queen,
                            1 => PieceType::Rook,
                            2 => PieceType::Bishop,
                            3 => PieceType::Knight,
                            _ => unreachable!(),
                        });

                        self.make_move(&mv);
                        self.promotion_move = None;
                        self.promotion_dialog = false;
                        self.promotion_animation.reset();
                        return;
                    }
                }

                self.moves = vec![];
                self.from = None;
                self.promotion_move = None;
                self.promotion_dialog = false;
                self.promotion_animation.reset();

                return;
            }

            let x = (self.cursor_pos[0]) / ((size[0] as f64) / 8.0);
            let y = (self.cursor_pos[1]) / ((size[1] as f64) / 8.0);

            let x = x as usize;
            let y = y as usize;

            let mv = self
                .moves
                .iter()
                .find(|mv| (mv.to.col, mv.to.row) == (x as i8, y as i8));

            if let Some(mv) = mv {
                if self.board.get_board()[mv.from.row as usize][mv.from.col as usize]
                    .unwrap()
                    .piece_type
                    == PieceType::Pawn
                    && (mv.to.row == 0 || mv.to.row == 7)
                {
                    self.promotion_color = self.board.whose_turn();
                    self.promotion_move = Some(mv.clone());
                    self.promotion_dialog = true;
                    self.promotion_animation.reset();
                    return;
                }

                self.make_move(&mv.clone());
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

    fn make_move(&mut self, mv: &Move) {
        self.client.make_move(
            (mv.from.col as usize, mv.from.row as usize),
            (mv.to.col as usize, mv.to.row as usize),
            chess_network_protocol::Piece::WhiteQueen,
        );

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

        self.last_from = Some((mv.from.col as usize, mv.from.row as usize));
        self.last_to = Some((mv.to.col as usize, mv.to.row as usize));

        if self.board.is_check() || self.board.is_checkmate() {
            let current_turn = self.board.whose_turn();
            self.check = Some(self.get_king_pos(current_turn));
        } else {
            self.check = None;
        }

        self.from = None;
        self.moves = Vec::new();

        if self.board.is_checkmate() {
            self.end_state = Some(if self.board.whose_turn() == Color::White {
                1
            } else {
                2
            });
            self.end_state_animation.reset();
        } else if self.board.is_stalemate() {
            self.end_state = Some(0);
            self.end_state_animation.reset();
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        if self.end_state.is_some() {
            self.end_state_animation.tick_dt(args.dt);
        }

        self.promotion_animation.tick_dt(args.dt);

        for a in self.animations.iter_mut() {
            a.tick_dt(args.dt);
        }

        self.animations.retain(|a| !a.is_done());
    }
}

struct ClientHandle {
    stream: TcpStream,
}

impl ClientHandle {
    fn make_move(
        &mut self,
        start: (usize, usize),
        end: (usize, usize),
        promotion: chess_network_protocol::Piece,
    ) {
        let mv = ClientToServer::Move(chess_network_protocol::Move {
            start_x: start.0,
            start_y: start.1,
            end_x: end.0,
            end_y: end.1,
            promotion,
        });

        serde_json::to_writer(&self.stream, &mv).unwrap();
    }

    fn listen_move(&mut self) -> chess_network_protocol::Move {
        let mut de = serde_json::Deserializer::from_reader(&self.stream);
        let deserialized = ServerToClient::deserialize(&mut de).unwrap();
        match deserialized {
            ServerToClient::State { move_made, .. } => move_made,
            _ => unimplemented!(),
        }
    }
}
