use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WordDictionaryInfo {
    pub word_list: Vec<String>,
    pub word_dictionary: HashSet<String>,
    pub day: u64,
    pub day_words: [String;3],
    pub index_word: usize
}

pub const WORD_DICTIONARY: Item<WordDictionaryInfo> = Item::new("word_dictionary");
pub const ADDRESS_AND_DAY_TO_CHOICE: Map<(&Addr, u64), String> = Map::new("address_and_day_to_choice");
pub const DEPLOYING_BLOCK_TIMESTAMP: Item<u64> = Item::new("deploying_block_timestamp");
pub const DAY_AND_BLOCK_TO_WINNER_COUNT: Map<(u64, u64), u64> = Map::new("day_and_block_to_winner_count");
pub const DAY_AND_ADDRESS_TO_BLOCK_WON: Map<(u64, &Addr), u64> = Map::new("day_and_address_to_block_won");
pub const DAY_AND_BLOCK_TO_DEPOSIT: Map<(u64, u64), Uint128> = Map::new("day_and_block_to_deposit");
