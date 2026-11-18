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
        Move {
            from,
            to,
            promotion: None,
        }
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
    /// The square a double push just skipped over, as notation writes it. Whether it is a target
    /// at all is [`Board::en_passant`], which asks whether a pawn stands where it could take — and
    /// that narrower answer is what the position key counts.
    ep_field: Option<Square>,
    /// Halfmoves since the last pawn move or capture, the 50-move rule's counter.
    halfmove: u16,
    /// The move number a FEN's last field carries.
    fullmove: u16,
    /// Where each king stands, so that "does this move leave the king exposed?" costs one probe
    /// instead of a scan of the board. Kept by the two places that move a piece.
    kings: [Option<Square>; 2],
}

const KNIGHT_STEPS: [(i8, i8); 8] = [
    (1, 2),
    (2, 1),
    (2, -1),
    (1, -2),
    (-1, -2),
    (-2, -1),
    (-2, 1),
    (-1, 2),
];
const DIAGONALS: [(i8, i8); 4] = [(1, 1), (1, -1), (-1, 1), (-1, -1)];
const STRAIGHTS: [(i8, i8); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

impl Board {
    /// The position every book starts from.
    pub fn start() -> Board {
        let mut board = Board::empty();
        board.castling =
            CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN | CASTLE_BLACK_KING | CASTLE_BLACK_QUEEN;
        for file in 0..8 {
            board.place(
                square(file, 0),
                Piece {
                    colour: Colour::White,
                    kind: back_rank(file),
                },
            );
            board.place(
                square(file, 1),
                Piece {
                    colour: Colour::White,
                    kind: Kind::Pawn,
                },
            );
            board.place(
                square(file, 6),
                Piece {
                    colour: Colour::Black,
                    kind: Kind::Pawn,
                },
            );
            board.place(
                square(file, 7),
                Piece {
                    colour: Colour::Black,
                    kind: back_rank(file),
                },
            );
        }
        board
    }

    pub const fn empty() -> Board {
        Board {
            squares: [None; 64],
            turn: Colour::White,
            castling: 0,
            ep_field: None,
            halfmove: 0,
            fullmove: 1,
            kings: [None; 2],
        }
    }

    /// A whole position at once, for the codec that reads one.
    pub(crate) fn from_parts(
        squares: [Option<Piece>; 64],
        turn: Colour,
        castling: u8,
        ep: Option<Square>,
        halfmove: u16,
        fullmove: u16,
    ) -> Board {
        let mut kings = [None; 2];
        for (at, slot) in squares.iter().enumerate() {
            if let Some(piece) = slot
                && piece.kind == Kind::King
            {
                kings[piece.colour as usize] = Some(at as Square);
            }
        }
        Board {
            squares,
            turn,
            castling,
            ep_field: ep,
            halfmove,
            fullmove,
            kings,
        }
    }

    fn place(&mut self, at: Square, piece: Piece) {
        self.squares[at as usize] = Some(piece);
        if piece.kind == Kind::King {
            self.kings[piece.colour as usize] = Some(at);
        }
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

    pub fn en_passant(&self) -> Option<Square> {
        let at = self.ep_field?;
        let mover = self.turn;
        let white = mover == Colour::White;
        if rank_of(at) != if white { 5 } else { 2 } {
            return None;
        }
        let pawn = |slot: Square| {
            self.piece_at(slot)
                .is_some_and(|p| p.colour == mover && p.kind == Kind::Pawn)
        };
        // The pawn that takes stands beside the square jumped, one rank back from it.
        let behind = if white { at - 8 } else { at + 8 };
        if file_of(at) > 0 && pawn(behind - 1) {
            return Some(at);
        }
        if file_of(at) < 7 && pawn(behind + 1) {
            return Some(at);
        }
        None
    }

    /// The en-passant square as a FEN would write it, which is a square whether or not anyone can
    /// capture on it. The position does not depend on it; the string does.
    pub const fn ep_field(&self) -> Option<Square> {
        self.ep_field
    }

    pub const fn halfmove(&self) -> u16 {
        self.halfmove
    }

    pub const fn fullmove(&self) -> u16 {
        self.fullmove
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
        if let Some(at) = self.en_passant() {
            key ^= KEY_TABLE[KEY_EN_PASSANT + file_of(at) as usize];
        }
        if self.turn == Colour::White {
            key ^= KEY_TABLE[KEY_TURN];
        }
        key
    }

    pub fn king_square(&self, of: Colour) -> Option<Square> {
        self.kings[of as usize]
    }

    /// Is `at` attacked by any piece of `by`?
    pub fn is_attacked(&self, at: Square, by: Colour) -> bool {
        let pawn = Piece {
            colour: by,
            kind: Kind::Pawn,
        };
        let below = if by == Colour::White { -1 } else { 1 };
        for step in [-1i8, 1] {
            if self.walk(at, step, below) == Some(pawn) {
                return true;
            }
        }
        let knight = Piece {
            colour: by,
            kind: Kind::Knight,
        };
        for (dx, dy) in KNIGHT_STEPS {
            if self.walk(at, dx, dy) == Some(knight) {
                return true;
            }
        }
        let king = Piece {
            colour: by,
            kind: Kind::King,
        };
        for dx in -1..=1 {
            for dy in -1..=1 {
                if (dx, dy) != (0, 0) && self.walk(at, dx, dy) == Some(king) {
                    return true;
                }
            }
        }
        let bishop = Piece {
            colour: by,
            kind: Kind::Bishop,
        };
        let rook = Piece {
            colour: by,
            kind: Kind::Rook,
        };
        let queen = Piece {
            colour: by,
            kind: Kind::Queen,
        };
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
            next.advance_clocks(mover, false);
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
        next.advance_clocks(mover, captured.is_some());
        next.squares[mv.from as usize] = None;
        next.squares[mv.to as usize] = Some(match mv.promotion {
            Some(kind) => Piece {
                colour: mover.colour,
                kind,
            },
            None => mover,
        });
        if mover.kind == Kind::King {
            next.kings[mover.colour as usize] = Some(mv.to);
        }
        // An en-passant capture leaves the captured pawn where it was.
        if mover.kind == Kind::Pawn
            && captured.is_none()
            && file_of(mv.from) != file_of(mv.to)
            && self.en_passant() == Some(mv.to)
        {
            let behind =
                (mv.to as i8 + if mover.colour == Colour::White { -8 } else { 8 }) as usize;
            next.squares[behind] = None;
        }
        next.turn = self.turn.other();
        next.ep_field = skipped_over(mv, mover);
        next.update_castling(mv);
        if next
            .king_square(mover.colour)
            .is_some_and(|king| next.is_attacked(king, mover.colour.other()))
        {
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
                    return captures || self.en_passant() == Some(mv.to);
                }
                false
            }
            Kind::Knight => (adx, ady) == (1, 2) || (adx, ady) == (2, 1),
            Kind::King => adx <= 1 && ady <= 1,
            Kind::Bishop => adx == ady && adx != 0 && self.path_clear(mv.from, mv.to),
            Kind::Rook => {
                (dx == 0 || dy == 0) && (dx != 0 || dy != 0) && self.path_clear(mv.from, mv.to)
            }
            Kind::Queen => {
                (dx == 0 || dy == 0 || adx == ady)
                    && (dx != 0 || dy != 0)
                    && self.path_clear(mv.from, mv.to)
            }
        }
    }

    fn path_clear(&self, from: Square, to: Square) -> bool {
        let (mut file, mut rank) = (file_of(from) as i8, rank_of(from) as i8);
        let (dx, dy) = (
            (file_of(to) as i8 - file).signum(),
            (rank_of(to) as i8 - rank).signum(),
        );
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
        // `safe` is every square the king stands on or crosses: castling out of check is not a
        // castle, and the square the king leaves is the one a check arrives at.
        let (right, king_from, rook_from, empty, safe) = match (mover.colour, king_side) {
            (Colour::White, true) => (CASTLE_WHITE_KING, 4u8, 7u8, &[5u8, 6][..], &[4u8, 5, 6][..]),
            (Colour::White, false) => (CASTLE_WHITE_QUEEN, 4, 0, &[1, 2, 3][..], &[4u8, 2, 3][..]),
            (Colour::Black, true) => (
                CASTLE_BLACK_KING,
                60,
                63,
                &[61, 62][..],
                &[60u8, 61, 62][..],
            ),
            (Colour::Black, false) => (
                CASTLE_BLACK_QUEEN,
                60,
                56,
                &[57, 58, 59][..],
                &[60u8, 58, 59][..],
            ),
        };
        if mv.from != king_from || mv.to as i8 != king_from as i8 + if king_side { 2 } else { -2 } {
            return Err(MoveError::NoCastle);
        }
        if self.castling & right == 0 {
            return Err(MoveError::NoCastle);
        }
        let rook = Piece {
            colour: mover.colour,
            kind: Kind::Rook,
        };
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
        self.kings[mover.colour as usize] = Some(mv.to);
        self.turn = enemy;
        self.ep_field = None;
        self.update_castling(mv);
        Ok(())
    }

    /// The two clocks a FEN carries: the halfmove clock resets on a pawn move or a capture, and
    /// the move number once black has moved.
    fn advance_clocks(&mut self, mover: Piece, captured: bool) {
        self.halfmove = if mover.kind == Kind::Pawn || captured {
            0
        } else {
            self.halfmove.saturating_add(1)
        };
        if mover.colour == Colour::Black {
            self.fullmove = self.fullmove.saturating_add(1);
        }
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

/// A square as a FEN, a PGN and a UCI move all write it.
pub fn square_name(at: Square) -> String {
    let mut name = String::with_capacity(2);
    name.push(char::from(b'a' + file_of(at)));
    name.push(char::from(b'1' + rank_of(at)));
    name
}

/// Two characters as a square, or nothing when they are not one.
pub fn square_from_name(name: &str) -> Option<Square> {
    let bytes = name.as_bytes();
    if bytes.len() != 2 || !bytes[0].is_ascii_lowercase() || !bytes[1].is_ascii_digit() {
        return None;
    }
    let (file, rank) = (bytes[0] - b'a', bytes[1] - b'1');
    if file > 7 || rank > 7 {
        return None;
    }
    Some(rank * 8 + file)
}

/// The square a double pawn push jumped over, which is the square a FEN names on the next line —
/// whether or not an enemy pawn is standing where it could take.
fn skipped_over(mv: Move, mover: Piece) -> Option<Square> {
    if mover.kind != Kind::Pawn {
        return None;
    }
    let two = if mover.colour == Colour::White { 2 } else { -2 };
    let home = if mover.colour == Colour::White { 1 } else { 6 };
    (rank_of(mv.from) as i8 == home && rank_of(mv.to) as i8 - rank_of(mv.from) as i8 == two)
        .then(|| ((mv.from as u16 + mv.to as u16) / 2) as Square)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Squares by name, so a test says `e4` rather than 20 and cannot be off by a rank.
    fn sq(name: &str) -> Square {
        let file = name.as_bytes()[0] - b'a';
        let rank = name.as_bytes()[1] - b'1';
        rank * 8 + file
    }

    fn piece(kind: Kind, colour: Colour) -> Piece {
        Piece { colour, kind }
    }

    #[test]
    fn castling_out_of_check_is_refused() {
        let mut board = Board::empty();
        board.place(sq("e1"), piece(Kind::King, Colour::White));
        board.place(sq("h1"), piece(Kind::Rook, Colour::White));
        board.place(sq("a8"), piece(Kind::King, Colour::Black));
        board.place(sq("b4"), piece(Kind::Bishop, Colour::Black));
        board.castling = CASTLE_WHITE_KING;
        assert!(
            board.is_attacked(sq("e1"), Colour::Black),
            "the king is in check"
        );
        assert_eq!(
            board.apply(Move::simple(sq("e1"), sq("g1"))),
            Err(MoveError::CastleIntoCheck)
        );
    }

    /// A board with only the two kings, ready for a position a start position cannot hold.
    fn bare(kings: [(&str, Colour); 2]) -> Board {
        let mut board = Board::empty();
        for (name, colour) in kings {
            board.place(sq(name), piece(Kind::King, colour));
        }
        board
    }

    #[test]
    fn the_start_position_has_the_key_the_format_agrees_on() {
        // gm2600.bin holds this key with its ten strongest opening moves, which is what pins it.
        assert_eq!(Board::start().key(), 0x463b96181691fc9c);
    }

    #[test]
    fn exactly_twenty_moves_are_legal_in_the_starting_position() {
        let start = Board::start();
        let mut legal = Vec::new();
        for from in 0..64 {
            for to in 0..64 {
                let mut board = start;
                if board.apply(Move::simple(from, to)).is_ok() {
                    legal.push((from, to));
                }
            }
        }
        assert_eq!(
            legal.len(),
            20,
            "eight pawns two steps each, two knights three squares each"
        );
        assert!(
            legal.contains(&(sq("e2"), sq("e4"))),
            "a pawn may push two squares home"
        );
        assert!(
            legal.contains(&(sq("b1"), sq("c3"))),
            "the b1 knight may reach c3"
        );
        assert!(
            !legal.contains(&(sq("b1"), sq("b2"))),
            "but not onto its own pawn"
        );
        assert!(
            !legal.contains(&(sq("e1"), sq("e2"))),
            "nor onto its own pawn"
        );
    }

    #[test]
    fn a_move_that_is_not_legal_is_refused_for_the_right_reason() {
        let mut board = Board::start();
        assert_eq!(
            board.apply(Move::simple(sq("d4"), sq("d5"))),
            Err(MoveError::Empty),
            "nothing there"
        );
        assert_eq!(
            board.apply(Move::simple(sq("b1"), sq("b1"))),
            Err(MoveError::BadGeometry),
            "nowhere to move"
        );
        assert_eq!(
            board.apply(Move::simple(sq("c1"), sq("b2"))),
            Err(MoveError::Occupied),
            "own pawn in the way"
        );
        assert_eq!(
            board.apply(Move::simple(sq("a2"), sq("a5"))),
            Err(MoveError::BadGeometry),
            "a pawn does not hop"
        );
        assert_eq!(
            board.apply(Move::simple(sq("a2"), sq("a3"))),
            Ok(()),
            "one step"
        );
        assert_eq!(
            board.apply(Move::simple(sq("d2"), sq("d3"))),
            Err(MoveError::WrongColour),
            "white may not play twice"
        );
        board.turn = Colour::White;
        assert_eq!(
            board.apply(Move::simple(sq("a3"), sq("a4"))),
            Ok(()),
            "the same pawn's next step"
        );
        board.turn = Colour::White;
        assert_eq!(
            board.apply(Move::simple(sq("a4"), sq("a6"))),
            Err(MoveError::BadGeometry),
            "no second double step"
        );
    }

    #[test]
    fn a_pinned_bishop_may_not_step_off_the_line() {
        let mut board = bare([("e1", Colour::White), ("e8", Colour::Black)]);
        board.place(sq("e2"), piece(Kind::Bishop, Colour::White));
        board.place(sq("e4"), piece(Kind::Queen, Colour::Black));
        board.place(sq("a2"), piece(Kind::Pawn, Colour::White));
        assert!(
            board.is_attacked(sq("e2"), Colour::Black),
            "the bishop's square is attacked down the file"
        );
        assert_eq!(
            board.apply(Move::simple(sq("e2"), sq("d3"))),
            Err(MoveError::LeavesKingInCheck),
            "off the file"
        );
        assert_eq!(
            board.apply(Move::simple(sq("a2"), sq("a3"))),
            Ok(()),
            "another piece may still move"
        );
    }

    #[test]
    fn a_pawn_may_be_taken_on_the_way_past() {
        let mut board = bare([("e1", Colour::White), ("e8", Colour::Black)]);
        board.place(sq("e5"), piece(Kind::Pawn, Colour::White));
        board.place(sq("d7"), piece(Kind::Pawn, Colour::Black));
        board.turn = Colour::Black;
        assert!(
            board.apply(Move::simple(sq("d7"), sq("d5"))).is_ok(),
            "black pushes two squares beside the pawn"
        );
        assert_eq!(board.en_passant(), Some(sq("d6")));
        assert!(
            board.apply(Move::simple(sq("e5"), sq("d6"))).is_ok(),
            "white takes on the way past"
        );
        assert_eq!(
            board.piece_at(sq("d5")),
            None,
            "the captured pawn is gone from the board"
        );
        assert_eq!(board.piece_at(sq("d6")).map(|p| p.kind), Some(Kind::Pawn));
        assert_eq!(board.en_passant(), None, "and the right is spent");
    }

    #[test]
    fn a_pawn_only_may_be_taken_on_the_way_past_at_once() {
        let mut board = bare([("e1", Colour::White), ("e8", Colour::Black)]);
        board.place(sq("e5"), piece(Kind::Pawn, Colour::White));
        board.place(sq("d7"), piece(Kind::Pawn, Colour::Black));
        board.place(sq("a7"), piece(Kind::Pawn, Colour::Black));
        board.turn = Colour::Black;
        board.apply(Move::simple(sq("d7"), sq("d5"))).unwrap();
        board.turn = Colour::Black; // white declines, black returns
        board.apply(Move::simple(sq("a7"), sq("a6"))).unwrap();
        board.turn = Colour::White;
        assert_eq!(
            board.en_passant(),
            None,
            "the right lapsed with the move that declined it"
        );
        assert_eq!(
            board.apply(Move::simple(sq("e5"), sq("d6"))),
            Err(MoveError::BadGeometry)
        );
    }

    #[test]
    fn a_pawn_reaching_the_end_must_say_what_it_became() {
        let mut board = bare([("a1", Colour::White), ("h8", Colour::Black)]);
        board.place(sq("a7"), piece(Kind::Pawn, Colour::White));
        assert_eq!(
            board.apply(Move::simple(sq("a7"), sq("a8"))),
            Err(MoveError::NeedsPromotion)
        );
        assert_eq!(
            board.apply(Move {
                from: sq("a7"),
                to: sq("a8"),
                promotion: Some(Kind::King)
            }),
            Err(MoveError::BadPromotion),
            "not to a king"
        );
        let queen = Move {
            from: sq("a7"),
            to: sq("a8"),
            promotion: Some(Kind::Queen),
        };
        assert!(board.apply(queen).is_ok());
        assert_eq!(board.piece_at(sq("a8")).map(|p| p.kind), Some(Kind::Queen));
        assert_eq!(
            board.apply(Move {
                from: sq("a8"),
                to: sq("a7"),
                promotion: Some(Kind::Queen)
            }),
            Err(MoveError::WrongColour)
        );
        let mut board = bare([("a1", Colour::White), ("h8", Colour::Black)]);
        board.place(sq("a3"), piece(Kind::Pawn, Colour::White));
        assert_eq!(
            board.apply(Move {
                from: sq("a3"),
                to: sq("a4"),
                promotion: Some(Kind::Queen)
            }),
            Err(MoveError::BadPromotion),
            "a promotion where none is due"
        );
    }

    #[test]
    fn castling_needs_the_right_the_rook_and_an_empty_way() {
        let mut board = Board::start();
        board.squares[sq("f1") as usize] = None;
        board.squares[sq("g1") as usize] = None;
        assert!(
            board.apply(Move::simple(sq("e1"), sq("g1"))).is_ok(),
            "white castles out"
        );
        assert_eq!(board.piece_at(sq("g1")).map(|p| p.kind), Some(Kind::King));
        assert_eq!(
            board.piece_at(sq("f1")).map(|p| p.kind),
            Some(Kind::Rook),
            "the rook came over"
        );
        assert_eq!(
            board.castling() & (CASTLE_WHITE_KING | CASTLE_WHITE_QUEEN),
            0,
            "the right is spent"
        );

        let mut board = Board::start();
        board.squares[sq("g1") as usize] = Some(piece(Kind::Bishop, Colour::White));
        assert_eq!(
            board.apply(Move::simple(sq("e1"), sq("g1"))),
            Err(MoveError::Occupied),
            "the destination is taken"
        );

        let mut board = Board::start();
        board.squares[sq("f1") as usize] = Some(piece(Kind::Queen, Colour::White));
        board.squares[sq("g1") as usize] = None;
        assert_eq!(
            board.apply(Move::simple(sq("e1"), sq("g1"))),
            Err(MoveError::NoCastle),
            "the way over is blocked"
        );

        let mut board = Board::start();
        board.squares[sq("f1") as usize] = None;
        board.squares[sq("g1") as usize] = None;
        board.castling = CASTLE_BLACK_KING;
        assert_eq!(
            board.apply(Move::simple(sq("e1"), sq("g1"))),
            Err(MoveError::NoCastle),
            "the right was spent"
        );

        let mut board = Board::start();
        board.squares[sq("f1") as usize] = None;
        board.squares[sq("g1") as usize] = None;
        board.squares[sq("b1") as usize] = None;
        board.squares[sq("c1") as usize] = None;
        board.squares[sq("d1") as usize] = None;
        board.castling = CASTLE_WHITE_QUEEN;
        assert!(
            board.apply(Move::simple(sq("e1"), sq("c1"))).is_ok(),
            "and queen-side"
        );
        assert_eq!(board.piece_at(sq("d1")).map(|p| p.kind), Some(Kind::Rook));
    }

    #[test]
    fn a_king_may_not_castle_through_a_square_that_is_attacked() {
        let mut board = bare([("e8", Colour::Black), ("a1", Colour::White)]);
        board.place(sq("a8"), piece(Kind::Rook, Colour::Black));
        board.place(sq("h8"), piece(Kind::Rook, Colour::Black));
        board.place(sq("f1"), piece(Kind::Queen, Colour::White)); // looking down the f-file at f8
        board.castling = CASTLE_BLACK_KING | CASTLE_BLACK_QUEEN;
        board.turn = Colour::Black;
        assert_eq!(
            board.apply(Move::simple(sq("e8"), sq("g8"))),
            Err(MoveError::CastleIntoCheck),
            "through the queen's file"
        );
        assert!(
            board.apply(Move::simple(sq("e8"), sq("c8"))).is_ok(),
            "the other side is clear"
        );
    }

    #[test]
    fn a_refused_move_leaves_the_board_alone() {
        let before = Board::start();
        let mut board = before;
        assert!(board.apply(Move::simple(sq("e2"), sq("e5"))).is_err());
        assert_eq!(board.key(), before.key());
    }

    #[test]
    fn the_key_changes_when_any_part_of_the_position_changes() {
        let start = Board::start();
        let mut after = start;
        after.apply(Move::simple(sq("d2"), sq("d4"))).unwrap();
        assert_ne!(start.key(), after.key());
        let mut rights = start;
        rights.castling ^= CASTLE_WHITE_KING;
        assert_ne!(start.key(), rights.key());
        let mut turn = start;
        turn.turn = Colour::Black;
        assert_ne!(start.key(), turn.key());
        // An en-passant square is the position only when a pawn is standing to use it.
        let mut capturable = Board::empty();
        capturable.turn = Colour::White;
        capturable.place(
            sq("d5"),
            Piece {
                colour: Colour::Black,
                kind: Kind::Pawn,
            },
        );
        capturable.place(
            sq("e5"),
            Piece {
                colour: Colour::White,
                kind: Kind::Pawn,
            },
        );
        capturable.ep_field = Some(sq("d6"));
        let mut unused = capturable;
        unused.squares[36] = None;
        assert_ne!(capturable.key(), unused.key());
        let mut notated = unused;
        notated.ep_field = None;
        assert_eq!(unused.key(), notated.key());
    }

    #[test]
    fn a_key_is_a_fold_of_the_position_not_of_its_history() {
        // Two move orders to one position give one key, which is the whole point of the key.
        let mut one = Board::start();
        for (from, to) in [("e2", "e4"), ("e7", "e5"), ("b1", "c3"), ("b8", "c6")] {
            one.apply(Move::simple(sq(from), sq(to))).unwrap();
        }
        let mut two = Board::start();
        for (from, to) in [("b1", "c3"), ("b8", "c6"), ("e2", "e4"), ("e7", "e5")] {
            two.apply(Move::simple(sq(from), sq(to))).unwrap();
        }
        assert_eq!(one.key(), two.key(), "same position, same key");
        assert_ne!(one.key(), Board::start().key());
    }
}
