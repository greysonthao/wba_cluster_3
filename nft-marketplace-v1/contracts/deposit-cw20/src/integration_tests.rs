#[cfg(test)]
mod tests {

    use crate::msg::{Cw20DepositResponse, QueryMsg};
    use anyhow::Error;
    use cosmwasm_std::{
        from_binary, to_binary, Addr, Binary, Coin, Empty, StdError, StdResult, Uint128,
    };
    use cw20::Cw20Coin;

    use cw_multi_test::{App, Contract, ContractWrapper, Executor};

    use crate::contract;
    use cw20_example::{self};

    use nft::{self};

    const USER: &str = "juno1xdekj862ff8vp9jr98cr2e0gfpcnplgj3p0awr";
    const BUYER: &str = "juno1pqn6edrdmr28ekdjv5j2u9uvh6m32tl306kh5h";

    fn mock_app() -> App {
        let init_funds = vec![Coin {
            denom: "utest".to_string(),
            amount: Uint128::new(1_000_000_000),
        }];
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked(USER.to_string()),
                    init_funds.clone(),
                )
                .unwrap()
        });

        app.init_modules(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &Addr::unchecked(BUYER.to_string()), init_funds)
                .unwrap()
        });

        app
    }

    fn contract_nft_marketplace() -> Box<dyn Contract<Empty>> {
        let contract =
            ContractWrapper::new(contract::execute, contract::instantiate, contract::query);
        Box::new(contract)
    }
    fn contract_cw20() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            cw20_example::contract::execute,
            cw20_example::contract::instantiate,
            cw20_example::contract::query,
        );
        Box::new(contract)
    }
    fn contract_cw721() -> Box<dyn Contract<Empty>> {
        let contract = ContractWrapper::new(
            nft::contract::entry::execute,
            nft::contract::entry::instantiate,
            nft::contract::entry::query,
        );
        Box::new(contract)
    }

    pub struct Suite {
        app: App,
        owner: String,
        nft_marketplace_id: u64,
        cw20_id: u64,
        cw721_id: u64,
    }

    impl Suite {
        fn init() -> StdResult<Suite> {
            let mut app = mock_app();
            let owner = USER.to_string();
            let nft_marketplace_id = app.store_code(contract_nft_marketplace());
            let cw20_id = app.store_code(contract_cw20());
            let cw721_id = app.store_code(contract_cw721());

            Ok(Suite {
                app,
                owner,
                nft_marketplace_id,
                cw20_id,
                cw721_id,
            })
        }

        fn instantiate_nft_marketplace(&mut self) -> Result<Addr, Error> {
            let code_id = self.nft_marketplace_id;
            let sender = Addr::unchecked(self.owner.clone());
            let init_msg = crate::msg::InstantiateMsg {};
            let send_funds = vec![];
            let label = "nft_marketplace".to_string();
            let admin = Some(self.owner.clone());

            self.app
                .instantiate_contract(code_id, sender, &init_msg, &send_funds, label, admin)
        }

        fn instantiate_cw20(&mut self) -> Result<Addr, Error> {
            let code_id = self.nft_marketplace_id;
            let sender = Addr::unchecked(self.owner.clone());
            let init_msg = cw20_base::msg::InstantiateMsg {
                name: "new_cw20_token".to_string(),
                symbol: "cw20".to_string(),
                decimals: 6,
                initial_balances: vec![Cw20Coin {
                    address: USER.to_string(),
                    amount: Uint128::new(1_000_000),
                }],
                mint: None,
                marketing: None,
            };
            let send_funds = vec![];
            let label = "new_cw20_contract".to_string();
            let admin = Some(self.owner.clone());

            self.app
                .instantiate_contract(code_id, sender, &init_msg, &send_funds, label, admin)
        }

        fn instantiate_cw721(&mut self) -> Result<Addr, Error> {
            let code_id = self.nft_marketplace_id;
            let sender = Addr::unchecked(self.owner.clone());
            let init_msg = cw721_base::InstantiateMsg {
                name: "cw721_project".to_string(),
                symbol: "cw721".to_string(),
                minter: String::from(USER.clone()),
            };
            let send_funds = vec![];
            let label = "new_cw721_contract".to_string();
            let admin = Some(self.owner.clone());

            self.app
                .instantiate_contract(code_id, sender, &init_msg, &send_funds, label, admin)
        }

        fn smart_query(&self, contract_addr: String, msg: QueryMsg) -> Result<Binary, StdError> {
            self.app.wrap().query_wasm_smart(contract_addr, &msg)
        }

        fn query_balance(&self, address: String, denom: String) -> Result<Coin, StdError> {
            self.app.wrap().query_balance(address, denom)
        }
    }

    #[test]
    fn test_deposit_and_withdraw_native() {
        let mut suite = Suite::init().unwrap();
        let nft_marketplace_addr = suite.instantiate_nft_marketplace().unwrap();

        //QUERY SENDER ADDRESS BALANCE
        let res = suite
            .query_balance(USER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(1000000000));

        //QUERY BUYER ADDRESS BALANCE
        let res = suite
            .query_balance(BUYER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(1000000000));

        //QUERY THE NFT MARKETPLACE CONTRACT. SHOULD BE 0 BALANCE
        let res = suite
            .query_balance(
                nft_marketplace_addr.clone().to_string(),
                "utest".to_string(),
            )
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(0));

        //DEPOSIT UTEST TOKENS INTO THE NFT MARKETPLACE
        let msg = crate::msg::ExecuteMsg::Deposit {};
        let send_funds = vec![Coin {
            denom: "utest".to_string(),
            amount: Uint128::new(1_000_000),
        }];

        let _res = suite
            .app
            .execute_contract(
                Addr::unchecked(suite.owner.clone()),
                nft_marketplace_addr.clone(),
                &msg,
                &send_funds,
            )
            .unwrap();

        let _res = suite
            .app
            .execute_contract(
                Addr::unchecked(suite.owner.clone()),
                nft_marketplace_addr.clone(),
                &msg,
                &send_funds,
            )
            .unwrap();
        let _res = suite
            .app
            .execute_contract(
                Addr::unchecked(BUYER.to_string()),
                nft_marketplace_addr.clone(),
                &msg,
                &send_funds,
            )
            .unwrap();

        //QUERY THE NFT MARKETPLACE CONTRACT FOR THE UTEST TOKENS
        let res = suite
            .query_balance(nft_marketplace_addr.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(3_000_000));

        //QUERY SENDER ADDRESS BALANCE
        let res = suite
            .query_balance(USER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(998_000_000));

        //QUERY BUYER ADDRESS BALANCE
        let res = suite
            .query_balance(BUYER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(999_000_000));

        //WITHDRAW Native TOKENS FROM NFT MARKETPLACE
        let msg = crate::msg::ExecuteMsg::Withdraw {
            amount: 1_999_999,
            denom: "utest".to_string(),
        };
        let send_funds = vec![];

        let _res = suite
            .app
            .execute_contract(
                Addr::unchecked(suite.owner.clone()),
                nft_marketplace_addr.clone(),
                &msg,
                &send_funds,
            )
            .unwrap();

        //println!("execute withdraw {:?}", res);

        //QUERY THE NFT MARKETPLACE CONTRACT FOR THE UTEST TOKENS
        let res = suite
            .query_balance(
                nft_marketplace_addr.clone().to_string(),
                "utest".to_string(),
            )
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(1_000_001));

        //QUERY SENDER ADDRESS BALANCE
        let res = suite
            .query_balance(USER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(999_999_999));
    }

    #[test]
    fn test_deposit_and_withdraw_cw20() {
        let mut suite = Suite::init().unwrap();
        let cw20_addr = suite.instantiate_cw20().unwrap();
        let nft_marketplace_addr = suite.instantiate_nft_marketplace().unwrap();

        //QUERY SENDER ADDRESS BALANCE
        let res = suite
            .query_balance(USER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(1000000000));

        //QUERY BUYER ADDRESS BALANCE
        let res = suite
            .query_balance(BUYER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(1000000000));

        //QUERY THE NFT MARKETPLACE CONTRACT for the cw20 token. SHOULD BE 0 BALANCE
        let msg = crate::msg::QueryMsg::GetCw20Deposit {
            address: USER.to_string(),
        };

        let res = suite.smart_query(cw20_addr.clone().to_string(), msg);

        match res {
            Err(_) => {}
            _ => panic!("should error here since there aren't any deposits yet"),
        };

        //println!("res: {:?}", res);

        //DEPOSIT CW20 TOKENS INTO THE NFT MARKETPLACE
        let cw20_hook = crate::msg::Cw20HookMsg::Deposit {
            owner: USER.to_string(),
            amount: 100,
        };
        let cw20_msg = cw20::Cw20ReceiveMsg {
            sender: nft_marketplace_addr.clone().to_string(),
            amount: Uint128::new(0),
            msg: to_binary(&cw20_hook).unwrap(),
        };
        let msg = crate::msg::ExecuteMsg::Receive(cw20_msg);
        let send_funds = vec![];

        let _res = suite
            .app
            .execute_contract(
                Addr::unchecked(suite.owner.clone()),
                nft_marketplace_addr.clone(),
                &msg,
                &send_funds,
            )
            .unwrap();

        let _res = suite
            .app
            .execute_contract(
                Addr::unchecked(suite.owner.clone()),
                nft_marketplace_addr.clone(),
                &msg,
                &send_funds,
            )
            .unwrap();

        //QUERY THE NFT MARKETPLACE CONTRACT FOR THE cw20 TOKENS
        let msg = crate::msg::QueryMsg::GetCw20Deposit {
            address: suite.owner.clone().to_string(),
        };
        let res = suite
            .smart_query(cw20_addr.clone().to_string(), msg)
            .unwrap();

        let value: Cw20DepositResponse = from_binary(&res).unwrap();

        println!("VALUE: {:?}", value);
        assert_eq!(value.deposits[0].owner.clone(), USER.to_string());

        /*
        //QUERY SENDER ADDRESS BALANCE
        let res = suite
            .query_balance(USER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(998_000_000));

        //QUERY BUYER ADDRESS BALANCE
        let res = suite
            .query_balance(BUYER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(999_000_000));

        //WITHDRAW Native TOKENS FROM NFT MARKETPLACE
        let msg = crate::msg::ExecuteMsg::Withdraw {
            amount: 1_999_999,
            denom: "utest".to_string(),
        };
        let send_funds = vec![];

        let _res = suite
            .app
            .execute_contract(
                Addr::unchecked(suite.owner.clone()),
                nft_marketplace_addr.clone(),
                &msg,
                &send_funds,
            )
            .unwrap();

        //println!("execute withdraw {:?}", res);

        //QUERY THE NFT MARKETPLACE CONTRACT FOR THE UTEST TOKENS
        let res = suite
            .query_balance(
                nft_marketplace_addr.clone().to_string(),
                "utest".to_string(),
            )
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(1_000_001));

        //QUERY SENDER ADDRESS BALANCE
        let res = suite
            .query_balance(USER.to_string(), "utest".to_string())
            .unwrap();

        assert_eq!(res.denom, "utest".to_string());
        assert_eq!(res.amount, Uint128::new(999_999_999)); */
    }
}
