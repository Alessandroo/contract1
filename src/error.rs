use cosmwasm_std::{StdError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Got a submessage reply with unknown id: {id}")]
    UnknownReplyId { id: u64 },

    #[error("Failed to parse Uint128")]
    Uint128ParseError,

    #[error("Arithmetic error")]
    ArithmeticError,

    #[error("Arithmetic error")]
    BadRequestError,
}