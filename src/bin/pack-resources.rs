use std::collections::HashMap;
use std::fs::read_dir;
use std::{env, fs::File};
use std::io::{prelude::*, BufReader, BufWriter};
use minicbor::bytes::ByteArray;
use minicbor_io::Writer;
use pokemon::pokedex::{self, Pokemon, Type, StatData, MoveListChunk, Move, LearnableMove, LearnCondition};
use serde_json::Value;

#[derive(Debug)]
struct ParseError(String);

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("usage: {} <path-to-pokemon-dir> <output-file>", args[0]);
        return;
    }

    let output_file = BufWriter::new(File::create(&args[2]).unwrap());
    let mut cbor_writer = Writer::new(output_file);

    let files = read_dir(&args[1]).unwrap();
    for f in files {
        let path = match f.ok().map(|f| f.path()) {
            Some(p) => p,
            None => continue
        };
        let json_filename = match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => path,
            _ => continue
        };
        let bmp_filename = json_filename.with_extension("bmp");

        println!("processing {:?} and {:?}", json_filename, bmp_filename);
        let mut bmp_file = File::open(bmp_filename).unwrap();
        let mut buffer = [0u8; 578];
        bmp_file.read_exact(&mut buffer).unwrap();

        let json_file = File::open(json_filename).unwrap();
        let json: Value = serde_json::from_reader(BufReader::new(json_file)).unwrap();

        let pokemon = parse_pokemon(&json, &buffer).unwrap();
        let move_list = parse_movelist(&json["moves"]).unwrap();

        cbor_writer.write(pokemon).unwrap();
        cbor_writer.write(move_list).unwrap();
    }

    println!("wrote all pokemon!");
}

fn parse_movelist(json: &Value) -> Result<Vec<MoveListChunk>, ParseError> {
    let mut all_chunks = Vec::new();
    let mut current_chunk: [Option<LearnableMove>; 16] = [None; 16];
    let mut current_chunk_i = 0;
    for m in json.as_array().unwrap_or(&Vec::new()) {
        let parsed = match parse_learnable_move(m) {
            Ok(p) => p,
            Err(e) => return Err(ParseError(format!("failed to parse move: {:?}", e)))
        };
        for i in parsed {
            if current_chunk_i == 16 {
                all_chunks.push(MoveListChunk{
                    is_final_chunk: false,
                    moves: current_chunk
                });
                current_chunk = [None; 16];
                current_chunk_i = 0;
            }

            current_chunk[current_chunk_i] = Some(i);
            current_chunk_i += 1;
        }
    }

    // Double check that we actually got results
    if current_chunk_i == 0 && all_chunks.len() == 0 {
        return Err(ParseError("no moves in movelist".to_string()));
    }

    // Finish the final chunk.
    all_chunks.push(MoveListChunk{
        is_final_chunk: true,
        moves: current_chunk
    });

    return Ok(all_chunks);
}

fn parse_learnable_move(json: &Value) -> Result<Vec<LearnableMove>, ParseError> {
    let parts: Vec<&str> = match json["move"]["url"].as_str() {
        Some(i) => i.split("/").collect(),
        None => return Err(ParseError("missing move id".to_string()))
    };
    let id: u16 = match parts[parts.len() - 2].parse() {
        Ok(i) => i,
        Err(e) => return Err(ParseError(format!("failed to parse id: {}", e)))
    };
    let mut moves = Vec::new();
    for method in json["version_group_details"].as_array().unwrap_or(&Vec::new()) {
        moves.push(LearnableMove{
            id,
            condition: match method["move_learn_method"].as_str() {
                Some("level-up") => LearnCondition::LevelUp{
                    level: match method["level_learned_at"].as_u64().map(|i| u8::try_from(i).ok()).flatten() {
                        Some(l) => l,
                        None => return Err(ParseError("missing or malformed level up method".to_string()))
                    }
                },
                Some("machine") => LearnCondition::Machine,
                Some(m) => return Err(ParseError(format!("unknown learn method {}", m))),
                None => return Err(ParseError("missing learn method".to_string()))
            }
        })
    }
    return match moves.len() {
        0 => Err(ParseError("move had no learn methods".to_string())),
        _ => Ok(moves)
    }
}

