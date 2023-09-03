#[async_trait::async_trait]
pub trait NamingServiceTrait {
    fn namehash(&self, domain: &str) -> Option<String>;
    async fn owner(&self, domain: &str) -> Option<String>;
}
