#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use std::borrow::BorrowMut;
use std::collections::HashSet;
use std::ops::Add;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{ADDR_AND_DAY_TO_CHOICE, DEPLOYING_BLOCK_TIMESTAMP, WORD_DICTIONARY};

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

pub fn execute_insert_word_dictionary(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut words_list: HashSet<String>,
) -> Result<Response, ContractError> {
    let mut dictionary = WORD_DICTIONARY.load(deps.storage)?.clone();
    dictionary.word_list = dictionary.word_list.union(&words_list).cloned().collect();
    WORD_DICTIONARY.save(deps.storage, &dictionary)?;
    Ok(Response::new().add_attribute("method","word inserted"))
}

pub fn update_today_word_and_return(
    deps: &mut DepsMut,
    env: Env,
    word: String,
) -> Result<(String, u64), ContractError> {
    let mut dictionary = WORD_DICTIONARY.load(deps.storage)?.clone();
    let deploy_time = DEPLOYING_BLOCK_TIMESTAMP.load(deps.storage)?;
    let current_day = (env.block.time.seconds() - deploy_time) / 86400;
    if dictionary.day < current_day {
        let mut i = 0;
        for word in dictionary.word_list.iter() {
            if i == current_day {
                dictionary.day_word = word.clone();
                break;
            }
            i = i + 1;
        }
        dictionary.day = current_day;
        WORD_DICTIONARY.save(deps.storage, &dictionary)?;
    }
    if !dictionary.word_list.contains(word.as_str()) {
        return Err(ContractError::WrongGuess {});
    }
    return Ok((dictionary.day_word, current_day));
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
    let (today_word, current_day) =
        update_today_word_and_return(deps.borrow_mut(), env, word.clone())?;
    let mut choice = "".to_string();
    for (i, ch) in word.chars().enumerate() {
        let pos = today_word.find(ch);
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
    let mut choice_store = ADDR_AND_DAY_TO_CHOICE.load(deps.storage, user_and_day.clone())?;
    choice_store.push_str(choice.as_str());
    ADDR_AND_DAY_TO_CHOICE.save(deps.storage, user_and_day, &choice_store);
    Ok(Response::new().add_attribute("choice",choice.as_str()))
}
