use crate::web3_domains::naming_service_traits::NamingServiceTrait;
use crate::web3_domains::utils::configs;
use ethereum_types::BigEndianHash;
use sha3::{Digest, Keccak256};
use web3;
use web3::contract::{Contract, Options};

use web3::transports::Http;
use web3::types::{Address, H256};

use std::path::Path;

pub struct EnsNamingService {
    registry_contract: Contract<Http>,
    resolver_contract: Contract<Http>,
    name_wrapper_contract: Contract<Http>,
}

impl EnsNamingService {
    async fn load_contract_addresses(network_id: usize) -> Option<(String, String, String)> {
        let current_file_path = file!();
        let current_dir = Path::new(current_file_path).parent().unwrap();
        let config_file_path = current_dir.join("ens_config.json");

        match configs::load_config_file(config_file_path).await {
            Ok(data) => {
                let network = &data["networks"][network_id.to_string()]["contracts"];

                let registry_address = network["ENSRegistry"]["address"].as_str()?.to_string();
                let resolver_address = network["PublicResolver"]["address"].as_str()?.to_string();
                let name_wrapper_address = network["NameWrapper"]["address"].as_str()?.to_string();

                Some((registry_address, resolver_address, name_wrapper_address))
            }
            Err(e) => {
                println!("Error: {}", e);
                None
            }
        }
    }

    pub async fn new(eth_rpc_url: String) -> Self {
        let contract_addresses = Self::load_contract_addresses(1).await;
        let (registry_address, resolver_address, name_wrapper_address) =
            contract_addresses.unwrap();

        let eth_transport = web3::transports::Http::new(&eth_rpc_url).unwrap();
        let eth_provider = web3::Web3::new(eth_transport);

        let registry_contract = Contract::from_json(
            eth_provider.eth(),
            registry_address.as_str().parse().unwrap(),
            include_bytes!("./registry_abis.json"),
        )
        .unwrap();

        let resolver_contract = Contract::from_json(
            eth_provider.eth(),
            resolver_address.as_str().parse().unwrap(),
            include_bytes!("./resolver_abis.json"),
        )
        .unwrap();

        let name_wrapper_contract = Contract::from_json(
            eth_provider.eth(),
            name_wrapper_address.as_str().parse().unwrap(),
            include_bytes!("./name_wrapper_abis.json"),
        )
        .unwrap();

        EnsNamingService {
            registry_contract,
            resolver_contract,
            name_wrapper_contract,
        }
    }

    fn ens_namehash(domain: &str) -> Option<H256> {
        let parts = domain.split('.').collect::<Vec<&str>>();

        let mut node = H256::zero();

        for label in parts.iter().rev() {
            let mut hasher = Keccak256::new();
            hasher.update(label.as_bytes());
            let label_hash = hasher.finalize();

            let mut hasher = Keccak256::new();
            hasher.update(&[node.as_bytes(), label_hash.as_slice()].concat());
            node = H256::from_slice(hasher.finalize().as_slice());
        }

        Some(H256::from_slice(&node.as_bytes()))
    }
}

#[async_trait::async_trait]
impl NamingServiceTrait for EnsNamingService {
    fn namehash(&self, domain: &str) -> Option<String> {
        let namehash = EnsNamingService::ens_namehash(domain).unwrap();

        Some(format!("0x{:x}", namehash))
    }

    async fn owner(&self, domain: &str) -> Option<String> {
        let namehash = EnsNamingService::ens_namehash(domain).unwrap();

        let mut registry_owner: Result<Address, web3::contract::Error> = self
            .registry_contract
            .query("owner", (namehash,), None, Options::default(), None)
            .await;

        if !registry_owner.is_ok() {
            return Some(Address::zero().to_string());
        }

        let registry_owner = registry_owner.unwrap();

        if registry_owner.is_zero() {
            return Some(Address::zero().to_string());
        }

        let name_wrapper_owner: Result<Address, web3::contract::Error> = self
            .name_wrapper_contract
            .query(
                "ownerOf",
                (namehash.into_uint(),),
                None,
                Options::default(),
                None,
            )
            .await;

        if !name_wrapper_owner.is_ok() {
            return Some(format!("{:#x}", registry_owner));
        }

        let owner = name_wrapper_owner.unwrap();
        if !owner.is_zero() {
            return Some(format!("{:#x}", owner));
        }

        return Some(format!("{:#x}", registry_owner));
    }
}
