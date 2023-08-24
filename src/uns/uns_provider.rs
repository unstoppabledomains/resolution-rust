use ethereum_types::BigEndianHash;
use sha3::{Digest, Keccak256};
use web3;
use web3::contract::{Contract, Options};

use web3::types::{Address, H256};

const ETH_MAINNET_PROXY_READER_ADDRESS: &str = "0x578853aa776Eef10CeE6c4dd2B5862bdcE767A8B";
const POLYGON_MAINNET_PROXY_READER_ADDRESS: &str = "0x91EDd8708062bd4233f4Dd0FCE15A7cb4d500091";

#[derive(Debug)]
pub struct UnsProvider {
    eth_contract: Contract<web3::transports::Http>,
    polygon_contract: Contract<web3::transports::Http>,
}

impl UnsProvider {
    pub fn new(eth_rpc_url: String, polygon_rpc_url: String) -> Self {
        let eth_transport = web3::transports::Http::new(&eth_rpc_url).unwrap();
        let eth_provider = web3::Web3::new(eth_transport);

        let polygon_transport = web3::transports::Http::new(&polygon_rpc_url).unwrap();
        let polygon_provider = web3::Web3::new(polygon_transport);

        let eth_contract = Contract::from_json(
            eth_provider.eth(),
            ETH_MAINNET_PROXY_READER_ADDRESS.parse().unwrap(),
            include_bytes!("./uns_abis.json"),
        )
        .unwrap();

        let polygon_contract = Contract::from_json(
            polygon_provider.eth(),
            POLYGON_MAINNET_PROXY_READER_ADDRESS.parse().unwrap(),
            include_bytes!("./uns_abis.json"),
        )
        .unwrap();

        UnsProvider {
            eth_contract,
            polygon_contract,
        }
    }

    pub fn uns_namehash(domain: &str) -> Option<H256> {
        let normalized_domain = domain.trim().to_lowercase();

        if normalized_domain.is_empty() {
            return None;
        }

        let mut concatenated_hashes = [0; 64];

        for label in normalized_domain.split('.').rev() {
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

    pub async fn owner(&self, domain: &str) -> Option<Address> {
        let namehash = UnsProvider::uns_namehash(domain).unwrap();

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
                return Some(owner);
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
            Ok(owner) => Some(owner),
            Err(e) => Some(Address::zero()),
        }
    }

    pub async fn reverseOf(&self, address: &Address) -> Option<String> {
        let eth_result: Result<String, web3::contract::Error> = self
            .eth_contract
            .query(
                "reverseNameOf",
                (address.to_owned(),),
                None,
                Options::default(),
                None,
            )
            .await;

        if eth_result.is_ok() {
            let domain = eth_result.unwrap();

            if !domain.is_empty() {
                return Some(domain);
            }
        }

        let matic_result: Result<String, web3::contract::Error> = self
            .polygon_contract
            .query(
                "reverseNameOf",
                (address.to_owned(),),
                None,
                Options::default(),
                None,
            )
            .await;

        match matic_result {
            Ok(domain) => Some(domain),
            Err(e) => Some("".to_owned()),
        }
    }
}
