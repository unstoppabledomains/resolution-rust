pub fn normalize_domain(domain: &str) -> String {
    let normalized_domain = domain.to_lowercase().trim().to_string();

    if normalized_domain.is_empty() {
        panic!("Domain is empty");
    }

    normalized_domain
}
