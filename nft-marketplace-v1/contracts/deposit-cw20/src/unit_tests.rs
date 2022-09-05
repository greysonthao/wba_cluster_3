#[cfg(test)]
mod tests {
    use cosmwasm_std::{from_binary, to_binary, DepsMut, Response, Uint128};

    use cw20::Cw20ReceiveMsg;
    use cw721::Cw721ReceiveMsg;

    use crate::contract::{execute, instantiate, query};
    use crate::error::ContractError;
    use crate::msg::{
        Cw20DepositResponse, Cw20HookMsg, Cw721DepositResponse, Cw721HookMsg, DepositResponse,
        ExecuteMsg, InstantiateMsg, QueryMsg,
    };

    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::Coin;

    const SENDER: &str = "sender_address";
    const AMOUNT: u128 = 100000;
    const DENOM: &str = "utest";

    fn proper_instantiate(deps: DepsMut) -> Result<Response, ContractError> {
        let msg = InstantiateMsg {};
        let info = mock_info(&SENDER, &[]);
        instantiate(deps, mock_env(), info, msg)
    }

    fn execute_cw20_deposit(deps: DepsMut) -> Result<Response, ContractError> {
        let cw20_msg = Cw20ReceiveMsg {
            sender: "".to_string(),
            amount: Uint128::new(0),
            msg: to_binary(&Cw20HookMsg::Deposit {
                owner: "right_guy".to_string(),
                amount: 100u128,
            })?,
        };

        let msg = ExecuteMsg::Receive(cw20_msg);
        let info = mock_info(&"contract_addr", &[]);
        execute(deps, mock_env(), info, msg)
    }

    fn execute_cw721_deposit(deps: DepsMut) -> Result<Response, ContractError> {
        let cw721_msg = Cw721ReceiveMsg {
            sender: "".to_string(),
            token_id: "".to_string(),
            msg: to_binary(&Cw721HookMsg::Deposit {
                owner: "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h".to_string(),
                token_id: "TNT".to_string(),
                cw20_contract: "cw20addr".to_string(),
                amount: 100,
            })?,
        };

        let msg = ExecuteMsg::ReceiveNft(cw721_msg);
        let info = mock_info(&"contract_addr", &[]);
        execute(deps, mock_env(), info, msg)
    }

