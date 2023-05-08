use std::collections::HashMap;

mod suite;

use crate::{
    contract::{ContractExecMsg, ContractQueryMsg, InstantiateMsg, QueryMsg},
    entry_points,
};
use anyhow::{bail, Result as AnyResult};
use cosmwasm_std::{from_slice, Addr, Coin, Decimal, Empty};

use cw_multi_test::{App, AppBuilder, Contract, Executor};

use crate::contract::{AffiliateSwap, MaxFeePercentageResponse};

impl Contract<Empty> for AffiliateSwap<'_> {
    fn execute(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        let msg = from_slice::<ContractExecMsg>(&msg)?;
        entry_points::execute(deps, env, info, msg).map_err(Into::into)
    }

    fn instantiate(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        info: cosmwasm_std::MessageInfo,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        let msg = from_slice::<InstantiateMsg>(&msg)?;
        entry_points::instantiate(deps, env, info, msg).map_err(Into::into)
    }

    fn query(
        &self,
        deps: cosmwasm_std::Deps<Empty>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Binary> {
        let msg = from_slice::<ContractQueryMsg>(&msg)?;
        entry_points::query(deps, env, msg).map_err(Into::into)
    }

    fn sudo(
        &self,
        deps: cosmwasm_std::DepsMut<Empty>,
        env: cosmwasm_std::Env,
        msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("sudo not implemented for contract")
    }

    fn reply(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: cosmwasm_std::Reply,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("reply not implemented for contract")
    }

    fn migrate(
        &self,
        _deps: cosmwasm_std::DepsMut<Empty>,
        _env: cosmwasm_std::Env,
        _msg: Vec<u8>,
    ) -> AnyResult<cosmwasm_std::Response<Empty>> {
        bail!("migrate not implemented for contract")
    }
}

pub struct TestEnv {
    pub app: App,
    pub creator: Addr,
    pub contract: Addr,
    pub accounts: HashMap<String, Addr>,
}

impl TestEnv {
    fn query_max_fee(&self) -> Decimal {
        let max_fee: MaxFeePercentageResponse = self
            .app
            .wrap()
            .query_wasm_smart(&self.contract, &QueryMsg::GetMaxFeePercentage {})
            .unwrap();
        max_fee.max_fee_percentage
    }
}

pub struct TestEnvBuilder {
    account_balances: HashMap<String, Vec<Coin>>,
    instantiate_msg: Option<InstantiateMsg>,
}

impl TestEnvBuilder {
    pub fn new() -> Self {
        Self {
            account_balances: HashMap::new(),
            instantiate_msg: None,
        }
    }

    pub fn with_instantiate_msg(mut self, msg: InstantiateMsg) -> Self {
        self.instantiate_msg = Some(msg);
        self
    }

    pub fn with_account(mut self, account: &str, balance: Vec<Coin>) -> Self {
        self.account_balances.insert(account.to_string(), balance);
        self
    }

    pub fn build(self) -> TestEnv {
        let mut app = AppBuilder::default().build(|router, _, storage| {
            for (account, balance) in self.account_balances.clone() {
                router
                    .bank
                    .init_balance(storage, &Addr::unchecked(account), balance)
                    .unwrap();
            }
        });

        let creator = Addr::unchecked("creator");
        let code_id = app.store_code(Box::new(AffiliateSwap::new()));
        let contract = app
            .instantiate_contract(
                code_id,
                creator.clone(),
                &self.instantiate_msg.expect("instantiate msg not set"),
                &[],
                "transmuter",
                None,
            )
            .unwrap();

        TestEnv {
            app,
            creator,
            contract,
            accounts: self
                .account_balances
                .keys()
                .map(|k| (k.clone(), Addr::unchecked(k.clone())))
                .collect(),
        }
    }
}
