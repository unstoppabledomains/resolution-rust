use crate::web3_domains::naming_service_traits::NamingServiceTrait;
use crate::web3_domains::utils::configs;
use ethereum_types::BigEndianHash;
use sha3::{Digest, Keccak256};
use web3;
use web3::contract::{Contract, Options};

use std::path::Path;
use web3::transports::Http;
use web3::types::{Address, H256};

pub struct UnsNamingService {
    eth_contract: Contract<Http>,
    polygon_contract: Contract<Http>,
}

impl UnsNamingService {
    async fn load_contract_addresses(
        eth_network_id: usize,
        matic_network_id: usize,
    ) -> Option<(String, String)> {
        let current_file_path = file!();
        let current_dir = Path::new(current_file_path).parent().unwrap();
        let config_file_path = current_dir.join("uns_configs.json");

        match configs::load_config_file(config_file_path).await {
            Ok(data) => {
                let eth_network = &data["networks"][eth_network_id.to_string()]["contracts"];
                let matic_network = &data["networks"][matic_network_id.to_string()]["contracts"];

                let eth_proxy_reader = eth_network["ProxyReader"]["address"].as_str()?.to_string();
                let matic_proxy_reader = matic_network["ProxyReader"]["address"]
                    .as_str()?
                    .to_string();

                Some((eth_proxy_reader, matic_proxy_reader))
            }
            Err(e) => {
                println!("Error: {}", e);
                None
            }
        }
    }

    pub async fn new(eth_rpc_url: String, polygon_rpc_url: String) -> Self {
        let contract_addresses = Self::load_contract_addresses(1, 137).await;
        let (eth_proxy_address, matic_proxy_address) = contract_addresses.unwrap();

        let eth_transport = web3::transports::Http::new(&eth_rpc_url).unwrap();
        let eth_provider = web3::Web3::new(eth_transport);

        let polygon_transport = web3::transports::Http::new(&polygon_rpc_url).unwrap();
        let polygon_provider = web3::Web3::new(polygon_transport);

        let eth_contract = Contract::from_json(
            eth_provider.eth(),
            eth_proxy_address.parse().unwrap(),
            include_bytes!("./uns_abis.json"),
        )
        .unwrap();

        let polygon_contract = Contract::from_json(
            polygon_provider.eth(),
            matic_proxy_address.parse().unwrap(),
            include_bytes!("./uns_abis.json"),
        )
        .unwrap();

        UnsNamingService {
            eth_contract,
            polygon_contract,
        }
    }

    pub fn uns_namehash(domain: &str) -> Option<H256> {
        let mut concatenated_hashes = [0; 64];

        for label in domain.split('.').rev() {
            let mut hasher = Keccak256::new();
            hasher.update(label.as_bytes());
            concatenated_hashes[32..].copy_from_slice(&hasher.finalize().as_slice());

            let mut hasher = Keccak256::new();
            hasher.update(&concatenated_hashes[..]);
            concatenated_hashes[..32].copy_from_slice(hasher.finalize().as_slice());
        }

        let mut res: [u8; 32] = [0; 32];
        res.copy_from_slice(&concatenated_hashes[..32]);
        Some(H256(res))
    }
}

#[async_trait::async_trait]
impl NamingServiceTrait for UnsNamingService {
    fn namehash(&self, domain: &str) -> Option<String> {
        let namehash = UnsNamingService::uns_namehash(domain).unwrap();

        Some(format!("0x{:x}", namehash))
    }

    async fn owner(&self, domain: &str) -> Option<String> {
        let namehash = UnsNamingService::uns_namehash(domain).unwrap();

        let eth_result: Result<Address, web3::contract::Error> = self
            .eth_contract
            .query(
                "ownerOf",
                (namehash.into_uint(),),
                None,
                Options::default(),
                None,
            )
            .await;

        if eth_result.is_ok() {
            let owner = eth_result.unwrap();

            if !owner.is_zero() {
                return Some(format!("0x{:#x}", owner));
            }
        }

        let matic_result: Result<Address, web3::contract::Error> = self
            .polygon_contract
            .query(
                "ownerOf",
                (namehash.into_uint(),),
                None,
                Options::default(),
                None,
            )
            .await;

        match matic_result {
            Ok(owner) => Some(format!("{:#x}", owner)),
            Err(e) => Some(Address::zero().to_string()),
        }
    }
}
