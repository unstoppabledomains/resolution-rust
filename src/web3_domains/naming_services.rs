use crate::web3_domains::ens_naming_service::ens_naming_service::EnsNamingService;
use crate::web3_domains::naming_service_traits::NamingServiceTrait;
use crate::web3_domains::uns_naming_service::uns_naming_service::UnsNamingService;

pub enum NamingService {
    UNS(UnsNamingService),
    ENS(EnsNamingService),
}

pub struct Web3Domain {
    uns: UnsNamingService,
    ens: EnsNamingService,
}

pub struct Web3DomainConfig {
    pub eth_rpc_url: String,
    pub polygon_rpc_url: String,
}

impl Web3Domain {
    fn split_domain(domain: &str) -> Option<(String, String)> {
        let mut parts = domain.split('.').collect::<Vec<&str>>();

        let tld = parts.pop()?.to_string();

        let label = parts.join(".");

        Some((label, tld))
    }

    fn get_naming_service(self, tld: String) -> Box<dyn NamingServiceTrait> {
        match tld.as_str() {
            "crypto" | "x" => Box::new(self.uns),
            "eth" | "kred" | "luxe" | "xyz" => Box::new(self.ens),
            _ => panic!("Unsupported TLD"),
        }
    }

    pub async fn new(config: Web3DomainConfig) -> Self {
        Web3Domain {
            uns: UnsNamingService::new(config.eth_rpc_url.clone(), config.polygon_rpc_url.clone()),
            ens: EnsNamingService::new(config.eth_rpc_url.clone()).await,
        }
    }

    pub fn namehash(self, domain: &str) -> Option<String> {
        let (label, tld) = Self::split_domain(domain)?;

        let service = self.get_naming_service(tld);

        service.namehash(domain)
    }

    pub async fn owner_of(self, domain: &str) -> Option<String> {
        let (label, tld) = Self::split_domain(domain)?;

        let service = self.get_naming_service(tld);

        service.owner(domain).await
    }
}
