#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    from_binary, to_binary, BankMsg, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdError, StdResult, Uint128, WasmMsg,
};
use cw2::set_contract_version;
use cw20::Cw20ReceiveMsg;
use cw721::Cw721ReceiveMsg;

use crate::error::ContractError;
use crate::msg::{
    Cw20DepositResponse, Cw20HookMsg, Cw721DepositResponse, Cw721HookMsg, DepositResponse,
    ExecuteMsg, InstantiateMsg, QueryMsg,
};
use crate::state::{
    Cw20Deposit, Cw721Deposit, Deposit, Offer, ASKS, CW20_DEPOSITS, CW721_DEPOSITS, DEPOSITS,
};

use nft;

const CONTRACT_NAME: &str = "deposit-cw20-example";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Receive(cw20_msg) => receive_cw20(deps, info, cw20_msg),
        ExecuteMsg::WithdrawCw20 { owner, amount } => try_withdraw_cw20(deps, info, owner, amount),
        ExecuteMsg::Deposit {} => try_deposit(deps, info),
        ExecuteMsg::Withdraw { amount, denom } => try_withdraw_deposit(deps, info, amount, denom),
        ExecuteMsg::ReceiveNft(cw721_msg) => receive_cw721(deps, info, cw721_msg),
        ExecuteMsg::WithdrawNft {
            cw721_contract,
            token_id,
        } => try_withdraw_cw721(deps, info, cw721_contract, token_id),
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit { owner, amount }) => {
            execute_cw20_deposit(deps, info, owner, amount)
        }
        Ok(Cw20HookMsg::Purchase {
            token_id,
            cw721_contract,
        }) => execute_purchase(deps, info, token_id, cw721_contract, cw20_msg),
        Err(_) => todo!(),
    }
}

pub fn receive_cw721(
    deps: DepsMut,
    info: MessageInfo,
    cw721_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    match from_binary(&cw721_msg.msg) {
        Ok(Cw721HookMsg::Deposit {
            owner,
            token_id,
            cw20_contract,
            amount,
        }) => execute_cw721_deposit(deps, info, owner, token_id, cw20_contract, amount),
        Err(_) => todo!(),
    }
}

pub fn execute_cw721_deposit(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    token_id: String,
    cw20_contract: String,
    amount: u128,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender.clone().to_string();

    match CW721_DEPOSITS.load(deps.storage, (&owner, &contract_addr, &token_id)) {
        Ok(_) => return Err(ContractError::Cw721AlreadyDeposited {}),
        Err(_) => {
            let deposit = Cw721Deposit {
                owner: owner.clone(),
                contract: contract_addr.clone(),
                token_id: token_id.clone(),
            };

            CW721_DEPOSITS.save(deps.storage, (&owner, &contract_addr, &token_id), &deposit)?;

            let ask = Offer {
                owner: owner.clone(),
                token_id: token_id.clone(),
                cw721_contract: contract_addr.clone().to_string(),
                cw20_contract: cw20_contract.clone(),
                amount,
            };

            ASKS.save(deps.storage, (&contract_addr.to_string(), &token_id), &ask)?;

            Ok(Response::new()
                .add_attribute("execute", "deposit_cw721")
                .add_attribute("owner", owner)
                .add_attribute("cw721_contract", contract_addr)
                .add_attribute("token_id", token_id)
                .add_attribute("cw20_addr", cw20_contract)
                .add_attribute("amount_requested", amount.to_string()))
        }
    }
}

pub fn execute_cw20_deposit(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: u128,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender.clone().to_string();

    match CW20_DEPOSITS.load(deps.storage, (&owner, &contract_addr)) {
        Ok(mut deposit) => {
            deposit.amount = deposit.amount.checked_add(amount.clone()).unwrap();
            deposit.count = deposit.count.checked_add(1).unwrap();

            CW20_DEPOSITS.save(deps.storage, (&owner, &contract_addr), &deposit)?;
        }
        Err(_) => {
            let deposit = Cw20Deposit {
                owner: owner.clone(),
                amount: amount.clone(),
                contract: contract_addr.clone(),
                count: 1,
            };

            CW20_DEPOSITS.save(deps.storage, (&owner, &contract_addr), &deposit)?;
        }
    }

    Ok(Response::new()
        .add_attribute("execute", "cw20_deposit")
        .add_attribute("owner", owner)
        .add_attribute("amount", amount.to_string())
        .add_attribute("contract", contract_addr))
}

