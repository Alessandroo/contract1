use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub query_denom: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    Start {
        contract2_addr: String,
        query_address: String,
    },
}

#[cw_serde]
pub enum Contract2ExecuteMsg {
    TriggerFlow {
        query_address: String,
        query_denom: String,
    },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(GetBalanceResponse)]
    BalanceInfo{},
}

#[cw_serde]
pub struct GetBalanceResponse {
    pub balance: String,
}