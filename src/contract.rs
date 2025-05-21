use std::str::FromStr;
#[cfg(not(feature = "library"))]
use cosmwasm_std::{entry_point};
use cosmwasm_std::{BalanceResponse, BankQuery, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, Response, StdError, StdResult, SubMsg, SubMsgResult, to_json_binary, Uint128, WasmMsg};
use cw2::set_contract_version;
use crate::ack::make_ack_fail;
use crate::error::ContractError;
use crate::msg::{Contract2ExecuteMsg, ExecuteMsg, GetCalculatedBalanceResponse, InstantiateMsg, QueryMsg};
use crate::state::{CURRENCY_HUB_ADDRESS, ContractRequest, SENT_REQUEST, is_currency_hub, EXCHANGE_RATE, REQUEST_STATUS, RequestStatus};

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
    let currency_hub_address = deps.api.addr_validate(&msg.currency_hub)?;

    CURRENCY_HUB_ADDRESS.save(deps.storage, &currency_hub_address)?;
    REQUEST_STATUS.save(deps.storage, &RequestStatus::None)?;

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
        ExecuteMsg::RequestTokenPrice { base_asset_denom, quote_asset_denom, query_address } => {
            let query_address = deps.api.addr_validate(&query_address)?;
            let request = ContractRequest {
                base_asset_denom: base_asset_denom.to_string(),
                quote_asset_denom: quote_asset_denom.to_string(),
                query_address,
            };
            SENT_REQUEST.save(deps.storage, &request)?;

            let currency_hub_address = CURRENCY_HUB_ADDRESS.load(deps.storage)?;

            let exec_msg = Contract2ExecuteMsg::GetTokenPrice { base_asset_denom, quote_asset_denom };

            let msg = CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: currency_hub_address.to_string(),
                msg: to_json_binary(&exec_msg)?,
                funds: vec![],
            });

            REQUEST_STATUS.save(deps.storage, &RequestStatus::Requested)?;

            Ok(Response::new()
                .add_submessage(SubMsg::reply_on_success(msg, REPLY_ID_FROM_CONTRACT2))
                .add_attribute("action", "request_token_price"))
        }
        ExecuteMsg::ResponseTokenPrice { base_asset_denom, quote_asset_denom, arithmetic_twap } => {
            if !is_currency_hub(deps.as_ref(), &info.sender) {
                return Err(ContractError::Unauthorized {});
            }

            let contract_request = SENT_REQUEST.load(deps.storage)?;
            if base_asset_denom != contract_request.base_asset_denom || quote_asset_denom != contract_request.quote_asset_denom {
                return Err(ContractError::ArithmeticError {});
            }

            REQUEST_STATUS.save(deps.storage, &RequestStatus::Answered)?;

            let arithmetic_twap = Uint128::from_str(&arithmetic_twap).map_err(|_| ContractError::Uint128ParseError)?;
            EXCHANGE_RATE.save(deps.storage, &arithmetic_twap)?;

            Ok(Response::new().add_attribute("action", "response_token_price"))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::CalculatedBalance {} => {
            let contract_request = SENT_REQUEST.load(deps.storage)?;
            let original_balance = query_balance(deps, contract_request.query_address.to_string(), contract_request.base_asset_denom)?;
            let arithmetic_twap = EXCHANGE_RATE.load(deps.storage)?;

            let exchanged_balance = calculate_required_balance(original_balance.amount, arithmetic_twap)?;
            let response = GetCalculatedBalanceResponse {
                query_address: contract_request.query_address.to_string(),
                original_balance: original_balance.amount.to_string(),
                exchanged_balance,
            };
            to_json_binary(&response)
        }
        QueryMsg::RequestStatus {} => {
            let request_status = REQUEST_STATUS.load(deps.storage)?;
            to_json_binary(&request_status)
        }
    }
}

fn query_balance(deps: Deps, address: String, denom: String) -> StdResult<Coin> {
    let balance: BalanceResponse = deps.querier.query(&QueryRequest::Bank(BankQuery::Balance {
        address,
        denom,
    }))?;
    Ok(balance.amount)
}

fn calculate_required_balance(
    balance_amount: Uint128,
    arithmetic_twap: Uint128
) -> Result<String, StdError> {
    // Perform the multiplication and check for overflow
    let required_amount_in_nano = balance_amount
        .checked_mul(arithmetic_twap)?;

    // Perform the division to adjust for the scale of twap_price (1_000_000_000_000_000_000)
    let required_amount = required_amount_in_nano
        .checked_div(Uint128::new(1_000_000_000_000_000_000))?;

    Ok(required_amount.to_string())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(deps: DepsMut, _env: Env, reply: Reply) -> Result<Response, ContractError> {
    match reply.id {
        REPLY_ID_FROM_CONTRACT2 => match reply.result {
            SubMsgResult::Ok(_g) => {
                REQUEST_STATUS.save(deps.storage, &RequestStatus::Accepted)?;

                Ok(Response::new().add_attribute("reply_from", "currency_hub_address"))
            },
            SubMsgResult::Err(err) => {
                deps.api.debug("WASMDEBUG: reply REPLY_ID_CONTRACT2 Fail");
                REQUEST_STATUS.save(deps.storage, &RequestStatus::Failed)?;
                Ok(Response::new().set_data(make_ack_fail(err)))
            }
        },
        _ => Err(ContractError::UnknownReplyId { id: reply.id }),
    }
}

#[cfg(test)]
mod tests {}