fn parse_move(json: Value) -> Result<Move, &'static str> {
    return Err("uh oh");
}

fn parse_pokemon(json: &Value, bitmap: &[u8; 578]) -> Result<Pokemon, &'static str> {
    let name = match json["name"].as_str() {
        Some(n) => {
            let mut i = 0;
            let mut name = ['\0'; 12];
            for c in n.chars() {
                name[i] = c;
                i += 1;
            }
            name
        },
        None => return Err("name was missing")
    };

    let mut types = [Option::None, Option::None];
    match json["types"].as_array() {
        Some(raw_types) => {
            for t in raw_types {
                let slot = match t["slot"].as_i64() {
                    Some(1) => 0,
                    Some(2) => 1,
                    None => return Err("missing type slot"),
                    _ => return Err("invalid type slot")
                };
                types[slot] = match t["type"].as_str() {
                    Some(val) => {
                        match type_from_string(val) {
                            Ok(t) => Some(t),
                            Err(_) => return Err("failed to parse type")
                        }
                    }
                    None => None
                }
            }
        }
        None => return Err("no types found")
    }

    let stats: HashMap<_, _> = match json["stats"].as_array() {
        Some(all_stats) => all_stats.iter().flat_map(|j| parse_as_stat_data(j)).collect(),
        None => return Err("missing stats")
    };

     Ok(pokedex::Pokemon {
        id: match parse_as_u8(&json["id"]) {
            Some(i) => i,
            None => return Err("missing id")
        },
        name,
        type_primary: match types[0] {
            Some(t) => t,
            None => return Err("no primary type given")
        },
        type_secondary: types[1],
        capture_rate: match parse_as_u8(&json["species"]["capture_rate"]) {
            Some(i) => i,
            None => return Err("missing capture rate")
        },
        base_experience: match parse_as_u8(&json["id"]) {
            Some(i) => i,
            None => return Err("missing base experience")
        },
        hp: stats["hp"],
        attack: stats["attack"],
        defense: stats["defense"],
        special_attack: stats["special-attack"],
        special_defense: stats["special-defense"],
        speed: stats["speed"],
        sprite: ByteArray::from(*bitmap),
    })
}

fn parse_as_u8(json: &Value) -> Option<u8> {
        return match json.as_u64().map(|i| u8::try_from(i)) {
            Some(i) => {
                match i {
                    Ok(i) => Some(i),
                    Err(_) => None
                }
            },
            None => None
        }
}

fn parse_as_stat_data(json: &Value) -> Option<(&str, StatData)> {
    Some((
        match json["stat"].as_str() {
            Some(i) => i,
            None => return None
        },
        StatData {
            base_value: match json["base_stat"].as_u64().map(|i| u16::try_from(i)) {
                Some(i) => {
                    match i {
                        Ok(i) => i,
                        Err(_) => return None
                    }
                },
                None => return None
            }
        }))
}

fn type_from_string(s: &str) -> Result<Type, &str> {
    match s {
        "normal" => Ok(Type::Normal),
        "fighting" => Ok(Type::Fighting),
        "flying" => Ok(Type::Flying),
        "poison" => Ok(Type::Poison),
        "ground" => Ok(Type::Ground),
        "rock" => Ok(Type::Rock),
        "bug" => Ok(Type::Bug),
        "ghost" => Ok(Type::Ghost),
        "steel" => Ok(Type::Steel),
        "fire" => Ok(Type::Fire),
        "water" => Ok(Type::Water),
        "grass" => Ok(Type::Grass),
        "electric" => Ok(Type::Electric),
        "psychic" => Ok(Type::Psychic),
        "ice" => Ok(Type::Ice),
        "dragon" => Ok(Type::Dragon),
        "dark" => Ok(Type::Dark),
        "fairy" => Ok(Type::Fairy),
        _ => Err("unknown type")
    }
}