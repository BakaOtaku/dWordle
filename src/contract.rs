#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
// use cw2::set_contract_version;
use std::borrow::BorrowMut;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Add;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADDRESS_AND_DAY_TO_CHOICE, DEPLOYING_BLOCK_TIMESTAMP, WORD_DICTIONARY};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:{{project-name}}";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    DEPLOYING_BLOCK_TIMESTAMP.save(deps.storage, &_env.block.time.seconds())?;
    Ok(Response::default().add_attribute("method","instantiated"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::MakeGuess { word } => execute_make_guess(deps, env, info, word),
        ExecuteMsg::InsertWordInDictionary { words_list } => {
            execute_insert_word_dictionary(deps, env, info, words_list)
        }
    }
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn vec_to_set(vec: Vec<String>) -> HashSet<String> {
    HashSet::from_iter(vec)
}

pub fn execute_insert_word_dictionary(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    words_list: Vec<String>,
) -> Result<Response, ContractError> {
    let mut dictionary = WORD_DICTIONARY.load(deps.storage)?.clone();
    dictionary.word_list.extend( words_list.clone());
    dictionary.word_dictionary.extend(vec_to_set(words_list));
    WORD_DICTIONARY.save(deps.storage, &dictionary)?;
    Ok(Response::new().add_attribute("method","word inserted"))
}

pub fn update_today_word_and_return(
    deps: &mut DepsMut,
    env: Env,
) -> Result<([String;3], u64, HashSet<String>), ContractError> {
    let mut dictionary = WORD_DICTIONARY.load(deps.storage)?.clone();
    let deploy_time = DEPLOYING_BLOCK_TIMESTAMP.load(deps.storage)?;
    let current_day = (env.block.time.seconds() - deploy_time) / 86400;
    if dictionary.day < current_day {

        dictionary.day_words[0] = dictionary.word_list[dictionary.index_word].clone();
        dictionary.day_words[1] = dictionary.word_list[dictionary.index_word+1].clone();
        dictionary.day_words[2] = dictionary.word_list[dictionary.index_word+2].clone();
        dictionary.index_word = dictionary.index_word+3;

        dictionary.day = current_day;
        WORD_DICTIONARY.save(deps.storage, &dictionary)?;
    }

    return Ok((dictionary.day_words, current_day, dictionary.word_dictionary));
}

#[derive(Hash)]
struct SenderNDay {
    day: u64,
    msg_sender: String,
}


pub fn execute_make_guess(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    word: String,
) -> Result<Response, ContractError> {
    if word.len() != 5 {
        return Err(ContractError::WrongGuess {});
    }

    // chargeTokenForAttempt

    let (today_word_list, current_day, word_dictionary) =
        update_today_word_and_return(deps.borrow_mut(), env)?;
    if word_dictionary.contains(&word) {
        return Err(ContractError::WrongGuess {});
    }
    let sender = info.sender.to_string();

    let calc_hash = SenderNDay{day: current_day,msg_sender: sender};

    let this_users_hash = calculate_hash( &calc_hash) as usize/3;
    let this_users_word = today_word_list[this_users_hash].clone();

    let mut choice = "".to_string();
    for (i, ch) in word.chars().enumerate() {
        let pos = this_users_word.find(ch);
        match pos {
            None => {
                choice = choice.clone().add("B");
            }
            Some(position) => {
                if position == i {
                    choice = choice.clone().add("G");
                } else {
                    choice = choice.clone().add("Y");
                }
            }
        }
    }
    let mut user_and_day = current_day.to_string();
    user_and_day.push_str(info.sender.as_str());
    let mut choice_store = ADDRESS_AND_DAY_TO_CHOICE.load(deps.storage, user_and_day.clone())?;
    choice_store.push_str(choice.as_str());
    ADDRESS_AND_DAY_TO_CHOICE.save(deps.storage, user_and_day, &choice_store)?;
    Ok(Response::new().add_attribute("choice",choice.as_str()))
}

pub fn claim(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    timestamp: u64
) -> Result<Response, ContractError> {
    let deploy_time = DEPLOYING_BLOCK_TIMESTAMP.load(deps.storage)?;
    let day = (timestamp - deploy_time) / 86400;
    let mut user_and_day = day.to_string();
    user_and_day.push_str(info.sender.as_str());
    let mut choice_store = ADDRESS_AND_DAY_TO_CHOICE.load(deps.storage, user_and_day.clone())?;
    let choice_len=choice_store.len();
    let char_vec: Vec<char> = choice_store.chars().collect();
    if char_vec[choice_len-1] == 'G' && char_vec[choice_len-2] == 'G' && char_vec[(choice_len-3) as usize]  == 'G' && char_vec[(choice_len-4) as usize]  == 'G' && char_vec[(choice_len-5)] == 'G'  {
        // mint logic here
    }
    Ok(Response::new().add_attribute("choice",choice_store.as_str()))
}
