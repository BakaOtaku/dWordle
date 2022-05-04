#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, coin, Uint128, Order, BankMsg, Addr};
use std::borrow::BorrowMut;
use std::cmp::max_by;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashSet};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Add, Div};
use cw_storage_plus::Bound;
use crate::coin_helpers::assert_sent_sufficient_coin;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{DAY_AND_ADDRESS_TO_BLOCK_WON, ADDRESS_AND_DAY_TO_CHOICE, DAY_AND_BLOCK_TO_DEPOSIT, DAY_AND_BLOCK_TO_WINNER_COUNT, DEPLOYING_BLOCK_TIMESTAMP, WORD_DICTIONARY};

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
    env: &Env,
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
        DAY_AND_ADDRESS_TO_BLOCK_WON.save(deps.storage, (current_day, &deps.api.addr_validate("secret1hg7lzjrmfaljqedgau87apzdj5ms4kfma4fwyy")?), &env.block.height)?;
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
    assert_sent_sufficient_coin(&info.funds, Some(coin(25000, "uscrt")))?;
    // chargeTokenForAttempt


    let (today_word_list, current_day, word_dictionary) =
        update_today_word_and_return(deps.borrow_mut(), &env)?;
    if word_dictionary.contains(&word) {
        return Err(ContractError::WrongGuess {});
    }
    DAY_AND_BLOCK_TO_DEPOSIT.update(deps.storage, (current_day, env.block.height), |already_deposited: Option<Uint128>| -> StdResult<_> { Ok(already_deposited.unwrap_or_default()+ Uint128::new(25000)) })?;

    let sender = info.sender.to_string();

    let calc_hash = SenderNDay{day: current_day,msg_sender: sender};

    let this_users_hash = calculate_hash( &calc_hash) as usize/3;
    let this_users_word = today_word_list[this_users_hash].clone();

    let mut choice = "".to_string();
    for (i, ch) in word.chars().enumerate() {
        let pos = this_users_word.find(ch);
        match pos {
            None => {
                choice = choice.add("B");
            }
            Some(position) => {
                if position == i {
                    choice = choice.add("G");
                } else {
                    choice = choice.add("Y");
                }
            }
        }
    }
    if check_if_correct(&choice)  {
        DAY_AND_BLOCK_TO_WINNER_COUNT.update(deps.storage, (current_day, env.block.height ), |already_winner_count: Option<u64>| -> StdResult<_> { Ok(already_winner_count.unwrap_or_default() + 1) })?;
        DAY_AND_ADDRESS_TO_BLOCK_WON.save(deps.storage, (current_day, &info.sender), &env.block.height)?;
    }

    let mut choice_store = ADDRESS_AND_DAY_TO_CHOICE.load(deps.storage, (&info.sender, current_day))?;
    choice_store.push_str(choice.as_str());
    ADDRESS_AND_DAY_TO_CHOICE.save(deps.storage, (&info.sender, current_day), &choice_store)?;
    Ok(Response::new().add_attribute("choice",choice.as_str()))
}

pub fn claim(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    day_claim: u64
) -> Result<Response, ContractError> {
    let deploy_time = DEPLOYING_BLOCK_TIMESTAMP.load(deps.storage)?;
    let current_day = (env.block.time.seconds() - deploy_time) / 86400;
    if day_claim >= current_day {
        return Err(ContractError::DayNotEnded {})
    }
    let block_won = DAY_AND_ADDRESS_TO_BLOCK_WON.load(deps.storage, (day_claim, &info.sender))?;
    let blocks_and_deposits: StdResult<Vec<_>> = DAY_AND_BLOCK_TO_DEPOSIT
        .prefix(current_day)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let result_vector = blocks_and_deposits?.clone();
    let first_block = result_vector.get(0).unwrap().0;
    let last_block = result_vector.last().unwrap().0;
    let mut total_reward = Uint128::from(0u128);
    let mut divisor = 1;
    for ele in result_vector {
        if ele.0 < block_won {
            continue;
        }
        divisor = DAY_AND_BLOCK_TO_WINNER_COUNT.load(deps.storage,(day_claim, ele.0)).unwrap_or(divisor);
        total_reward =  total_reward + ele.1 / Uint128::from(divisor);
    }

    // transfer funds here
    Ok(Response::new().add_message(
        BankMsg::Send {
            to_address: info.sender.to_string(),
            amount: vec![coin(total_reward.u128(), "uscrt")]
        }
    ).add_attributes(vec![("amount", total_reward.to_string()), ("claimed by",info.sender.to_string())]))
}

pub fn check_if_correct(
    word_to_check: &String
) -> bool{
    let choice_len= word_to_check.len();
    let char_vec: Vec<char> = word_to_check.chars().collect();
    if char_vec[choice_len-1] == 'G' && char_vec[choice_len-2] == 'G' && char_vec[(choice_len-3)]  == 'G' && char_vec[(choice_len-4)]  == 'G' && char_vec[(choice_len-5)] == 'G'  {
        return true;
    }
    false
}
