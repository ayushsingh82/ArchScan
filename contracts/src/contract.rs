use crate::error::ContractError;
use crate::handlers::{
    execute_extend_subdomain_expiry, execute_register, execute_remove_subdomain,
    execute_renew_registration, execute_set_subdomain, execute_update_config,
    execute_update_resolver, execute_user_metadata_update, execute_withdraw_fees,
};
use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::read_utils::{
    format_name, query_resolver, query_resolver_address, query_resolver_expiration,
};
use crate::state::{config, config_read, Config};

use archid_token::Metadata;

use cosmwasm_std::{
    entry_point, to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response,
    StdError, StdResult, SubMsgResult,
};

use cw2::{get_contract_version, set_contract_version};

pub type NameExtension = Option<Metadata>;

const CONTRACT_NAME: &str = "crates.io:archid-registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, StdError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let config_state = Config {
        admin: msg.admin,
        wallet: msg.wallet,
        cw721: msg.cw721,
        base_cost: msg.base_cost,
        base_expiration: msg.base_expiration,
    };
    config(deps.storage).save(&config_state)?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Register { name } => execute_register(deps, env, info, format_name(name)),
        ExecuteMsg::RenewRegistration { name } => {
            execute_renew_registration(deps, env, info, format_name(name))
        }
        ExecuteMsg::UpdateResolver { name, new_resolver } => {
            execute_update_resolver(info, deps, env, format_name(name), new_resolver)
        }
        ExecuteMsg::RegisterSubdomain {
            domain,
            subdomain,
            new_resolver,
            new_owner,
            expiration,
        } => execute_set_subdomain(
            info,
            deps,
            env,
            format_name(domain),
            subdomain,
            new_resolver,
            new_owner,
            expiration,
        ),
        ExecuteMsg::ExtendSubdomainExpiry {
            domain,
            subdomain,
            expiration,
        } => execute_extend_subdomain_expiry(
            info,
            deps,
            env,
            format_name(domain),
            subdomain,
            expiration,
        ),
        ExecuteMsg::UpdateUserDomainData {
            name,
            metadata_update,
        } => execute_user_metadata_update(info, deps, format_name(name), metadata_update),

        ExecuteMsg::UpdateConfig { config } => execute_update_config(deps, env, info, config),

        ExecuteMsg::Withdraw { amount } => execute_withdraw_fees(info, deps, amount),

        ExecuteMsg::RemoveSubdomain { domain, subdomain } => {
            execute_remove_subdomain(info, deps, env, format_name(domain), subdomain)
        }
    }
}
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ResolveRecord { name } => query_resolver(deps, env, name),
        QueryMsg::ResolveAddress { address } => query_resolver_address(deps, env, address),
        QueryMsg::RecordExpiration { name } => query_resolver_expiration(deps, env, name),
        QueryMsg::Config {} => to_json_binary(&config_read(deps.storage).load()?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match msg.result {
        SubMsgResult::Ok(_) => Ok(Response::default()),
        SubMsgResult::Err(_) => Err(ContractError::Unauthorized {}),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    let original_version = get_contract_version(deps.storage)?;
    let name = CONTRACT_NAME.to_string();
    let version = CONTRACT_VERSION.to_string();
    if original_version.contract != name {
        return Err(ContractError::InvalidInput {});
    }
    if original_version.version >= version {
        return Err(ContractError::InvalidInput {});
    }
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::default())
}