pub fn try_withdraw_cw20(
    deps: DepsMut,
    info: MessageInfo,
    owner: String,
    amount: u128,
) -> Result<Response, ContractError> {
    let contract_addr = info.sender.clone().to_string();

    match CW20_DEPOSITS.load(deps.storage, (&owner, &contract_addr)) {
        Ok(mut deposit) => {
            deposit.count = deposit.count.checked_sub(1).unwrap();
            deposit.amount = deposit.amount.checked_sub(amount.clone()).unwrap();

            CW20_DEPOSITS.save(deps.storage, (&owner, &contract_addr), &deposit)?;

            Ok(Response::new()
                .add_attribute("execute", "withdraw_cw20")
                .add_attribute("amount", amount.to_string())
                .add_attribute("from", contract_addr)
                .add_attribute("to", owner))
        }
        Err(_) => Err(ContractError::NoCw20ToWithdraw {}),
    }
}

pub fn try_deposit(deps: DepsMut, info: MessageInfo) -> Result<Response, ContractError> {
    let sender = info.sender.clone().to_string();
    let funds = info.funds[0].clone();

    match DEPOSITS.load(deps.storage, (&sender, &funds.denom)) {
        Ok(mut deposit) => {
            deposit.amount.amount = deposit.amount.amount.checked_add(funds.amount).unwrap();
            deposit.count = deposit.count.checked_add(1).unwrap();

            DEPOSITS.save(deps.storage, (&sender, &funds.denom), &deposit)?;
        }
        Err(_) => {
            let deposit = Deposit {
                owner: sender.clone(),
                amount: funds.clone(),
                count: 1,
            };

            DEPOSITS.save(deps.storage, (&sender, &funds.denom), &deposit)?;
        }
    }
    Ok(Response::new()
        .add_attribute("execute", "deposit")
        .add_attribute("amount", funds.amount.to_string())
        .add_attribute("denom", funds.denom))
}

pub fn try_withdraw_deposit(
    deps: DepsMut,
    info: MessageInfo,
    amount: u128,
    denom: String,
) -> Result<Response, ContractError> {
    let sender = info.sender.clone().to_string();

    match DEPOSITS.load(deps.storage, (&sender, &denom)) {
        Ok(mut deposit) => {
            deposit.count = deposit.count.checked_sub(1).unwrap();
            deposit.amount.amount = deposit
                .amount
                .amount
                .checked_sub(Uint128::from(amount))
                .unwrap();

            let msg = BankMsg::Send {
                to_address: sender.clone(),
                amount: vec![Coin {
                    denom: deposit.amount.denom.clone(),
                    amount: Uint128::new(amount),
                }],
            };

            DEPOSITS.save(deps.storage, (&sender, &denom), &deposit)?;
            Ok(Response::new()
                .add_attribute("execute", "withdraw_deposit")
                .add_attribute("amount", amount.to_string())
                .add_attribute("denom", deposit.amount.denom.clone())
                .add_attribute("to", sender)
                .add_message(msg))
        }
        Err(_) => todo!(),
    }
}

pub fn try_withdraw_cw721(
    deps: DepsMut,
    info: MessageInfo,
    cw721_contract: String,
    token_id: String,
) -> Result<Response, ContractError> {
    match CW721_DEPOSITS.load(
        deps.storage,
        (&info.sender.clone().to_string(), &cw721_contract, &token_id),
    ) {
        Ok(_) => {
            CW721_DEPOSITS.remove(
                deps.storage,
                (&info.sender.clone().to_string(), &cw721_contract, &token_id),
            );

            let exec_msg = nft::contract::ExecuteMsg::TransferNft {
                recipient: info.sender.clone().to_string(),
                token_id: token_id.clone(),
            };

            let msg = WasmMsg::Execute {
                contract_addr: cw721_contract.clone(),
                msg: to_binary(&exec_msg)?,
                funds: vec![],
            };

            Ok(Response::new()
                .add_attribute("execute", "cw721_withdraw")
                .add_attribute("token_id", token_id)
                .add_attribute("receiver_address", info.sender.to_string())
                .add_message(msg))
        }
        Err(_) => return Err(ContractError::NoCw721ToWithdraw {}),
    }
}

