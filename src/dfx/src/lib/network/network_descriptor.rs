use crate::lib::network::local_server_descriptor::LocalServerDescriptor;
use dfx_core::config::model::dfinity::NetworkType;
use dfx_core::config::model::dfinity::{DEFAULT_IC_GATEWAY, DEFAULT_IC_GATEWAY_TRAILING_SLASH};
use dfx_core::error::network_config::NetworkConfigError;
use dfx_core::error::network_config::NetworkConfigError::{
    NetworkHasNoProviders, NetworkMustBeLocal,
};

use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkTypeDescriptor {
    Ephemeral { wallet_config_path: PathBuf },

    Persistent,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkDescriptor {
    pub name: String,
    pub providers: Vec<String>,
    pub r#type: NetworkTypeDescriptor,
    pub is_ic: bool,
    pub local_server_descriptor: Option<LocalServerDescriptor>,
}

impl NetworkTypeDescriptor {
    pub fn new(r#type: NetworkType, ephemeral_wallet_config_path: &Path) -> Self {
        match r#type {
            NetworkType::Ephemeral => NetworkTypeDescriptor::Ephemeral {
                wallet_config_path: ephemeral_wallet_config_path.to_path_buf(),
            },
            NetworkType::Persistent => NetworkTypeDescriptor::Persistent,
        }
    }
}

impl NetworkDescriptor {
    pub fn ic() -> Self {
        NetworkDescriptor {
            name: "ic".to_string(),
            providers: vec![DEFAULT_IC_GATEWAY.to_string()],
            r#type: NetworkTypeDescriptor::Persistent,
            is_ic: true,
            local_server_descriptor: None,
        }
    }

    /// Determines whether the provided connection is the official IC or not.
    #[allow(clippy::ptr_arg)]
    pub fn is_ic(network_name: &str, providers: &Vec<String>) -> bool {
        let name_match = matches!(
            network_name,
            "ic" | DEFAULT_IC_GATEWAY | DEFAULT_IC_GATEWAY_TRAILING_SLASH
        );
        let provider_match = {
            providers.len() == 1
                && matches!(
                    providers.get(0).unwrap().as_str(),
                    DEFAULT_IC_GATEWAY | DEFAULT_IC_GATEWAY_TRAILING_SLASH
                )
        };
        name_match || provider_match
    }

    /// Return the first provider in the list
    pub fn first_provider(&self) -> Result<&str, NetworkConfigError> {
        match self.providers.first() {
            Some(provider) => Ok(provider),
            None => Err(NetworkHasNoProviders(self.name.clone())),
        }
    }

    pub fn local_server_descriptor(&self) -> Result<&LocalServerDescriptor, NetworkConfigError> {
        match &self.local_server_descriptor {
            Some(p) => Ok(p),
            None => Err(NetworkMustBeLocal(self.name.clone())),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ic_by_netname() {
        assert!(NetworkDescriptor::is_ic("ic", &vec![]));
        assert!(NetworkDescriptor::is_ic(DEFAULT_IC_GATEWAY, &vec![]));
        assert!(NetworkDescriptor::is_ic(
            DEFAULT_IC_GATEWAY_TRAILING_SLASH,
            &vec![]
        ));
    }

    #[test]
    fn ic_by_provider() {
        assert!(NetworkDescriptor::is_ic(
            "not_ic",
            &vec![DEFAULT_IC_GATEWAY.to_string()]
        ));
        assert!(NetworkDescriptor::is_ic(
            "not_ic",
            &vec![DEFAULT_IC_GATEWAY_TRAILING_SLASH.to_string()]
        ));
    }

    #[test]
    fn ic_by_netname_fail() {
        assert!(!NetworkDescriptor::is_ic("not_ic", &vec![]));
    }

    #[test]
    fn ic_by_provider_fail_string() {
        assert!(!NetworkDescriptor::is_ic(
            "not_ic",
            &vec!["not_ic_provider".to_string()]
        ));
    }

    #[test]
    fn ic_by_provider_fail_unique() {
        assert!(!NetworkDescriptor::is_ic(
            "not_ic",
            &vec![
                DEFAULT_IC_GATEWAY.to_string(),
                "some_other_provider".to_string()
            ]
        ));
    }
}
