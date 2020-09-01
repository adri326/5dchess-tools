use chess5dlib::{game::*, moves::*, resolve::*};
use std::env;
use std::fs::File;
use std::io::prelude::*;
extern crate json;

fn main() -> std::io::Result<()> {
    let path = env::args().last().unwrap();

    let mut file = File::open(path)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let game = Game::from(json::parse(&contents).expect("Couldn't parse JSON"));

    // for boards in probable_moves(&game, game.get_last_board(0.0).unwrap(), &vec![])
    //     .into_iter()
    //     .map(|m| m.generate_vboards(&game, &game.info, &vec![]).unwrap().1)
    // {
    //     println!("{}\n", boards[0]);
    // }

    // println!("{:#?}", probable_moves(&game, game.get_last_board(-1.0).unwrap(), &vec![]));

    let virtual_boards: Vec<Board> = vec![];

    let opponent_boards = get_opponent_boards(&game, &virtual_boards, &game.info);
    let board = game.get_last_board(0.0).unwrap();

    let lore = generate_lore(
        &game,
        &virtual_boards,
        board,
        opponent_boards.into_iter(),
        &game.info,
    );

    let moves = probable_moves(&game, board, &virtual_boards)
        .into_iter()
        .map(|m| {
            let (info, vboards) = m
                .generate_vboards(&game, &game.info, &virtual_boards)
                .unwrap();
            (m, info, vboards)
        })
        .collect::<Vec<_>>();
    let moves = score_moves(&game, &virtual_boards, board, &lore, moves, &game.info);

    println!("{:?}", lore);
    println!(
        "{:#?}",
        moves
            .into_iter()
            .rev()
            .take(5)
            .map(|(m, _b, _i, s)| format!("{:?} : {}", m, s))
            .collect::<Vec<_>>()
    );

    let mut mv: Vec<Move> = Vec::new();
    for board in get_own_boards(&game, &virtual_boards, &game.info) {
        let moves = probable_moves(&game, board, &virtual_boards)
            .into_iter()
            .map(|m| {
                let (info, vboards) = m
                    .generate_vboards(&game, &game.info, &virtual_boards)
                    .unwrap();
                (m, info, vboards)
            })
            .collect::<Vec<_>>();
        let mut moves = score_moves(&game, &virtual_boards, board, &lore, moves, &game.info);
        mv.push(moves.last().unwrap().0);
    }
    println!("{:?}", mv);
    println!(
        "{:?}",
        score_moveset(
            &game,
            &virtual_boards,
            &game.info,
            get_opponent_boards(&game, &virtual_boards, &game.info).into_iter(),
            mv
        )
    );

    // let legal = legal_movesets(&game, &vec![], &game.info);
    // println!("{:#?}", legal);
    // for (_m, i, _b) in legal {
    //     println!("{:#?}", i);
    //     // let responses = legal_movesets(&game, &b, &i);
    //     // println!("{:#?}", responses);
    //     // println!("{:#?}", responses.iter().take(1).map(|(_m, i, b)| legal_movesets(&game, b, i).len()).collect::<Vec<_>>());
    //     break;
    // }

    Ok(())
}
