#![no_std]

pub mod pokedex {
    use minicbor::{Encode, Decode, bytes::ByteArray};
    
    #[derive(Encode, Decode, Debug)]
    pub struct Pokemon {
        #[n(0)] pub species_id: u8,
        #[n(1)] pub nickname: Option<[u8; 12]>,

        #[n(2)] pub level: u8,
        #[n(3)] pub xp: u16,
        #[n(5)] pub current_hp: u16,

        #[n(6)] pub hp: StatData,
        #[n(7)] pub attack: StatData,
        #[n(8)] pub defense: StatData,
        #[n(9)] pub special_attack: StatData,
        #[n(10)] pub special_defense: StatData,
        #[n(11)] pub speed: StatData,

        #[n(12)] pub moves: [Option<u16>; 4],
    }

    #[derive(Encode, Decode, Debug, Clone, Copy)]
    pub struct StatData {
        #[n(0)] pub value: u16,
        #[n(2)] pub effort_value: u16,
        #[n(3)] pub individual_value: u8,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct PokemonSpecies {
        #[n(0)] pub id: u8,
        #[n(1)] pub name: [u8; 12],
        #[n(2)] pub type_primary: Type,
        #[n(3)] pub type_secondary: Option<Type>,
        #[n(14)] pub growth_rate: GrowthRate,

        #[n(4)] pub capture_rate: u8,
        #[n(5)] pub base_experience: u8,

        #[n(7)] pub hp: SpeciesStatData,
        #[n(8)] pub attack: SpeciesStatData,
        #[n(9)] pub defense: SpeciesStatData,
        #[n(10)] pub special_attack: SpeciesStatData,
        #[n(11)] pub special_defense: SpeciesStatData,
        #[n(12)] pub speed: SpeciesStatData,

        #[n(13)] pub sprite: ByteArray<578>,
    }

    #[derive(Encode, Decode, Debug)]
    #[cbor(index_only)]
    pub enum Stats {
        #[n(0)] Hp,
        #[n(1)] Attack,
        #[n(2)] Defense,
        #[n(3)] SpecialAttack,
        #[n(4)] SpecialDefense,
        #[n(5)] Speed,
    }

    #[derive(Encode, Decode, Debug, Clone, Copy)]
    pub struct SpeciesStatData {
        #[n(0)] pub base_value: u16,
        #[n(1)] pub effort_value_yield: u8,
    }

    #[derive(Encode, Decode, Debug, Clone, Copy)]
    pub enum GrowthRate {
        #[n(0)] ERRATIC,
        #[n(1)] FAST,
        #[n(2)] MEDIUM_FAST,
        #[n(3)] MEDIUM_SLOW,
        #[n(4)] SLOW,
        #[n(5)] FLUCTUATING
    }

    impl GrowthRate {
        fn xp_for_level(&self, level: u8) {
            match self {
                GrowthRate::ERRATIC => todo!(),
                GrowthRate::FAST => todo!(),
                GrowthRate::MEDIUM_FAST => todo!(),
                GrowthRate::MEDIUM_SLOW => todo!(),
                GrowthRate::SLOW => todo!(),
                GrowthRate::FLUCTUATING => todo!(),
            }
        }
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

    impl Type {
        pub fn name(&self) -> &'static str {
            match self {
                Type::Normal => "NORMAL",
                Type::Fighting => "FIGHTING",
                Type::Flying => "FLYING",
                Type::Poison => "POISON",
                Type::Ground => "GROUND",
                Type::Rock => "ROCK",
                Type::Bug => "BUG",
                Type::Ghost => "GHOST",
                Type::Steel => "STEEL",
                Type::Fire => "FIRE",
                Type::Water => "WATER",
                Type::Grass => "GRASS",
                Type::Electric => "ELECTRIC",
                Type::Psychic => "PSYCHIC",
                Type::Ice => "ICE",
                Type::Dragon => "DRAGON",
                Type::Dark => "DARK",
                Type::Fairy => "FAIRY"
            }
        }
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
        #[n(0)] LevelUp(#[n(0)] u8),
        #[n(1)] Machine,
    }

    #[derive(Encode, Decode, Debug)]
    pub struct Move {
        #[n(0)] pub id: u16,
        #[n(1)] pub name: [u8; 12],

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
        #[n(1)] pub stat: Stats,
    }
}

pub mod generation {
    use crate::pokedex::{Pokemon, PokemonSpecies, StatData, SpeciesStatData, StatChange, LearnableMove, LearnCondition};

    trait Random {
        fn random(&self) -> u8;

        fn generate_iv(&self) -> u8 {
            return self.random() & 0b00011111;
        }
    }

    pub fn generate_pokemon(species: PokemonSpecies, move_list: &mut dyn Iterator<Item = &LearnableMove>, level: u8, rng: &dyn Random) -> Pokemon {
        let mut moves = [None; 4];
        let mut i = 0;
        for m in move_list {
            if let LearnCondition::LevelUp(lvl) = m.condition {
                if lvl > level {
                    break;
                }
                moves[i] = Some(m.id);
                i = (i + 1) % 4;
            }
        }

        let hp_iv = rng.generate_iv();
        let hp = StatData {
            value: calculate_hp_stat(species.hp.base_value, 0, hp_iv, level),
            effort_value: 0,
            individual_value: hp_iv
        };
        return Pokemon{
            species_id: species.id,
            nickname: None,
            level,
            xp: species.growth_rate.xp_for_level(level),
            current_hp: hp.value,
            hp,
            attack: new_stat(species.attack, level, rng),
            defense: new_stat(species.defense, level, rng),
            special_attack: new_stat(species.special_attack, level, rng),
            special_defense: new_stat(species.special_defense, level, rng),
            speed: new_stat(species.speed, level, rng),
            moves
        }
    }

    fn new_stat(stat: SpeciesStatData, level: u8, rng: &dyn Random) -> StatData {
        let iv = rng.generate_iv();
        return StatData { value: calculate_stat(stat.base_value, 0, iv, level), effort_value: 0, individual_value: iv }
    }

    fn calculate_stat(base: u16, ev: u16, iv: u8, level: u8) -> u16 {
        return (((2 * base) + (iv as u16) + ev) * (level as u16) / 100) + 5
    }

    fn calculate_hp_stat(base: u16, ev: u16, iv: u8, level: u8) -> u16 {
        return (((2 * base) + (iv as u16) + ev) * (level as u16) / 100) + (level as u16) + 10
    }
}
