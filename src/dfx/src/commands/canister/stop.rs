use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister;
use crate::lib::root_key::fetch_root_key_if_needed;

use candid::Principal;
use clap::Parser;
use slog::info;

/// Stops a currently running canister.
#[derive(Parser)]
pub struct CanisterStopOpts {
    /// Specifies the name or id of the canister to stop.
    /// You must specify either a canister name/id or the --all option.
    canister: Option<String>,

    /// Stops all of the canisters configured in the dfx.json file.
    #[clap(long, required_unless_present("canister"))]
    all: bool,
}

async fn stop_canister(
    env: &dyn Environment,
    canister: &str,
    call_sender: &CallSender,
) -> DfxResult {
    let log = env.get_logger();
    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id =
        Principal::from_text(canister).or_else(|_| canister_id_store.get(canister))?;

    info!(
        log,
        "Stopping code for canister {}, with canister_id {}",
        canister,
        canister_id.to_text(),
    );

    canister::stop_canister(env, canister_id, call_sender).await?;

    Ok(())
}

pub async fn exec(
    env: &dyn Environment,
    opts: CanisterStopOpts,
    call_sender: &CallSender,
) -> DfxResult {
    let config = env.get_config_or_anyhow()?;

    fetch_root_key_if_needed(env).await?;

    if let Some(canister) = opts.canister.as_deref() {
        stop_canister(env, canister, call_sender).await
    } else if opts.all {
        if let Some(canisters) = &config.get_config().canisters {
            for canister in canisters.keys() {
                stop_canister(env, canister, call_sender).await?;
            }
        }
        Ok(())
    } else {
        unreachable!()
    }
}
