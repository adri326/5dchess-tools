use chess5dlib::{game::*, moves::*, moveset::*, resolve::*};
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

    // let moves = probable_moves(&game, board, &virtual_boards)
    //     .into_iter()
    //     .map(|m| {
    //         let (info, vboards) = m
    //             .generate_vboards(&game, &game.info, &virtual_boards)
    //             .unwrap();
    //         (m, info, vboards)
    //     })
    //     .collect::<Vec<_>>();
    // let moves = score_moves(&game, &virtual_boards, board, &lore, moves, &game.info);

    // println!("{:?}", lore);
    // println!(
    //     "{:#?}",
    //     moves
    //         .into_iter()
    //         .rev()
    //         .take(5)
    //         .map(|(m, _b, _i, s)| format!("{:?} : {}", m, s))
    //         .collect::<Vec<_>>()
    // );

    // let mut mv: Vec<Move> = Vec::new();
    // for board in get_own_boards(&game, &virtual_boards, &game.info) {
    //     let moves = probable_moves(&game, board, &virtual_boards)
    //         .into_iter()
    //         .map(|m| {
    //             let (info, vboards) = m
    //                 .generate_vboards(&game, &game.info, &virtual_boards)
    //                 .unwrap();
    //             (m, info, vboards)
    //         })
    //         .collect::<Vec<_>>();
    //     let mut moves = score_moves(&game, &virtual_boards, board, &lore, moves, &game.info);
    //     mv.push(moves.last().unwrap().0);
    // }
    // println!("{:?}", mv);
    // println!(
    //     "{:?}",
    //     score_moveset(
    //         &game,
    //         &virtual_boards,
    //         &game.info,
    //         get_opponent_boards(&game, &virtual_boards, &game.info).into_iter(),
    //         mv
    //     )
    // );

    let ranked_moves = get_own_boards(&game, &virtual_boards, &game.info)
        .into_iter()
        .map(|board| {
            let lore = generate_lore(
                &game,
                &virtual_boards,
                board,
                get_opponent_boards(&game, &virtual_boards, &game.info).into_iter(),
                &game.info,
            );
            let probables = probable_moves(&game, board, &virtual_boards)
                .into_iter()
                .map(|mv| {
                    let (new_info, new_vboards) = mv
                        .generate_vboards(&game, &game.info, &virtual_boards)
                        .unwrap();
                    (mv, new_info, new_vboards)
                })
                .collect::<Vec<_>>();
            score_moves(&game, &virtual_boards, board, &lore, probables, &game.info)
        })
        .collect::<Vec<_>>();

    let mut movesets = generate_movesets(&game, &virtual_boards, &game.info, ranked_moves)
        .map(|ms| {
            score_moveset(
                &game,
                &virtual_boards,
                &game.info,
                get_opponent_boards(&game, &virtual_boards, &game.info).into_iter(),
                ms,
            )
        })
        .filter(|x| x.is_some())
        .map(|x| x.unwrap())
        .take(40)
        .collect::<Vec<_>>();

    if game.info.active_player {
        movesets.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
    } else {
        movesets.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap());
    }

    for moveset in movesets.iter().take(3) {
        println!("{:?}: {}", moveset.0, moveset.3);
        for b in &moveset.1 {
            println!("{}", b);
            println!("");
        }
    }

    Ok(())
}
