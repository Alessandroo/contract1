use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Deps, Uint128};
use cw_storage_plus::Item;

pub const CURRENCY_HUB_ADDRESS: Item<Addr> = Item::new("currency_hub_address");

pub const SENT_REQUEST: Item<ContractRequest> = Item::new("sent_request");

pub const EXCHANGE_RATE: Item<Uint128> = Item::new("exchange_rate");

pub const REQUEST_STATUS: Item<RequestStatus> = Item::new("request_status");

#[cw_serde]
#[derive(Default)]
pub enum RequestStatus {
    #[default]
    None,
    Requested,
    Accepted,
    Answered,
    Failed,
}

#[cw_serde]
pub struct ContractRequest {
    pub base_asset_denom: String,
    pub quote_asset_denom: String,
    pub query_address: Addr,
}

pub fn is_currency_hub(deps: Deps, sender: &Addr) -> bool {
    CURRENCY_HUB_ADDRESS
        .load(deps.storage)
        .map(|currency_hub| currency_hub == sender)
        .map_err(|_| false)
        .unwrap_or(false)
}