pub fn execute_purchase(
    deps: DepsMut,
    _info: MessageInfo,
    token_id: String,
    cw721_contract: String,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    //let buyer = info.sender.clone().to_string();

    let original_owner: String;

    match ASKS.load(deps.storage, (&cw721_contract, &token_id)) {
        Ok(ask) => {
            if Uint128::new(ask.amount) != cw20_msg.amount {
                return Err(ContractError::InvalidBid {});
            }

            original_owner = ask.owner.clone();

            let exec_msg = nft::contract::ExecuteMsg::TransferNft {
                recipient: cw20_msg.sender.clone(),
                token_id: token_id.clone(),
            };
            let msg = WasmMsg::Execute {
                contract_addr: cw721_contract.clone(),
                msg: to_binary(&exec_msg)?,
                funds: vec![],
            };
            CW721_DEPOSITS.remove(deps.storage, (&ask.owner, &cw721_contract, &token_id));
            ASKS.remove(deps.storage, (&cw721_contract, &token_id));

            Ok(Response::new()
                .add_attribute("execute", "nft_purchase")
                .add_attribute("token_id", token_id)
                .add_attribute("from", original_owner)
                .add_attribute("to", cw20_msg.sender)
                .add_message(msg))
        }
        Err(_) => return Err(ContractError::NoBidsForTokenID {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCw20Deposit { address } => to_binary(&try_query_cw20_deposit(deps, address)?),
        QueryMsg::GetDeposits { address } => to_binary(&try_query_deposit(deps, address)?),
        QueryMsg::GetCw721Deposit { address, contract } => {
            to_binary(&try_query_cw721_deposit(deps, address, contract)?)
        }
    }
}

pub fn try_query_deposit(deps: Deps, address: String) -> StdResult<DepositResponse> {
    let _valid_addr = deps.api.addr_validate(&address)?;

    let res: StdResult<Vec<_>> = DEPOSITS
        .prefix(&address)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let deposits_found = res?;

    if deposits_found.len() == 0 {
        return Err(StdError::generic_err("No deposits found for that address"));
    }

    let mut deposits: Vec<Deposit> = vec![];

    for deposit in deposits_found {
        deposits.push(deposit.1);
    }

    Ok(DepositResponse { deposits })
}

pub fn try_query_cw20_deposit(deps: Deps, address: String) -> StdResult<Cw20DepositResponse> {
    let _valid_addr = deps.api.addr_validate(&address)?;

    let wrapped_deposits: StdResult<Vec<_>> = CW20_DEPOSITS
        .prefix(&address)
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let deposits = wrapped_deposits?;

    if deposits.len() == 0 {
        return Err(StdError::generic_err(
            "No cw20 deposits exist for that address",
        ));
    }

    let mut real_deposits: Vec<Cw20Deposit> = vec![];
    for deposit in deposits {
        real_deposits.push(deposit.1);
    }

    Ok(Cw20DepositResponse {
        deposits: real_deposits,
    })
}

pub fn try_query_cw721_deposit(
    deps: Deps,
    address: String,
    contract: String,
) -> StdResult<Cw721DepositResponse> {
    let _valid_addr = deps.api.addr_validate(&address)?;
    let _valid_contract_addr = deps.api.addr_validate(&contract)?;

    let res: StdResult<Vec<_>> = CW721_DEPOSITS
        .prefix((&address, &contract))
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let deposits_found = res?;

    if deposits_found.len() == 0 {
        return Err(StdError::generic_err(
            "No cw721 deposits exist for that address",
        ));
    }

    let mut deposits: Vec<Cw721Deposit> = vec![];
    for deposit in deposits_found {
        deposits.push(deposit.1);
    }

    Ok(Cw721DepositResponse { deposits })
}
