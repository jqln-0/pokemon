use std::{env, fs::File};
use std::io::{prelude::*, BufReader, BufWriter};
use minicbor::bytes::ByteArray;
use minicbor_io::Writer;
use pokemon::pokedex::{self, Pokemon, Type, Stats};
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("usage: {} <path-to-pokemon> <output>", args[0]);
        return;
    }

    println!("you provided {}", args[1]);

    let mut bmp_file = File::open(format!("{}.bmp", args[1])).unwrap();
    let mut buffer = [0u8; 578];
    bmp_file.read_exact(&mut buffer).unwrap();

    let json_file = File::open(format!("{}.json", args[1])).unwrap();
    let json: Value = serde_json::from_reader(BufReader::new(json_file)).unwrap();

    let pokemon = parse_pokemon(json, &buffer).unwrap();

    println!("created pokemon {:?}", pokemon);

    let output_file = BufWriter::new(File::create(&args[2]).unwrap());
    let mut cbor_writer = Writer::new(output_file);
    let outsize = cbor_writer.write(pokemon).unwrap();

    println!("wrote to file. final size was {} bytes", outsize);
}

fn parse_pokemon(json: Value, bitmap: &[u8; 578]) -> Result<Pokemon, &'static str> {
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

    let stats = match json["stats"].as_array() {
        Some(all_stats) => all_stats.iter().flat_map(|j| parse_as_stats(j)).collect(),
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
        stats,
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

fn parse_as_stats(json: &Value) -> Option<Stats> {
    Some(Stats {
        stat: match json["stat"].as_str() {
            Some(s) => match s {
                "hp" => Stat::Hp,
                "attack" => Stat::Attack,
                "defense" => Stat::Defense,
                "special-attack" => Stat::SpecialAttack,
                "special-defense" => Stat::SpecialDefense,
                "speed" => Stat::Speed,
                _ => return None
            }
            None => return None
        },
        base_value: match json["base_state"] {
            Some(i) => i,
            None => return None
        }
    })
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