use cosmwasm_std::{
    to_json_binary, Addr, Binary, BlockInfo, Deps, DepsMut, Env, Order, QueryRequest, Record,
    StdError, StdResult, WasmQuery,
};

use archid_token::{Extension, Metadata, QueryMsg as Cw721QueryMsg};
use cw721_updatable::{NftInfoResponse, OwnerOfResponse};

use crate::error::ContractError;
use crate::msg::{RecordExpirationResponse, ResolveAddressResponse, ResolveRecordResponse};
use crate::state::{resolver_read, NameRecord};

const MIN_NAME_LENGTH: u64 = 3;
const MAX_NAME_LENGTH: u64 = 64;
const SUFFIX: &str = ".arch";
pub fn query_name_owner(
    id: &str,
    cw721: &Addr,
    deps: &DepsMut,
) -> Result<OwnerOfResponse, StdError> {
    let query_msg: archid_token::QueryMsg<Extension> = Cw721QueryMsg::OwnerOf {
        token_id: id.to_owned(),
        include_expired: None,
    };
    let req = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cw721.to_string(),
        msg: to_json_binary(&query_msg).unwrap(),
    });
    let res: OwnerOfResponse = deps.querier.query(&req)?;
    Ok(res)
}

pub fn query_resolver(deps: Deps, env: Env, name: String) -> StdResult<Binary> {
    let key = name.as_bytes();
    let curr = (resolver_read(deps.storage).may_load(key)?).unwrap();

    let address = match curr.is_expired(&env.block) {
        true => None,
        false => Some(String::from(&curr.resolver)),
    };

    let resp = ResolveRecordResponse {
        address,
        expiration: curr.expiration,
    };
    to_json_binary(&resp)
}

pub fn query_resolver_expiration(deps: Deps, _env: Env, name: String) -> StdResult<Binary> {
    let key = name.as_bytes();
    let curr = (resolver_read(deps.storage).may_load(key)?).unwrap();
    let resp = RecordExpirationResponse {
        created: curr.created,
        expiration: curr.expiration,
    };
    to_json_binary(&resp)
}

pub fn query_resolver_address(deps: Deps, env: Env, address: Addr) -> StdResult<Binary> {
    let curr: StdResult<Vec<Record<NameRecord>>> = resolver_read(deps.storage)
        .range(None, None, Order::Ascending)
        .collect();

    let records = curr.unwrap();

    let names = records
        .into_iter()
        .filter(|(_i, record)| record.resolver == address)
        .collect::<Vec<Record<NameRecord>>>();

    let unexpired_names = names
        .into_iter()
        .filter(|(_i, record)| !record.is_expired(&env.block))
        .collect::<Vec<Record<NameRecord>>>();

    let mut output_names = vec![];
    for (key, _record) in unexpired_names.into_iter() {
        output_names.push(String::from_utf8(key).unwrap());
    }

    let resp = ResolveAddressResponse {
        names: Some(output_names.clone()),
    };
    to_json_binary(&resp)
}

pub fn query_current_metadata(
    id: &str,
    cw721: &Addr,
    deps: &DepsMut,
) -> Result<Metadata, StdError> {
    let query_msg: archid_token::QueryMsg<Extension> = Cw721QueryMsg::NftInfo {
        token_id: id.to_owned(),
    };
    let req = QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: cw721.to_string(),
        msg: to_json_binary(&query_msg).unwrap(),
    });
    let res: NftInfoResponse<Metadata> = deps.querier.query(&req)?;
    Ok(res.extension)
}
fn invalid_char(c: char) -> bool {
    let is_valid = c.is_ascii_digit() || c.is_ascii_lowercase() || (c == '-' || c == '_');
    !is_valid
}

pub fn is_expired(deps: &DepsMut, key: &[u8], block: &BlockInfo) -> bool {
    let r = resolver_read(deps.storage).may_load(key).unwrap();
    match r.is_some() {
        true => r.unwrap().is_expired(block),
        _ => true,
    }
}

/// validate_name returns an error if the name is invalid
/// (we require 3-64 lowercase ascii letters, numbers, or . - _)
pub fn validate_name(name: &str) -> Result<(), ContractError> {
    let length = name.len() as u64;
    let suffix_index = length as usize - SUFFIX.len();
    let body = &name[0..suffix_index];
    if (body.len() as u64) < MIN_NAME_LENGTH {
        Err(ContractError::NameTooShort {
            length,
            min_length: MIN_NAME_LENGTH,
        })
    } else if (body.len() as u64) > MAX_NAME_LENGTH {
        Err(ContractError::NameTooLong {
            length,
            max_length: MAX_NAME_LENGTH,
        })
    } else {
        match body.find(invalid_char) {
            None => Ok(()),
            Some(bytepos_invalid_char_start) => {
                let c = name[bytepos_invalid_char_start..].chars().next().unwrap();
                Err(ContractError::InvalidCharacter { c })
            }
        }
    }
}
pub fn validate_subdomain(name: &str) -> Result<(), ContractError> {
    let length = name.len() as u64;
    if (name.len() as u64) < MIN_NAME_LENGTH {
        Err(ContractError::NameTooShort {
            length,
            min_length: MIN_NAME_LENGTH,
        })
    } else if (name.len() as u64) > MAX_NAME_LENGTH {
        Err(ContractError::NameTooLong {
            length,
            max_length: MAX_NAME_LENGTH,
        })
    } else {
        match name.find(invalid_char) {
            None => Ok(()),
            Some(bytepos_invalid_char_start) => {
                let c = name[bytepos_invalid_char_start..].chars().next().unwrap();
                Err(ContractError::InvalidCharacter { c })
            }
        }
    }
}
pub fn format_name(name: String) -> String {
    let domain_route = format!("{}{}", name, String::from(SUFFIX));
    domain_route
}
pub fn get_name_body(name: String) -> String {
    let length = name.len() as u64;
    let suffix_index = length as usize - SUFFIX.len();
    let body = &name[0..suffix_index];
    String::from(body)
}
pub fn get_subdomain_prefix(name: String) -> Option<Vec<String>> {
    let body = get_name_body(name);
    let components: Vec<_> = body.split('.').collect();
    match components.len() {
        1 => None,
        2 => Some(vec![
            String::from(components[0]),
            String::from(components[1]),
        ]),
        _ => None,
    }
}
