use cosmwasm_schema::{cw_serde, QueryResponses};
use crate::state::RequestStatus;

#[cw_serde]
pub struct InstantiateMsg {
    pub currency_hub: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    RequestTokenPrice {
        base_asset_denom: String,
        quote_asset_denom: String,
        query_address: String,
    },
    ResponseTokenPrice {
        base_asset_denom: String,
        quote_asset_denom: String,
        arithmetic_twap: String,
    },
}

#[cw_serde]
pub enum Contract2ExecuteMsg {
    GetTokenPrice {
        base_asset_denom: String,
        quote_asset_denom: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetCalculatedBalanceResponse)]
    CalculatedBalance{},

    #[returns(GetRequestStatusResponse)]
    RequestStatus{},
}

#[cw_serde]
pub struct GetCalculatedBalanceResponse {
    pub query_address: String,
    pub original_balance: String,
    pub exchanged_balance: String,
}

#[cw_serde]
pub struct GetRequestStatusResponse {
    pub request_status: RequestStatus,
}