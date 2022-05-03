use crate::error::ContractError;
use cosmwasm_std::{Coin, Uint128};

pub fn assert_sent_sufficient_coin(
    sent: &[Coin],
    required: Option<Coin>,
) -> Result<Option<Coin>, ContractError> {
    let mut sent_coin: Coin = Coin {
        denom: String::from("token"),
        amount: Uint128::from(0u64),
    };
    if let Some(required_coin) = required {
        let required_amount = required_coin.amount.u128();
        if required_amount > 0 {
            let sent_sufficient_funds = sent.iter().any(|coin| {
                // check if a given sent coin matches denom
                // and has sufficient amount
                sent_coin = coin.clone();
                coin.denom == required_coin.denom && coin.amount.u128() >= required_amount
            });

            return if sent_sufficient_funds {
                Ok(Some(sent_coin))
            } else {
                Err(ContractError::InsufficientFundsSend {})
            }
        }
    }

    Ok(Some(sent_coin))
}
