const fs = require("fs");
const Chess = require("5d-chess-js");

let contents = fs.readFileSync("/tmp/db/curr.5dpgn", "utf8");

try { fs.mkdirSync('./db'); } catch(err){}
try { fs.mkdirSync('./db/standard'); } catch(err){}
try { fs.mkdirSync('./db/standard/white'); } catch(err){}
try { fs.mkdirSync('./db/standard/black'); } catch(err){}
try { fs.mkdirSync('./db/standard/stalemate'); } catch(err){}
try { fs.mkdirSync('./db/standard/white_timeout'); } catch(err){}
try { fs.mkdirSync('./db/standard/black_timeout'); } catch(err){}
try { fs.mkdirSync('./db/standard/stalemate_timeout'); } catch(err){}
try { fs.mkdirSync('./db/standard/none'); } catch(err){}

let [path, ...raw] = contents.split("\n");
raw = raw.join("\n");

let match_path = /^\/tmp\/db\/(\w+)\/(\w+)\/[0-9]+-([0-9]+)\.5dpgn$/.exec(path);
if (match_path !== null) {
    let game = new Chess(undefined, "standard");
    let headers = raw.split("\n").filter(l => l.startsWith("[")).join("\n");
    let moves = raw.split("\n").filter(l => !l.startsWith("[")).join("\n");

    if (moves.trim() !== "") {
        game.import(moves);
        game.metadata.hash = game.hash;

        let export_path = `./db/${match_path[1]}/${match_path[2]}/${game.metadata.hash}-${match_path[3]}.5dpgn`;

        let exported = game.export().split("\n").filter(l => l.trim() != `[Mode "5D"]` && l.trim() != `[Board "Standard"]`).join("\n");
        fs.writeFileSync(export_path, headers + "\n" + exported);
    }
} else {
    console.error("Unmatched path!");
}
