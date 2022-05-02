use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use cosmwasm_std::Addr;
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
pub const ADDRESS_AND_DAY_TO_CHOICE: Map<String, String> = Map::new("address_and_day_to_choice");
pub const DEPLOYING_BLOCK_TIMESTAMP: Item<u64> = Item::new("deploying_block_timestamp");
pub const BLOCK_AND_DAY_TO_WINNER_COUNT: Map<(u128, u128), u128> = Map::new("block_and_day_to_winner_count");
pub const ADDRESS_AND_DAY_TO_BLOCK_WON: Map<(u128,u128), u128> = Map::new("address_and_day_to_block_won");
pub const BLOCK_AND_DAY_TO_DEPOSIT: Map<(u128,u128), u128> = Map::new("address_and_day_to_deposit");
