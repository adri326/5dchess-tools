use chess5dlib::prelude::*;

pub fn test_properties(
    kind: PieceKind,
    (royal, castle, castle_to, promote, enpassant, kickstart): (bool, bool, bool, bool, bool, bool),
) {
    let piece = Piece::new(kind, true, true);
    assert!(royal == piece.is_royal());
    assert!(castle == piece.can_castle());
    assert!(castle_to == piece.can_castle_to());
    assert!(promote == piece.can_promote());
    assert!(enpassant == piece.can_enpassant());
    assert!(kickstart == piece.can_kickstart());
}

#[test]
pub fn test_piece_properties() {
    test_properties(PieceKind::Pawn, (false, false, false, true, true, true));
    test_properties(PieceKind::Brawn, (false, false, false, true, true, true));
    test_properties(
        PieceKind::Knight,
        (false, false, false, false, false, false),
    );
    test_properties(
        PieceKind::Bishop,
        (false, false, false, false, false, false),
    );
    test_properties(PieceKind::Rook, (false, false, true, false, false, false));
    test_properties(PieceKind::Queen, (false, false, false, false, false, false));
    test_properties(
        PieceKind::Princess,
        (false, false, false, false, false, false),
    );
    test_properties(PieceKind::King, (true, true, false, false, false, false));
    test_properties(
        PieceKind::Unicorn,
        (false, false, false, false, false, false),
    );
    test_properties(
        PieceKind::Dragon,
        (false, false, false, false, false, false),
    );
    test_properties(
        PieceKind::CommonKing,
        (false, false, false, false, false, false),
    );
    test_properties(
        PieceKind::RoyalQueen,
        (true, false, false, false, false, false),
    );
}
