#![no_std]

pub mod pokedex {
    use minicbor::{Encode, Decode, bytes::ByteArray};

    #[derive(Encode, Decode, Debug)]
    pub struct Pokemon {
        #[n(0)] pub id: u8,
        #[n(1)] pub name: [u8; 12],
        #[n(2)] pub type_primary: Type,
        #[n(3)] pub type_secondary: Option<Type>,

        #[n(4)] pub capture_rate: u8,
        #[n(5)] pub base_experience: u8,

        #[n(7)] pub hp: StatData,
        #[n(8)] pub attack: StatData,
        #[n(9)] pub defense: StatData,
        #[n(10)] pub special_attack: StatData,
        #[n(11)] pub special_defense: StatData,
        #[n(12)] pub speed: StatData,

        #[n(13)] pub sprite: ByteArray<578>,
    }

    #[derive(Encode, Decode, Debug)]
    #[cbor(index_only)]
    pub enum Stat {
        #[n(0)] Hp,
        #[n(1)] Attack,
        #[n(2)] Defense,
        #[n(3)] SpecialAttack,
        #[n(4)] SpecialDefense,
        #[n(5)] Speed,
    }

    #[derive(Encode, Decode, Debug, Clone, Copy)]
    pub struct StatData {
        #[n(1)] pub base_value: u16,
    }

    // These could be created dynamically from the JSON data. This isn't so bad
    // though so /shruggie.
    #[derive(Encode, Decode, Debug, Clone, Copy)]
    #[cbor(index_only)]
    pub enum Type {
        #[n(0)] Normal,
        #[n(1)] Fighting,
        #[n(2)] Flying,
        #[n(3)] Poison,
        #[n(4)] Ground,
        #[n(5)] Rock,
        #[n(6)] Bug,
        #[n(7)] Ghost,
        #[n(8)] Steel,
        #[n(9)] Fire,
        #[n(10)] Water,
        #[n(11)] Grass,
        #[n(12)] Electric,
        #[n(13)] Psychic,
        #[n(14)] Ice,
        #[n(15)] Dragon,
        #[n(16)] Dark,
        #[n(17)] Fairy,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct MoveListChunk {
        #[n(0)] pub is_final_chunk: bool,
        #[n(1)] pub moves: [Option<LearnableMove>; 16],
    }

    #[derive(Encode, Decode, Debug, Copy, Clone)]
    pub struct LearnableMove {
        #[n(0)] pub id: u16,
        #[n(1)] pub condition: LearnCondition,
    }

    #[derive(Encode, Decode, Debug, Copy, Clone)]
    pub enum LearnCondition {
        #[n(0)] LevelUp {
            #[n(0)] level: u8
        },
        #[n(1)] Machine,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct Move {
        #[n(0)] pub id: u16,
        #[n(1)] pub name: [char; 12],

        #[n(2)] pub type_: Type,
        #[n(3)] pub damage_class: DamageClass,
        #[n(4)] pub target: Target,

        #[n(5)] pub accuracy: u8,
        #[n(6)] pub power: u8,
        #[n(7)] pub pp: u8,
        #[n(8)] pub priority: u8,

        #[n(9)] pub parameters: Parameters,
        #[n(10)] pub stat_changes: [Option<StatChange>; 2]
    }

    #[derive(Encode, Decode, Debug)]
    #[cbor(map)]
    pub struct Parameters {
        #[n(0)] pub ailment: Option<AilmentParameter>,
        #[n(1)] pub crit_rate: u8,
        #[n(2)] pub drain: u8,
        #[n(3)] pub flinch_chance: u8,
        #[n(4)] pub healing: u8,
        #[n(5)] pub stat_chance: u8,
        #[n(6)] pub turn_range: Option<ParameterRange>,
        #[n(7)] pub hit_range: Option<ParameterRange>,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct AilmentParameter {
        #[n(0)] pub ailment: AilmentType,
        #[n(1)] pub chance: u8,
    }

    #[derive(Encode, Decode, Debug)]
    #[cbor(index_only)]
    pub enum AilmentType {
        #[n(0)] Burn,
        #[n(1)] Confusion,
        #[n(2)] Disable,
        #[n(3)] Freeze,
        #[n(4)] LeechSeed,
        #[n(5)] Paralysis,
        #[n(6)] Poison,
        #[n(7)] Sleep,
        #[n(8)] Trap,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct ParameterRange {
        #[n(0)] pub min: u8,
        #[n(1)] pub max: u8,
    }

    #[derive(Encode, Decode, Debug)]
    #[cbor(index_only)]
    pub enum DamageClass {
        #[n(0)] Physical,
        #[n(1)] Special,
        #[n(2)] Status,
    }

    #[derive(Encode, Decode, Debug)]
    #[cbor(index_only)]
    pub enum Target {
        #[n(0)] AllOpponents,
        #[n(1)] AllOtherPokemon,
        #[n(2)] EntireField,
        #[n(3)] RandomOpponent,
        #[n(4)] SelectedPokemon,
        #[n(5)] SpecificMove,
        #[n(6)] User,
        #[n(7)] UserField,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct StatChange {
        #[n(0)] pub amount: u8,
        #[n(1)] pub stat: Stat,
    }
}
