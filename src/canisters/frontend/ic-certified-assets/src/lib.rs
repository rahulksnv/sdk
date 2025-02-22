//! This module declares canister methods expected by the assets canister client.
pub mod http;
pub mod rc_bytes;
pub mod state_machine;
pub mod types;
mod url_decode;

#[cfg(test)]
mod tests;

pub use crate::state_machine::StableState;
use crate::{
    http::{HttpRequest, HttpResponse, StreamingCallbackHttpResponse, StreamingCallbackToken},
    rc_bytes::RcBytes,
    state_machine::{AssetDetails, CertifiedTree, EncodedAsset, State},
    types::*,
};
use candid::{candid_method, Principal};
use ic_cdk::api::{
    call::ManualReply,
    caller, data_certificate,
    management_canister::{main::canister_status, provisional::CanisterIdRecord},
    set_certified_data, time, trap,
};
use ic_cdk_macros::{query, update};
use std::cell::RefCell;

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

#[update]
#[candid_method(update)]
async fn authorize(other: Principal) {
    match has_permission_or_is_controller(&Permission::ManagePermissions).await {
        Err(e) => trap(&e),
        Ok(_) => STATE.with(|s| s.borrow_mut().grant_permission(other, &Permission::Commit)),
    }
}

#[update]
#[candid_method(update)]
async fn grant_permission(arg: GrantPermissionArguments) {
    match has_permission_or_is_controller(&Permission::ManagePermissions).await {
        Err(e) => trap(&e),
        Ok(_) => STATE.with(|s| {
            s.borrow_mut()
                .grant_permission(arg.to_principal, &arg.permission)
        }),
    }
}

#[update]
#[candid_method(update)]
async fn validate_grant_permission(arg: GrantPermissionArguments) -> Result<String, String> {
    Ok(format!(
        "grant {} permission to principal {}",
        arg.permission, arg.to_principal
    ))
}

#[update]
#[candid_method(update)]
async fn deauthorize(other: Principal) {
    let check_access_result = if other == caller() {
        // this isn't "ManagePermissions" because these legacy methods only
        // deal with the Commit permission
        has_permission_or_is_controller(&Permission::Commit).await
    } else {
        is_controller().await
    };
    match check_access_result {
        Err(e) => trap(&e),
        Ok(_) => STATE.with(|s| s.borrow_mut().revoke_permission(other, &Permission::Commit)),
    }
}

#[update]
#[candid_method(update)]
async fn revoke_permission(arg: RevokePermissionArguments) {
    let check_access_result = if arg.of_principal == caller() {
        has_permission_or_is_controller(&arg.permission).await
    } else {
        has_permission_or_is_controller(&Permission::ManagePermissions).await
    };
    match check_access_result {
        Err(e) => trap(&e),
        Ok(_) => STATE.with(|s| {
            s.borrow_mut()
                .revoke_permission(arg.of_principal, &arg.permission)
        }),
    }
}

#[update]
#[candid_method(update)]
async fn validate_revoke_permission(arg: RevokePermissionArguments) -> Result<String, String> {
    Ok(format!(
        "revoke {} permission from principal {}",
        arg.permission, arg.of_principal
    ))
}

#[query(manual_reply = true)]
#[candid_method(query)]
fn list_authorized() -> ManualReply<Vec<Principal>> {
    STATE.with(|s| ManualReply::one(s.borrow().list_permitted(&Permission::Commit)))
}

#[query(manual_reply = true)]
#[candid_method(query)]
fn list_permitted(arg: ListPermittedArguments) -> ManualReply<Vec<Principal>> {
    STATE.with(|s| ManualReply::one(s.borrow().list_permitted(&arg.permission)))
}

#[update]
#[candid_method(update)]
async fn take_ownership() {
    let caller = caller();
    match is_controller().await {
        Err(e) => trap(&e),
        Ok(_) => STATE.with(|s| s.borrow_mut().take_ownership(caller)),
    }
}

