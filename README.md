# resolution-rust

Resolution-rust is a library for interacting with blockchain domain names. It can be used to retrieve domain's owner address and perform reverse resolution.

It calls the smart contracts on Ethereum and Polygon blockchains to retrieve the domain information.

resolution-go is primarily built and maintained by [Unstoppable Domains](https://unstoppabledomains.com/).

## Example

```rust
use uns::uns_provider::UnsProvider;

#[tokio::main]
async fn main() {
    let uns_provider = uns::uns_provider::UnsProvider::new(
        "https://mainnet.infura.io/v3/<infura_key>".to_owned(),
        "https://polygon-mainnet.infura.io/v3/<infura_key>".to_owned(),
    );

    let domain = "tun.x";
    let namehash = uns::uns_provider::UnsProvider::uns_namehash(domain).unwrap();

    print!("Namehash for {} is {}", domain, namehash);

    // Resolution
    let ownerAddress = uns_provider.owner(domain).await.unwrap();
    print!("Owner of {} is {#x}", domain, ownerAddress);

    // Reverse Resolution
    let address = "0x05391f2407b664fbd1dca5aea9eea89a29b946b4"
        .parse()
        .expect("Failed to parse address");

    let foundDomain = uns_provider.reverseOf(&address).await.unwrap();

    print!("Address of {#x} points to {}", address, foundDomain);
}

```

## Tests

1. Set env vars

```bash
export ETH_RPC_URL="https://polygon-mainnet.infura.io/v3/<your_key>"

export POLYGON_RPC_URL="https://polygon-mainnet.infura.io/v3/<your_key>"

```

2. Run test

```bash
cargo test -- --nocapture

```