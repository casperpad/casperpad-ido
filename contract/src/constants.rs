// KEY_NAMES
pub const CONTRACT_NAME_KEY_NAME: &str = "casper_ido";
pub const OWNER_KEY_NAME: &str = "owner";
pub const TREASURY_WALLET_KEY_NAME: &str = "treasury_wallet";
pub const DEFAULT_TREASURY_WALLET_KEY_NAME: &str = "default_treasury_wallet";
pub const PROJECTS_KEY_NAME: &str = "projects";
// RUNTIME_NAMES
pub const DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "default_treasury_wallet";
pub const OWNER_RUNTIME_ARG_NAME: &str = "owner";
pub const PROJECT_RUNTIME_ARG_NAME: &str = "project";
pub const WALLET_RUNTIME_ARG_NAME: &str = "wallet";

// runtime arguments for create project
pub const PROJECT_ID_RUNTIME_ARG_NAME: &str = "id";
pub const PROJECT_NAME_RUNTIME_ARG_NAME: &str = "name";
pub const PROJECT_START_TIME_RUNTIME_ARG_NAME: &str = "start_time";
pub const PROJECT_END_TIME_RUNTIME_ARG_NAME: &str = "end_time";
pub const PROJECT_PRIVATE_RUNTIME_ARG_NAME: &str = "private";
pub const PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME: &str = "token_symbol";
pub const PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME: &str = "token_total_supply";
pub const PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME: &str = "token_price";

// ENTRY_POINT_NAMES
pub const TRANSFER_OWNERSHIP_ENRTY_NAME: &str = "transfer_ownership";
pub const GET_OWNER_ENTRY_NAME: &str = "get_owner";
pub const SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME: &str = "set_default_treasury_wallet";
pub const SET_PROJECT_TREASURY_WALLET_ENTRY_NAME: &str = "set_project_treasury_wallet";
pub const CREATE_PROJECT_ENTRY_NAME: &str = "add_project";
