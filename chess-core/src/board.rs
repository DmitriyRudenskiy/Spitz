//! A chessboard small enough to keep in a register file and correct enough to trust a book with.
//!
//! This is the part of the rules the database needs: play the move a book names, know whether it was
//! legal, and compute the position key the interchange format indexes by. There is no move generator
//! here on purpose — nothing in the database asks "what can I play", only "was this playable".
//!
//! Squares are numbered a1=0, b1=1 … h1=7, a2=8 … h8=63.

use crate::key_table::KEY_TABLE;

pub const CASTLE_WHITE_KING: u8 = 1 << 0;
pub const CASTLE_WHITE_QUEEN: u8 = 1 << 1;
pub const CASTLE_BLACK_KING: u8 = 1 << 2;
pub const CASTLE_BLACK_QUEEN: u8 = 1 << 3;

/// Offsets into the key table: 12 piece kinds × 64 squares, then four castling rights, then eight
/// en-passant files, then the side to move.
pub const KEY_CASTLING: usize = 768;
pub const KEY_EN_PASSANT: usize = 772;
pub const KEY_TURN: usize = 780;

pub type Square = u8;

pub const fn file_of(square: Square) -> u8 {
    square % 8
}

pub const fn rank_of(square: Square) -> u8 {
    square / 8
}

