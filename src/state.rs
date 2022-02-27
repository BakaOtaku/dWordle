use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WordDictionaryInfo {
    pub word_list: HashSet<String>,
    pub day: u64,
    pub day_word: String,
}

pub const WORD_DICTIONARY: Item<WordDictionaryInfo> = Item::new("word_dictionary");
pub const ADDR_AND_DAY_TO_CHOICE: Map<String, String> = Map::new("add_and_day_to_choice");
pub const DEPLOYING_BLOCK_TIMESTAMP: Item<u64> = Item::new("deploying_block_timestamp");
