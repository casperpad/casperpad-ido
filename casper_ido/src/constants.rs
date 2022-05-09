// KEY_NAMES
pub const CONTRACT_NAME_KEY_NAME: &str = "casper_ido";
pub const OWNER_KEY_NAME: &str = "owner";
pub const DEFAULT_TREASURY_WALLET_KEY_NAME: &str = "default_treasury_wallet";
pub const PROJECTS_KEY_NAME: &str = "projects";
pub const INVESTS_KEY_NAME: &str = "invests";
pub const CLAIMS_KEY_NAME: &str = "claims";
pub const RESULT_KEY_NAME: &str = "result";
pub const MERKLE_ROOT_KEY_NAME: &str = "merkle_root";
pub const PURSE_KEY_NAME: &str = "purse";
// RUNTIME_NAMES
pub const DEFAULT_TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "default_treasury_wallet";
pub const OWNER_RUNTIME_ARG_NAME: &str = "owner";
pub const CSPR_AMOUNT_RUNTIME_ARG_NAME: &str = "cspr_amount";

// runtime arguments for create project
pub const PROJECT_ID_RUNTIME_ARG_NAME: &str = "id";
pub const PROJECT_NAME_RUNTIME_ARG_NAME: &str = "name";
pub const PROJECT_SALE_START_TIME_RUNTIME_ARG_NAME: &str = "sale_start_time";
pub const PROJECT_SALE_END_TIME_RUNTIME_ARG_NAME: &str = "sale_end_time";
pub const PROJECT_OPEN_TIME_RUNTIME_ARG_NAME: &str = "open_time";
pub const PROJECT_PRIVATE_RUNTIME_ARG_NAME: &str = "private";
pub const PROJECT_TOKEN_ADDRESS_RUNTIME_ARG_NAME: &str = "token_address";
pub const PROJECT_TOKEN_SYMBOL_RUNTIME_ARG_NAME: &str = "token_symbol";
pub const PROJECT_TOKEN_PRICE_USD_RUNTIME_ARG_NAME: &str = "token_price";
pub const PROJECT_TOKEN_TOTAL_SUPPLY_RUNTIME_ARG_NAME: &str = "token_total_supply";
pub const PROJECT_CLAIM_STATUS_RUNTIME_ARG_NAME: &str = "claim_status";
pub const PROJECT_USERS_LENGTH_RUNTIME_ARG_NAME: &str = "users_length";
pub const PROJECT_CAPACITY_USD_RUNTIME_ARG_NAME: &str = "capacity_usd";
pub const PROJECT_SCHEDULES_RUNTIME_ARG_NAME: &str = "schedules";
pub const PROJECT_STATUS_RUNTIME_ARG_NAME: &str = "status";
pub const PROJECT_REWARD_MULTIPLY_RUNTIME_ARG_NAME: &str = "reward_multiply";
pub const PROJECT_LOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME: &str = "locked_token_amount";
pub const PROJECT_UNLOCKED_TOKEN_AMOUNT_RUNTIME_ARG_NAME: &str = "unlocked_token_amount";
pub const PROJECT_USERS_RUNTIME_ARG_NAME: &str = "users";
pub const TREASURY_WALLET_RUNTIME_ARG_NAME: &str = "treasury_wallet";
pub const MERKLE_ROOT_RUNTIME_ARG_NAME: &str = "merkle_root";
pub const PROOF_RUNTIME_ARG_NAME: &str = "proof";
pub const SCHEDULE_ID_RUNTIME_ARG_NAME: &str = "schedule_id";
// ENTRY_POINT_NAMES
pub const TRANSFER_OWNERSHIP_ENRTY_NAME: &str = "transfer_ownership";
pub const SET_DEFAULT_TREASURY_WALLET_ENTRY_NAME: &str = "set_default_treasury_wallet";
pub const CREATE_PROJECT_ENTRY_NAME: &str = "add_project";
pub const ADD_INVEST_ENTRY_NAME: &str = "add_invest";
pub const SET_PROJECT_STATUS_ENTRY_NAME: &str = "set_project_status";
pub const SET_MERKLE_ROOT_ENTRY_NAME: &str = "set_merkle_root";
pub const CLAIM_ENTRY_NAME: &str = "claim";
pub const GET_PURSE_ENTRY_NAME: &str = "get_purse";
