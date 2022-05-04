#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{coin, DepsMut};

    use crate::contract::{execute, instantiate};
    use crate::error::ContractError;
    use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg };

    fn init_with_owner_helper(deps: DepsMut) {
        let msg = InstantiateMsg { };
        let info = mock_info("admin", &[coin(0u128, "uscrt")]);
        let _res = instantiate(deps, mock_env(), info, msg)
            .expect("contract successfully handles InstantiateMsg");
    }

    fn insert_words_helper(deps: DepsMut) {
        let msg = ExecuteMsg::InsertWordInDictionary { words_list: vec!["terra".to_string(),"cosmo".to_string(),"track".to_string()] };
        let info = mock_info("admin", &[coin(0u128, "uscrt")]);
        let _res = execute(deps, mock_env(), info, msg)
            .expect("contract successfully handles InsertWords");
    }

    fn make_guess_helper(deps: DepsMut) {
        let msg = ExecuteMsg::MakeGuess { word: "terra".to_string() };
        let info = mock_info("notAdmin1", &[coin(25000u128, "uscrt")]);
        println!("{}", mock_env().block.time.seconds());
        let _res = execute(deps, mock_env() , info, msg)
            .expect("contract successfully handles makeGuess");
    }

    #[test]
    fn mock_init() {
        let mut deps = mock_dependencies();
        init_with_owner_helper(deps.as_mut());
    }

    #[test]
    fn insert_words_in_dictionary() {
        let mut deps = mock_dependencies();
        init_with_owner_helper(deps.as_mut());
        insert_words_helper(deps.as_mut());
    }

    #[test]
    fn make_guess() {
        let mut deps = mock_dependencies();
        init_with_owner_helper(deps.as_mut());
        insert_words_helper(deps.as_mut());
        make_guess_helper(deps.as_mut());
    }

}
