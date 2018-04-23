use hyper::status::StatusCode;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum ApiError {
    TransactionNotFound,
    TransactionHashInvalid,
    AssetIdNotFound,
    AssetIdInvalid,
    EmptyRequestBody,
    IncorrectRequest,
    WalletHexInvalid,
    HeightIsMissing,
    BlockNotFound,
}

impl ApiError {
    pub fn to_status(&self) -> StatusCode {
        match *self {
            ApiError::TransactionNotFound => StatusCode::NotFound,
            ApiError::TransactionHashInvalid => StatusCode::BadRequest,
            ApiError::AssetIdNotFound => StatusCode::NotFound,
            ApiError::AssetIdInvalid => StatusCode::BadRequest,
            ApiError::EmptyRequestBody => StatusCode::BadRequest,
            ApiError::IncorrectRequest => StatusCode::BadRequest,
            ApiError::WalletHexInvalid => StatusCode::BadRequest,
            ApiError::HeightIsMissing => StatusCode::BadRequest,
            ApiError::BlockNotFound => StatusCode::NotFound,
        }
    }
}
