use chess5dlib::prelude::*;

#[test]
fn new_blank() {
    assert!(Tile::new_blank(None).is_blank());
    assert!(!Tile::new_blank(None).is_void());
    assert!(Tile::new_blank(None) == Tile::Blank);
    assert!(match Tile::new_blank(None) {
        Tile::Blank => true,
        _ => false
    });
    assert!(!Tile::new_blank(Some(Piece::new(PieceKind::Pawn, true, false))).is_blank());
    assert!(!Tile::new_blank(Some(Piece::new(PieceKind::Pawn, true, false))).is_void());
    assert!(!Tile::new_blank(Some(Piece::new(PieceKind::Pawn, true, false))).is_empty());
    assert!(Tile::new_blank(Some(Piece::new(PieceKind::Pawn, true, false))).is_piece_of_color(true));
    assert!(Tile::new_blank(Some(Piece::new(PieceKind::Pawn, true, false))).piece().is_some());
}

#[test]
fn new_void() {
    assert!(Tile::new_void(None).is_void());
    assert!(!Tile::new_void(None).is_blank());
    assert!(Tile::new_void(None) == Tile::Void);
    assert!(match Tile::new_void(None) {
        Tile::Void => true,
        _ => false
    });
    assert!(!Tile::new_void(Some(Piece::new(PieceKind::Pawn, true, false))).is_blank());
    assert!(!Tile::new_void(Some(Piece::new(PieceKind::Pawn, true, false))).is_void());
    assert!(!Tile::new_void(Some(Piece::new(PieceKind::Pawn, true, false))).is_empty());
    assert!(Tile::new_void(Some(Piece::new(PieceKind::Pawn, true, false))).is_piece_of_color(true));
    assert!(Tile::new_void(Some(Piece::new(PieceKind::Pawn, true, false))).piece().is_some());
}

#[test]
fn from() {
    let piece = Piece::new(PieceKind::Pawn, true, false);
    assert_eq!(Tile::from(piece), Tile::Piece(piece));
    assert_eq!(Tile::from(Some(Tile::Piece(piece))), Tile::Piece(piece));
    assert_eq!(Tile::from(None), Tile::Void);
}

#[test]
fn into() {
    let tile = Tile::Piece(Piece::new(PieceKind::Pawn, true, false));
    assert!(Option::<Piece>::from(tile) == Some(Piece::new(PieceKind::Pawn, true, false)));
    assert!(Option::<Piece>::from(Tile::Void) == None);
    assert!(Option::<Piece>::from(Tile::Blank) == None);
}