    fn execute_deposit(deps: DepsMut) -> Result<Response, ContractError> {
        let msg = ExecuteMsg::Deposit {};
        let info = mock_info(
            &SENDER,
            &[Coin {
                amount: Uint128::new(AMOUNT),
                denom: DENOM.to_string(),
            }],
        );
        execute(deps, mock_env(), info, msg)
    }

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let _res = proper_instantiate(deps.as_mut()).unwrap();
    }

    #[test]
    fn test_deposit_cw721_and_query() {
        let mut deps = mock_dependencies();
        let _res = proper_instantiate(deps.as_mut()).unwrap();
        let _res = execute_cw721_deposit(deps.as_mut()).unwrap();

        let msg = QueryMsg::GetCw721Deposit {
            address: "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h".to_string(),
            contract: "contract_addr".to_string(),
        };

        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let res: Cw721DepositResponse = from_binary(&res).unwrap();

        //println!("RES: {:?}", res);
        assert_eq!(
            res.deposits[0].owner,
            "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h"
        );
    }

    #[test]
    fn test_deposit_cw721_and_withdraw() {
        let mut deps = mock_dependencies();
        let _res = proper_instantiate(deps.as_mut()).unwrap();
        let _res = execute_cw721_deposit(deps.as_mut()).unwrap();

        let msg = QueryMsg::GetCw721Deposit {
            address: "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h".to_string(),
            contract: "contract_addr".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: Cw721DepositResponse = from_binary(&res).unwrap();
        assert_eq!(
            res.deposits[0].owner,
            "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h"
        );

        let msg = ExecuteMsg::WithdrawNft {
            cw721_contract: "contract_addr".to_string(),
            token_id: "TNT".to_string(),
        };
        let info = mock_info(&"juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        //println!("RES: {:?}", res);

        let msg = QueryMsg::GetCw721Deposit {
            address: "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h".to_string(),
            contract: "contract_addr".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg);

        match res {
            Err(_) => {}
            _ => panic!("Should error here"),
        }
    }

    #[test]
    fn test_deposit_cw721_and_purchase() {
        let mut deps = mock_dependencies();
        let _res = proper_instantiate(deps.as_mut()).unwrap();
        let _res = execute_cw721_deposit(deps.as_mut()).unwrap();

        let msg = QueryMsg::GetCw721Deposit {
            address: "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h".to_string(),
            contract: "contract_addr".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: Cw721DepositResponse = from_binary(&res).unwrap();
        assert_eq!(
            res.deposits[0].owner,
            "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h"
        );

        /*  let msg = ExecuteMsg::WithdrawNft {
            cw721_contract: "contract_addr".to_string(),
            token_id: "TNT".to_string(),
        };
        let info = mock_info(&"juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap(); */
        //println!("RES: {:?}", res);

        let cw20_msg = Cw20ReceiveMsg {
            sender: "buyer_addr".to_string(),
            amount: Uint128::new(100),
            msg: to_binary(&Cw20HookMsg::Purchase {
                token_id: "TNT".to_string(),
                cw721_contract: "contract_addr".to_string(),
            })
            .unwrap(),
        };

        let msg = ExecuteMsg::Receive(cw20_msg);
        let info = mock_info(&"buyer_addr", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
        //println!("RES: {:?}", res);

        let msg = QueryMsg::GetCw721Deposit {
            address: "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h".to_string(),
            contract: "contract_addr".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg);

        match res {
            Err(_) => {}
            _ => panic!("Should error here"),
        }
    }

    #[test]
    fn test_deposit_and_query() {
        let mut deps = mock_dependencies();
        let _res = execute_deposit(deps.as_mut()).unwrap();
        let _res = execute_deposit(deps.as_mut()).unwrap();

        let msg = QueryMsg::GetDeposits {
            address: "wrong_guy".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg);

        match res {
            Err(_) => {}
            _ => panic!("Should error here"),
        }

        let msg = QueryMsg::GetDeposits {
            address: SENDER.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();

        let res: DepositResponse = from_binary(&res).unwrap();

        println!("RES: {:?}", res);
    }
    #[test]
    fn test_deposit_and_withdraw() {
        let mut deps = mock_dependencies();
        let _res = execute_deposit(deps.as_mut()).unwrap();
        let _res = execute_deposit(deps.as_mut()).unwrap();

        let msg = QueryMsg::GetDeposits {
            address: SENDER.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: DepositResponse = from_binary(&res).unwrap();
        assert_eq!(res.deposits[0].amount.amount, Uint128::new(200000));

        let msg = ExecuteMsg::Withdraw {
            amount: 1,
            denom: DENOM.to_string(),
        };
        let info = mock_info(&SENDER, &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        //println!("RES: {:?}", res);

        let msg = QueryMsg::GetDeposits {
            address: SENDER.to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), msg).unwrap();
        let res: DepositResponse = from_binary(&res).unwrap();
        assert_eq!(res.deposits[0].amount.amount, Uint128::new(199999));
    }

    #[test]
    fn test_cw20_deposit_and_query() {
        let mut deps = mock_dependencies();
        let _res = proper_instantiate(deps.as_mut()).unwrap();
        let _res = execute_cw20_deposit(deps.as_mut()).unwrap();

        let query_msg = QueryMsg::GetCw20Deposit {
            address: "wrong_guy".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg);

        match res {
            Err(_) => {}
            _ => panic!("Should error here"),
        }

        let query_msg = QueryMsg::GetCw20Deposit {
            address: "right_guy".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();

        let _res: Cw20DepositResponse = from_binary(&res).unwrap();
        //println!("AMOUNT IN DEPOSIT CONTRACT: {:?}", res);
    }

    #[test]
    fn test_cw20_deposit_and_withdraw() {
        let mut deps = mock_dependencies();
        let _res = proper_instantiate(deps.as_mut()).unwrap();
        let _res = execute_cw20_deposit(deps.as_mut()).unwrap();

        let query_msg = QueryMsg::GetCw20Deposit {
            address: "right_guy".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let res: Cw20DepositResponse = from_binary(&res).unwrap();
        println!("AMOUNT IN CONTRACT AFTER 1ST DEPOSIT: {:?}", res);

        let msg = ExecuteMsg::WithdrawCw20 {
            owner: "wrong_guy".to_string(),
            amount: 100u128,
        };
        let info = mock_info("contract_addr", &[]);
        let res = execute(deps.as_mut(), mock_env(), info, msg);
        match res {
            Err(ContractError::NoCw20ToWithdraw {}) => {}
            _ => panic!("should error here"),
        }

        let msg = ExecuteMsg::WithdrawCw20 {
            owner: "right_guy".to_string(),
            amount: 100u128,
        };
        let info = mock_info("contract_addr", &[]);
        let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

        let query_msg = QueryMsg::GetCw20Deposit {
            address: "right_guy".to_string(),
        };
        let res = query(deps.as_ref(), mock_env(), query_msg).unwrap();
        let res: Cw20DepositResponse = from_binary(&res).unwrap();
        println!("AMOUNT IN CONTRACT AFTER WITHDRAWAL: {:?}", res);
    }
}
