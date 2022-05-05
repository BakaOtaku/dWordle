#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, coin, Uint128, Order, BankMsg, Binary, Deps, to_binary};
use std::borrow::BorrowMut;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashSet};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::{Add};
use crate::coin_helpers::assert_sent_sufficient_coin;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{DAY_AND_ADDRESS_TO_BLOCK_WON, ADDRESS_AND_DAY_TO_CHOICE, DAY_AND_BLOCK_TO_DEPOSIT, DAY_AND_BLOCK_TO_WINNER_COUNT, DEPLOYING_BLOCK_TIMESTAMP, WORD_DICTIONARY, OWNER};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, StdError> {
    DEPLOYING_BLOCK_TIMESTAMP.save(deps.storage, &env.block.time.seconds())?;
    OWNER.save(deps.storage,&info.sender)?;
    Ok(Response::default().add_attribute("owner",info.sender.to_string()))
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
        },
        ExecuteMsg::ClaimReward { day } => {
            claim(deps, env, info, day)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::DayQueryDeposit {day}  => query_day_data(deps, day),
        QueryMsg::DayQueryWinnerCount {day}  => query_day_winner(deps, day),
    }
}

fn query_day_winner(deps: Deps, day: u64) -> StdResult<Binary> {
    let blocks_and_deposits: StdResult<Vec<_>> = DAY_AND_BLOCK_TO_WINNER_COUNT
        .prefix(day)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let result_vector = blocks_and_deposits?.clone();
    to_binary(&result_vector)
}

fn query_day_data(deps: Deps, day: u64) -> StdResult<Binary> {
    let blocks_and_deposits: StdResult<Vec<_>> = DAY_AND_BLOCK_TO_DEPOSIT
        .prefix(day)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();
    let result_vector = blocks_and_deposits?.clone();
    to_binary(&result_vector)
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
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    words_list: Vec<String>,
) -> Result<Response, ContractError> {
    let owner = OWNER.load(deps.storage)?;
    if !info.sender.to_string().eq(&owner.to_string()) {
        return Err(ContractError::Unauthorized {})
    }
    let mut dictionary = WORD_DICTIONARY.load(deps.storage).unwrap_or_default().clone();
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
    if dictionary.day < current_day + 1 {

        dictionary.day_words[0] = dictionary.word_list[dictionary.index_word].clone();
        dictionary.day_words[1] = dictionary.word_list[dictionary.index_word+1].clone();
        dictionary.day_words[2] = dictionary.word_list[dictionary.index_word+2].clone();
        dictionary.index_word = dictionary.index_word+3;

        dictionary.day = current_day + 1;
        WORD_DICTIONARY.save(deps.storage, &dictionary)?;
        let owner = OWNER.load(deps.storage)?;
        DAY_AND_ADDRESS_TO_BLOCK_WON.save(deps.storage, (current_day, &owner), &env.block.height)?;
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
    if !word_dictionary.contains(&word) {
        return Err(ContractError::WrongGuess {});
    }
    let mut choice_store = ADDRESS_AND_DAY_TO_CHOICE.load(deps.storage, (&info.sender, current_day)).unwrap_or_default();
    if choice_store.len() >= 30 {
        return Err(ContractError::GameOver {});
    }
    DAY_AND_BLOCK_TO_DEPOSIT.update(deps.storage, (current_day, env.block.height), |already_deposited: Option<Uint128>| -> StdResult<_> { Ok(already_deposited.unwrap_or_default()+ Uint128::new(25000)) })?;

    let sender = info.sender.to_string();

    let calc_hash = SenderNDay{day: current_day,msg_sender: sender};

    let this_users_hash = (calculate_hash( &calc_hash) as usize)%3;
    let this_users_word = today_word_list[this_users_hash].clone();

    let mut choice = "".to_string();
    for (i, ch) in word.chars().enumerate() {
        if ch == this_users_word.chars().nth(i).unwrap_or_default() {
            choice = choice.add("G");
        } else {
            let pos = this_users_word.find(ch);
            match pos {
                None => {
                    choice = choice.add("B");
                }
                Some(..) => {
                    choice = choice.add("Y");
                }
            }
        }
    }
    if check_if_correct(&choice)  {
        DAY_AND_BLOCK_TO_WINNER_COUNT.update(deps.storage, (current_day, env.block.height ), |already_winner_count: Option<u64>| -> StdResult<_> { Ok(already_winner_count.unwrap_or_default() + 1) })?;
        DAY_AND_ADDRESS_TO_BLOCK_WON.save(deps.storage, (current_day, &info.sender), &env.block.height)?;
    }

    choice_store.push_str(choice.as_str());
    ADDRESS_AND_DAY_TO_CHOICE.save(deps.storage, (&info.sender, current_day), &choice_store)?;
    Ok(Response::new().add_attribute("choice",choice.as_str()))
}

pub fn claim(
    deps: DepsMut,
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
    let mut total_reward = Uint128::from(0u128);
    let mut divisor = 1;
    for ele in result_vector {
        if ele.0 < block_won {
            continue;
        }
        divisor = DAY_AND_BLOCK_TO_WINNER_COUNT.load(deps.storage,(day_claim, ele.0)).unwrap_or(divisor);
        total_reward =  total_reward + ele.1 / Uint128::from(divisor);
    }
    DAY_AND_ADDRESS_TO_BLOCK_WON.save(deps.storage, (day_claim, &info.sender), &u64::MAX)?;
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
