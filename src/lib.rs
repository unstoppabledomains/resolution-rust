pub mod uns;

#[cfg(test)]
mod tests {
    use crate::uns;
    use dotenv::dotenv;
    use web3::types::H160;

    fn get_uns_provider() -> uns::uns_provider::UnsProvider {
        dotenv().ok();
        let eth_rpc_url = std::env::var("ETH_RPC_URL").unwrap();
        let polygon_rpc_url = std::env::var("POLYGON_RPC_URL").unwrap();

        uns::uns_provider::UnsProvider::new(eth_rpc_url, polygon_rpc_url)
    }

    #[test]
    fn compute_uns_namehash() {
        let domain = "tun.x";
        let hash = uns::uns_provider::UnsProvider::uns_namehash(domain).unwrap();

        assert_eq!(
            hash,
            "0xf69f403357efbbff39b5d523057f76f79285b5a9026ad746c5c6dda081444e4e"
                .parse()
                .unwrap()
        );
    }

    #[tokio::test]
    async fn get_owner_of_L2_domain() {
        let domain = "tun.x";
        let uns_provider = get_uns_provider();

        let owner = uns_provider.owner(domain).await.unwrap();

        let expected_address = "0x05391f2407b664fbd1dca5aea9eea89a29b946b4"
            .parse()
            .expect("Failed to parse address");

        assert_eq!(owner, expected_address);
    }

    #[tokio::test]
    async fn get_owner_of_L1_domain() {
        let domain = "brad.crypto";
        let uns_provider = get_uns_provider();

        let owner = uns_provider.owner(domain).await.unwrap();

        let expected_address = "0x8aad44321a86b170879d7a244c1e8d360c99dda8"
            .parse()
            .expect("Failed to parse address");

        assert_eq!(owner, expected_address);
    }

    #[tokio::test]
    async fn get_owner_of_unknown_domain() {
        let domain = "i-dont-know-who-are-you.x";
        let uns_provider = get_uns_provider();

        let owner = uns_provider.owner(domain).await.unwrap();

        assert_eq!(owner, H160::zero());
    }

    #[tokio::test]
    async fn get_reverse_of_L2_domain() {
        let address = "0x05391f2407b664fbd1dca5aea9eea89a29b946b4"
            .parse()
            .expect("Failed to parse address");
        let uns_provider = get_uns_provider();

        let domain = uns_provider.reverseOf(&address).await.unwrap();

        assert_eq!(domain, "tun.x");
    }

    #[tokio::test]
    async fn get_reverse_of_L1_domain() {
        let address = "0x0c57b28c9766932524a3566ca3cfe76ba73608ce"
            .parse()
            .expect("Failed to parse address");
        let uns_provider = get_uns_provider();

        let domain = uns_provider.reverseOf(&address).await.unwrap();

        assert_eq!(domain, "pepe.crypto");
    }

    #[tokio::test]
    async fn get_reverse_of_unknown_address() {
        let address = "0x0c57b28c9766932524a3566c0000000000000000"
            .parse()
            .expect("Failed to parse address");
        let uns_provider = get_uns_provider();

        let domain = uns_provider.reverseOf(&address).await.unwrap();

        assert_eq!(domain, "");
    }
}
