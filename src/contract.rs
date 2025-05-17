#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point};
use cosmwasm_std::{Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult, SubMsg, SubMsgResult, to_json_binary, WasmMsg};
use cw2::set_contract_version;

use crate::ack::make_ack_fail;
use crate::error::ContractError;
use crate::msg::{Contract2ExecuteMsg, ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{BALANCE_INFO, QUERY_DENOM};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:contract1";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const REPLY_ID_FROM_CONTRACT2: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.api.debug("WASMDEBUG: instantiate");
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    QUERY_DENOM.save(deps.storage, &msg.query_denom)?;

    Ok(Response::new()
        .add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Start { contract2_addr, query_address } => {
            let contract2_address = deps.api.addr_validate(&contract2_addr)?;
            let query_denom = QUERY_DENOM.load(deps.storage)?;

            let exec_msg = Contract2ExecuteMsg::TriggerFlow { query_address, query_denom };

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: contract2_address.to_string(),
                msg: to_json_binary(&exec_msg)?,
                funds: vec![],
            });

            Ok(Response::new()
                .add_submessage(SubMsg::reply_on_success(msg, REPLY_ID_FROM_CONTRACT2))
                .add_attribute("action", "start_flow"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::BalanceInfo {} => {
            let balance_info = BALANCE_INFO.load(deps.storage)?;
            to_json_binary(&balance_info)
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        REPLY_ID_FROM_CONTRACT2 => match reply.result {
            SubMsgResult::Ok(msg) => {
                deps.api.debug("WASMDEBUG: reply REPLY_ID_CONTRACT2 Ok");
                let mut reported_address = String::new();
                let mut reported_balance = String::new();

                for event in msg.events {
                    for attr in event.attributes {
                        if attr.key == "reported_address" {
                            reported_address = attr.value;
                        } else if attr.key == "reported_balance" {
                            reported_balance = attr.value;
                        }
                    }
                }

                BALANCE_INFO.save(deps.storage, &reported_balance)?;

                Ok(Response::new()
                    .add_attribute("reply_from", "contract2")
                    .add_attribute("address", reported_address)
                    .add_attribute("balance", reported_balance))
            },
            SubMsgResult::Err(err) => {
                deps.api.debug("WASMDEBUG: reply REPLY_ID_CONTRACT2 Fail");
                Ok(Response::new().set_data(make_ack_fail(err)))
            }
        },
        _ => Err(ContractError::UnknownReplyId { id: reply.id }),
    }
}

#[cfg(test)]
mod tests {}