const fn square(file: u8, rank: u8) -> Square {
    rank * 8 + file
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Colour {
    Black = 0,
    White = 1,
}

impl Colour {
    pub const fn other(self) -> Colour {
        match self {
            Colour::White => Colour::Black,
            Colour::Black => Colour::White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub colour: Colour,
    pub kind: Kind,
}

impl Piece {
    /// The piece's place on the key table's piece axis: black and white alternate, pawn first, which
    /// is the order the format was published in.
    const fn key_index(self) -> usize {
        2 * (self.kind as usize) + self.colour as usize
    }
}

/// A move: two squares and, for a pawn reaching the last rank, what it became.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Kind>,
}

impl Move {
    pub const fn simple(from: Square, to: Square) -> Move {
        Move { from, to, promotion: None }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveError {
    /// Nothing on the from-square.
    Empty,
    /// The piece on the from-square does not belong to the side to move.
    WrongColour,
    /// The to-square holds a piece of the mover's own colour.
    Occupied,
    /// The piece does not travel like that.
    BadGeometry,
    /// A pawn reached the last rank without saying what it became.
    NeedsPromotion,
    /// A promotion on a square that does not need one, or to a piece that cannot be promoted to.
    BadPromotion,
    /// The move leaves the mover's king attacked.
    LeavesKingInCheck,
    /// Castling is not available: no right, no rook, or the way is blocked.
    NoCastle,
    /// Castling through or into a square that is attacked.
    CastleIntoCheck,
}

#[derive(Clone, Copy, Debug)]
pub struct Board {
    squares: [Option<Piece>; 64],
    turn: Colour,
    castling: u8,
    /// The square a pawn could capture into on the way past, present only when that capture is
    /// actually possible — which is also what the position key counts.
    ep: Option<Square>,
}

const KNIGHT_STEPS: [(i8, i8); 8] =
    [(1, 2), (2, 1), (2, -1), (1, -2), (-1, -2), (-2, -1), (-2, 1), (-1, 2)];
const DIAGONALS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const STRAIGHTS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

impl Board {
    /// The position every book starts from.
    pub fn start() -> Board {
        let mut board = Board::empty();
        board.castling = CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN | CASTLE_BLACK_KING | CASTLE_BLACK_QUEEN;
        for file in 0..8 {
            board.place(square(file, 0), Piece { colour: Colour::White, kind: back_rank(file) });
            board.place(square(file, 1), Piece { colour: Colour::White, kind: Kind::Pawn });
            board.place(square(file, 6), Piece { colour: Colour::Black, kind: Kind::Pawn });
            board.place(square(file, 7), Piece { colour: Colour::Black, kind: back_rank(file) });
        }
        board
    }

    pub const fn empty() -> Board {
        Board { squares: [None; 64], turn: Colour::White, castling: 0, ep: None }
    }

    const fn place(&mut self, at: Square, piece: Piece) {
        self.squares[at as usize] = Some(piece);
    }

    pub const fn turn(&self) -> Colour {
        self.turn
    }

    pub const fn piece_at(&self, at: Square) -> Option<Piece> {
        self.squares[at as usize]
    }

    pub const fn castling(&self) -> u8 {
        self.castling
    }

    pub const fn en_passant(&self) -> Option<Square> {
        self.ep
    }

    /// The 8-byte key the interchange format indexes by. Computed from the position rather than
    /// carried along with it, so a board built wrongly cannot hide the fact.
    pub fn key(&self) -> u64 {
        let mut key = 0;
        for (at, slot) in self.squares.iter().enumerate() {
            if let Some(piece) = slot {
                key ^= KEY_TABLE[64 * piece.key_index() + at];
            }
        }
        for bit in 0..4 {
            if self.castling & (1 << bit) != 0 {
                key ^= KEY_TABLE[KEY_CASTLING + bit];
            }
        }
        if let Some(at) = self.ep {
            key ^= KEY_TABLE[KEY_EN_PASSANT + file_of(at) as usize];
        }
        if self.turn == Colour::White {
            key ^= KEY_TABLE[KEY_TURN];
        }
        key
    }

    pub fn king_square(&self, of: Colour) -> Option<Square> {
        self.squares
            .iter()
            .position(|slot| slot.is_some_and(|p| p.kind == Kind::King && p.colour == of))
            .map(|at| at as Square)
    }

    /// Is `at` attacked by any piece of `by`?
    pub fn is_attacked(&self, at: Square, by: Colour) -> bool {
        let pawn = Piece { colour: by, kind: Kind::Pawn };
        let below = if by == Colour::White { -1 } else { 1 };
        for step in [-1i8, 1] {
            if self.walk(at, step, below) == Some(pawn) {
                return true;
            }
        }
        let knight = Piece { colour: by, kind: Kind::Knight };
        for (dx, dy) in KNIGHT_STEPS {
            if self.walk(at, dx, dy) == Some(knight) {
                return true;
            }
        }
        let king = Piece { colour: by, kind: Kind::King };
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx, dy) != (0, 0) && self.walk(at, dx, dy) == Some(king) {
                    return true;
                }
            }
        }
        let bishop = Piece { colour: by, kind: Kind::Bishop };
        let rook = Piece { colour: by, kind: Kind::Rook };
        let queen = Piece { colour: by, kind: Kind::Queen };
        for (dx, dy) in DIAGONALS {
            if matches!(self.ray(at, dx, dy), Some(p) if p == bishop || p == queen) {
                return true;
            }
        }
        for (dx, dy) in STRAIGHTS {
            if matches!(self.ray(at, dx, dy), Some(p) if p == rook || p == queen) {
                return true;
            }
        }
        false
    }

    /// The piece one step from `at` by the given offsets, if that is on the board.
    fn walk(&self, at: Square, dx: i8, dy: i8) -> Option<Piece> {
        let (file, rank) = (file_of(at) as i8 + dx, rank_of(at) as i8 + dy);
        if (0..8).contains(&file) && (0..8).contains(&rank) {
            self.squares[(rank * 8 + file) as usize]
        } else {
            None
        }
    }

    /// The first piece along a ray, if any.
    fn ray(&self, at: Square, dx: i8, dy: i8) -> Option<Piece> {
        let (mut file, mut rank) = (file_of(at) as i8 + dx, rank_of(at) as i8 + dy);
        while (0..8).contains(&file) && (0..8).contains(&rank) {
            if let Some(piece) = self.squares[(rank * 8 + file) as usize] {
                return Some(piece);
            }
            file += dx;
            rank += dy;
        }
        None
    }

    /// Play a move, refusing anything that was not legal. The board is unchanged when this fails.
    pub fn apply(&mut self, mv: Move) -> Result<(), MoveError> {
        let mover = self.squares[mv.from as usize].ok_or(MoveError::Empty)?;
        if mover.colour != self.turn {
            return Err(MoveError::WrongColour);
        }
        if mv.from == mv.to {
            return Err(MoveError::BadGeometry);
        }
        if self.squares[mv.to as usize].is_some_and(|p| p.colour == mover.colour) {
            return Err(MoveError::Occupied);
        }
        if mover.kind == Kind::King && (mv.to as i8 - mv.from as i8).abs() == 2 {
            let mut next = *self;
            next.apply_castle(mv, mover)?;
            *self = next;
            return Ok(());
        }
        if !self.geometry_allows(mv, mover) {
            return Err(MoveError::BadGeometry);
        }
        let promotion_rank = if mover.colour == Colour::White { 7 } else { 0 };
        match mv.promotion {
            None => {}
            Some(kind @ (Kind::Knight | Kind::Bishop | Kind::Rook | Kind::Queen)) => {
                if mover.kind != Kind::Pawn || rank_of(mv.to) != promotion_rank {
                    return Err(MoveError::BadPromotion);
                }
                let _ = kind;
            }
            Some(_) => return Err(MoveError::BadPromotion),
        }
        if mover.kind == Kind::Pawn && rank_of(mv.to) == promotion_rank && mv.promotion.is_none() {
            return Err(MoveError::NeedsPromotion);
        }

        let mut next = *self;
        let captured = self.squares[mv.to as usize];
        next.squares[mv.from as usize] = None;
        next.squares[mv.to as usize] = Some(match mv.promotion {
            Some(kind) => Piece { colour: mover.colour, kind },
            None => mover,
        });
        // An en-passant capture leaves the captured pawn where it was.
        if mover.kind == Kind::Pawn && captured.is_none() && self.ep == Some(mv.to) {
            let behind = (mv.to as i8 + if mover.colour == Colour::White { -8 } else { 8 }) as usize;
            next.squares[behind] = None;
        }
        next.turn = self.turn.other();
        next.ep = next.en_passant_after(mv, mover);
        next.update_castling(mv);
        if next.king_square(mover.colour).is_some_and(|king| next.is_attacked(king, mover.colour.other())) {
            return Err(MoveError::LeavesKingInCheck);
        }
        *self = next;
        Ok(())
    }

    /// Does the piece travel from `from` to `to` the way its kind does, with the way clear?
    fn geometry_allows(&self, mv: Move, mover: Piece) -> bool {
        let (dx, dy) = (
            file_of(mv.to) as i8 - file_of(mv.from) as i8,
            rank_of(mv.to) as i8 - rank_of(mv.from) as i8,
        );
        let (adx, ady) = (dx.abs(), dy.abs());
        let captures = self.squares[mv.to as usize].is_some();
        match mover.kind {
            Kind::Pawn => {
                let forward = if mover.colour == Colour::White { 1 } else { -1 };
                let home = if mover.colour == Colour::White { 1 } else { 6 };
                if adx == 0 && dy == forward && !captures {
                    return true;
                }
                if dy == 2 * forward && adx == 0 && rank_of(mv.from) as i8 == home && !captures {
                    let between = (mv.from as i8 + 8 * forward) as usize;
                    return self.squares[between].is_none();
                }
                if adx == 1 && dy == forward {
                    return captures || self.ep == Some(mv.to);
                }
                false
            }
            Kind::Knight => (adx, ady) == (1, 2) || (adx, ady) == (2, 1),
            Kind::King => adx <= 1 && ady <= 1,
            Kind::Bishop => adx == ady && adx != 0 && self.path_clear(mv.from, mv.to),
            Kind::Rook => (dx == 0 || dy == 0) && (dx != 0 || dy != 0) && self.path_clear(mv.from, mv.to),
            Kind::Queen => (dx == 0 || dy == 0 || adx == ady) && (dx != 0 || dy != 0) && self.path_clear(mv.from, mv.to),
        }
    }

    fn path_clear(&self, from: Square, to: Square) -> bool {
        let (mut file, mut rank) = (file_of(from) as i8, rank_of(from) as i8);
        let (dx, dy) = ((file_of(to) as i8 - file).signum(), (rank_of(to) as i8 - rank).signum());
        loop {
            file += dx;
            rank += dy;
            if file == file_of(to) as i8 && rank == rank_of(to) as i8 {
                return true;
            }
            if self.squares[(rank * 8 + file) as usize].is_some() {
                return false;
            }
        }
    }

    fn apply_castle(&mut self, mv: Move, mover: Piece) -> Result<(), MoveError> {
        let king_side = mv.to > mv.from;
        let (right, king_from, rook_from, empty, safe) = match (mover.colour, king_side) {
            (Colour::White, true) => (CASTLE_WHITE_KING, 4u8, 7u8, &[5u8, 6][..], &[5u8, 6][..]),
            (Colour::White, false) => (CASTLE_WHITE_QUEEN, 4, 0, &[1, 2, 3][..], &[2, 3][..]),
            (Colour::Black, true) => (CASTLE_BLACK_KING, 60, 63, &[61, 62][..], &[61, 62][..]),
            (Colour::Black, false) => (CASTLE_BLACK_QUEEN, 60, 56, &[57, 58, 59][..], &[58, 59][..]),
        };
        if mv.from != king_from || mv.to as i8 != king_from as i8 + if king_side { 2 } else { -2 } {
            return Err(MoveError::NoCastle);
        }
        if self.castling & right == 0 {
            return Err(MoveError::NoCastle);
        }
        let rook = Piece { colour: mover.colour, kind: Kind::Rook };
        if self.squares[rook_from as usize] != Some(rook) {
            return Err(MoveError::NoCastle);
        }
        if empty.iter().any(|at| self.squares[*at as usize].is_some()) {
            return Err(MoveError::NoCastle);
        }
        let enemy = mover.colour.other();
        if safe.iter().any(|at| self.is_attacked(*at, enemy)) {
            return Err(MoveError::CastleIntoCheck);
        }
        let rook_to = if king_side { mv.to - 1 } else { mv.to + 1 };
        self.squares[king_from as usize] = None;
        self.squares[rook_from as usize] = None;
        self.squares[mv.to as usize] = Some(mover);
        self.squares[rook_to as usize] = Some(rook);
        self.turn = enemy;
        self.ep = None;
        self.update_castling(mv);
        Ok(())
    }

    fn update_castling(&mut self, mv: Move) {
        if mv.from == 4 {
            self.castling &= !(CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN);
        }
        if mv.from == 60 {
            self.castling &= !(CASTLE_BLACK_KING | CASTLE_BLACK_QUEEN);
        }
        for (corner, right) in [
            (0u8, CASTLE_WHITE_QUEEN),
            (7, CASTLE_WHITE_KING),
            (56, CASTLE_BLACK_QUEEN),
            (63, CASTLE_BLACK_KING),
        ] {
            if mv.from == corner || mv.to == corner {
                self.castling &= !right;
            }
        }
    }

    /// The square a pawn that has just pushed two ranks can be captured on, but only when an enemy
    /// pawn stands where it could take — the same condition the key counts.
    fn en_passant_after(&self, mv: Move, mover: Piece) -> Option<Square> {
        if mover.kind != Kind::Pawn {
            return None;
        }
        let forward: i8 = if mover.colour == Colour::White { 1 } else { -1 };
        if rank_of(mv.from) as i8 != if mover.colour == Colour::White { 1 } else { 6 }
            || rank_of(mv.to) as i8 - rank_of(mv.from) as i8 != 2 * forward
        {
            return None;
        }
        let enemy_pawn = Piece { colour: mover.colour.other(), kind: Kind::Pawn };
        for step in [-1i8, 1] {
            if self.walk(mv.to, step, 0) == Some(enemy_pawn) {
                return Some(((mv.from as u16 + mv.to as u16) / 2) as Square);
            }
        }
        None
    }
}

fn back_rank(file: u8) -> Kind {
    match file {
        0 | 7 => Kind::Rook,
        1 | 6 => Kind::Knight,
        2 | 5 => Kind::Bishop,
        3 => Kind::Queen,
        _ => Kind::King,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn piece(kind: Kind, colour: Colour) -> Piece {
        Piece { colour, kind }
    }

    #[test]
    fn the_start_position_has_the_key_the_format_agrees_on() {
        // Every book of the format contains the starting position, so its key is pinned here.
        assert_eq!(Board::start().key(), 0x463b96181691fc9c);
    }

    #[test]
    fn exactly_twenty_moves_are_legal_in_the_starting_position() {
        let start = Board::start();
        let mut legal = 0;
        for from in 0..64 {
            for to in 0..64 {
                let mut board = start;
                if board.apply(Move::simple(from, to)).is_ok() {
                    legal += 1;
                }
            }
        }
        assert_eq!(legal, 20);
    }

    #[test]
    fn a_move_that_is_not_legal_is_refused_for_the_right_reason() {
        let mut board = Board::start();
        assert_eq!(board.apply(Move::simple(8, 20)), Err(MoveError::BadGeometry), "a pawn does not L-move");
        assert_eq!(board.apply(Move::simple(8, 16)), Ok(()), "one step");
        assert_eq!(board.apply(Move::simple(12, 20)), Ok(()));
        assert_eq!(board.apply(Move::simple(13, 21)), Err(MoveError::WrongColour), "white again");
        assert_eq!(board.apply(Move::simple(1, 1)), Err(MoveError::BadGeometry), "nowhere to move");
        assert_eq!(board.apply(Move::simple(20, 12)), Err(MoveError::Occupied), "own pawn in the way");
        assert_eq!(board.apply(Move::simple(48, 0)), Err(MoveError::Empty), "nothing there");
    }

    #[test]
    fn a_pinned_piece_may_not_move_off_the_line() {
        let mut board = Board::empty();
        board.place(4, piece(Kind::King, Colour::White));
        board.place(12, piece(Kind::Pawn, Colour::White));
        board.place(20, piece(Kind::Bishop, Colour::White));
        board.place(36, piece(Kind::Queen, Colour::Black));
        board.place(60, piece(Kind::King, Colour::Black));
        assert!(board.is_attacked(12, Colour::Black), "the pawn down the file is attacked");
        assert_eq!(board.apply(Move::simple(20, 28)), Err(MoveError::LeavesKingInCheck));
        assert_eq!(board.apply(Move::simple(20, 12 + 8)), Err(MoveError::Occupied), "still pinned either way");
    }

    #[test]
    fn a_pawn_may_be_taken_on_the_way_past() {
        let mut board = Board::empty();
        board.place(4, piece(Kind::King, Colour::White));
        board.place(60, piece(Kind::King, Colour::Black));
        board.place(36, piece(Kind::Pawn, Colour::White)); // e5
        board.place(51, piece(Kind::Pawn, Colour::Black)); // d7
        board.turn = Colour::Black;
        assert!(board.apply(Move::simple(51, 35)).is_ok(), "black pushes two squares beside the pawn");
        assert_eq!(board.en_passant(), Some(43), "d6");
        assert!(board.apply(Move::simple(36, 43)).is_ok(), "white takes on the way past");
        assert_eq!(board.piece_at(35), None, "the captured pawn is gone from the board");
        assert_eq!(board.piece_at(43).map(|p| p.kind), Some(Kind::Pawn));
    }

    #[test]
    fn a_pawn_reaching_the_end_must_say_what_it_became() {
        let mut board = Board::empty();
        board.place(0, piece(Kind::King, Colour::White));
        board.place(48, piece(Kind::Pawn, Colour::White)); // a7
        board.place(63, piece(Kind::King, Colour::Black));
        assert_eq!(board.apply(Move::simple(48, 56)), Err(MoveError::NeedsPromotion));
        let queen = Move { from: 48, to: 56, promotion: Some(Kind::Queen) };
        assert!(board.apply(queen).is_ok());
        assert_eq!(board.piece_at(56).map(|p| p.kind), Some(Kind::Queen));
        assert_eq!(board.apply(Move { from: 56, to: 48, promotion: Some(Kind::Queen) }), Err(MoveError::BadPromotion), "a queen does not promote");
        let mut board = Board::empty();
        board.place(0, piece(Kind::King, Colour::White));
        board.place(8, piece(Kind::Pawn, Colour::White));
        board.place(63, piece(Kind::King, Colour::Black));
        assert_eq!(board.apply(Move { from: 8, to: 16, promotion: Some(Kind::King) }), Err(MoveError::BadPromotion));
    }

    #[test]
    fn castling_needs_the_right_the_rook_and_an_empty_way() {
        let mut board = Board::start();
        board.squares[5] = None;
        board.squares[6] = None;
        assert!(board.apply(Move::simple(4, 6)).is_ok(), "white castles out");
        assert_eq!(board.piece_at(6).map(|p| p.kind), Some(Kind::King));
        assert_eq!(board.piece_at(5).map(|p| p.kind), Some(Kind::Rook), "the rook came over");
        assert_eq!(board.castling() & (CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN), 0, "the right is spent");

        let mut board = Board::start();
        board.squares[5] = None;
        board.squares[6] = Some(piece(Kind::Bishop, Colour::White));
        assert_eq!(board.apply(Move::simple(4, 6)), Err(MoveError::NoCastle), "a piece in the way");

        let mut board = Board::start();
        board.castling = CASTLE_BLACK_KING | CASTLE_BLACK_QUEEN;
        board.turn = Colour::Black;
        board.squares[61] = None;
        board.squares[62] = None;
        board.squares[4] = Some(piece(Kind::Queen, Colour::White));
        assert_eq!(board.apply(Move::simple(60, 62)), Err(MoveError::CastleIntoCheck), "through the queen's square");

        let mut board = Board::start();
        board.squares[1] = None;
        board.squares[2] = None;
        board.squares[3] = None;
        board.castling = CASTLE_WHITE_QUEEN;
        assert!(board.apply(Move::simple(4, 2)).is_ok(), "and queen-side");
        assert_eq!(board.piece_at(3).map(|p| p.kind), Some(Kind::Rook));
    }

    #[test]
    fn a_refused_move_leaves_the_board_alone() {
        let before = Board::start();
        let mut board = before;
        assert!(board.apply(Move::simple(8, 20)).is_err());
        assert_eq!(board.key(), before.key());
    }

    #[test]
    fn the_key_changes_when_any_part_of_the_position_changes() {
        let start = Board::start();
        let mut after = start;
        after.apply(Move::simple(12, 20)).unwrap();
        assert_ne!(start.key(), after.key());
        let mut rights = start;
        rights.castling ^= CASTLE_WHITE_KING;
        assert_ne!(start.key(), rights.key());
        let mut turn = start;
        turn.turn = Colour::Black;
        assert_ne!(start.key(), turn.key());
        let mut ep = start;
        ep.ep = Some(16);
        assert_ne!(start.key(), ep.key());
    }

    #[test]
    fn a_key_is_a_fold_of_the_position_not_of_its_history() {
        // Two ways of reaching the same position give the same key, which is the whole point.
        let mut one = Board::start();
        one.apply(Move::simple(1, 17)).unwrap(); // Nc3
        one.apply(Move::simple(57, 49)).unwrap();
        one.apply(Move::simple(17, 26)).unwrap(); // Nb5
        let mut two = Board::start();
        two.apply(Move::simple(1, 17)).unwrap();
        two.apply(Move::simple(56, 48)).unwrap();
        two.apply(Move::simple(17, 26)).unwrap();
        assert_eq!(one.key(), two.key(), "same position, same key");
        assert_ne!(one.key(), Board::start().key());
    }
}