#[query]
#[candid_method(query)]
fn retrieve(key: Key) -> RcBytes {
    STATE.with(|s| match s.borrow().retrieve(&key) {
        Ok(bytes) => bytes,
        Err(msg) => trap(&msg),
    })
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn store(arg: StoreArg) {
    STATE.with(move |s| {
        if let Err(msg) = s.borrow_mut().store(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    });
}

#[update(guard = "can_prepare")]
#[candid_method(update)]
fn create_batch() -> CreateBatchResponse {
    STATE.with(|s| CreateBatchResponse {
        batch_id: s.borrow_mut().create_batch(time()),
    })
}

#[update(guard = "can_prepare")]
#[candid_method(update)]
fn create_chunk(arg: CreateChunkArg) -> CreateChunkResponse {
    STATE.with(|s| match s.borrow_mut().create_chunk(arg, time()) {
        Ok(chunk_id) => CreateChunkResponse { chunk_id },
        Err(msg) => trap(&msg),
    })
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn create_asset(arg: CreateAssetArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().create_asset(arg) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    })
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn set_asset_content(arg: SetAssetContentArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().set_asset_content(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    })
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn unset_asset_content(arg: UnsetAssetContentArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().unset_asset_content(arg) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    })
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn delete_asset(arg: DeleteAssetArguments) {
    STATE.with(|s| {
        s.borrow_mut().delete_asset(arg);
        set_certified_data(&s.borrow().root_hash());
    });
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn clear() {
    STATE.with(|s| {
        s.borrow_mut().clear();
        set_certified_data(&s.borrow().root_hash());
    });
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn commit_batch(arg: CommitBatchArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().commit_batch(arg, time()) {
            trap(&msg);
        }
        set_certified_data(&s.borrow().root_hash());
    });
}

#[query]
#[candid_method(query)]
fn get(arg: GetArg) -> EncodedAsset {
    STATE.with(|s| match s.borrow().get(arg) {
        Ok(asset) => asset,
        Err(msg) => trap(&msg),
    })
}

#[query]
#[candid_method(query)]
fn get_chunk(arg: GetChunkArg) -> GetChunkResponse {
    STATE.with(|s| match s.borrow().get_chunk(arg) {
        Ok(content) => GetChunkResponse { content },
        Err(msg) => trap(&msg),
    })
}

#[query]
#[candid_method(query)]
fn list() -> Vec<AssetDetails> {
    STATE.with(|s| s.borrow().list_assets())
}

#[query]
#[candid_method(query)]
fn certified_tree() -> CertifiedTree {
    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    STATE.with(|s| s.borrow().certified_tree(&certificate))
}

#[query]
#[candid_method(query)]
fn http_request(req: HttpRequest) -> HttpResponse {
    let certificate = data_certificate().unwrap_or_else(|| trap("no data certificate available"));

    STATE.with(|s| {
        s.borrow().http_request(
            req,
            &certificate,
            candid::Func {
                method: "http_request_streaming_callback".to_string(),
                principal: ic_cdk::id(),
            },
        )
    })
}

#[query]
#[candid_method(query)]
fn http_request_streaming_callback(token: StreamingCallbackToken) -> StreamingCallbackHttpResponse {
    STATE.with(|s| {
        s.borrow()
            .http_request_streaming_callback(token)
            .unwrap_or_else(|msg| trap(&msg))
    })
}

#[query]
#[candid_method(query)]
fn get_asset_properties(key: Key) -> AssetProperties {
    STATE.with(|s| {
        s.borrow()
            .get_asset_properties(key)
            .unwrap_or_else(|msg| trap(&msg))
    })
}

#[update(guard = "can_commit")]
#[candid_method(update)]
fn set_asset_properties(arg: SetAssetPropertiesArguments) {
    STATE.with(|s| {
        if let Err(msg) = s.borrow_mut().set_asset_properties(arg) {
            trap(&msg);
        }
    })
}

fn can(permission: Permission) -> Result<(), String> {
    STATE.with(|s| {
        s.borrow()
            .can(&caller(), &permission)
            .then_some(())
            .ok_or_else(|| format!("Caller does not have {} permission", permission))
    })
}

fn can_commit() -> Result<(), String> {
    can(Permission::Commit)
}

fn can_prepare() -> Result<(), String> {
    can(Permission::Prepare)
}

async fn has_permission_or_is_controller(permission: &Permission) -> Result<(), String> {
    let caller = caller();
    let has_permission = STATE.with(|s| s.borrow().has_permission(&caller, permission));
    if has_permission {
        Ok(())
    } else {
        match canister_status(CanisterIdRecord {
            canister_id: ic_cdk::api::id(),
        })
        .await
        {
            Err((code, msg)) => trap(&format!(
                "Caller does not have {} permission. Failed to determine if caller is canister controller with code {:?} and message '{}'",
                permission,
                code, msg
            )),
            Ok((a,)) => {
                if a.settings.controllers.contains(&caller) {
                    Ok(())
                } else {
                    Err(format!("Caller does not have {} permission and is not a controller.", permission))
                }
            }
        }
    }
}

async fn is_controller() -> Result<(), String> {
    let caller = caller();
    match canister_status(CanisterIdRecord {
        canister_id: ic_cdk::api::id(),
    })
    .await
    {
        Err((code, msg)) => trap(&format!(
            "Failed to determine if caller is canister controller with code {:?} and message '{}'",
            code, msg
        )),
        Ok((a,)) => {
            if a.settings.controllers.contains(&caller) {
                Ok(())
            } else {
                Err("Caller is not a controller.".to_string())
            }
        }
    }
}

pub fn init() {
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.clear();
        s.grant_permission(caller(), &Permission::Commit);
    });
}

pub fn pre_upgrade() -> StableState {
    STATE.with(|s| s.take().into())
}

pub fn post_upgrade(stable_state: StableState) {
    STATE.with(|s| {
        *s.borrow_mut() = State::from(stable_state);
        set_certified_data(&s.borrow().root_hash());
    });
}

#[test]
fn candid_interface_compatibility() {
    use candid::utils::{service_compatible, CandidSource};
    use std::path::PathBuf;

    candid::export_service!();
    let new_interface = __export_service();

    let old_interface =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets.did");

    println!("Exported interface: {}", new_interface);

    service_compatible(
        CandidSource::Text(&new_interface),
        CandidSource::File(old_interface.as_path()),
    )
    .expect("The assets canister interface is not compatible with the assets.did file");
}
