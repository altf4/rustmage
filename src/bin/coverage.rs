use rustmage::Card;

fn main() {
    let standard: Vec<_> = Card::catalogue()
        .filter(|card| card.is_standard_legal())
        .collect();
    let implemented: Vec<_> = standard
        .iter()
        .filter(|card| card.is_supported())
        .map(|card| card.name())
        .collect();
    let unsupported = standard.len() - implemented.len();
    println!(
        "{}",
        serde_json::json!({
            "catalogue": rustmage::CATALOGUE_VERSION,
            "standard_legal_cards": standard.len(),
            "implemented_standard_cards": implemented.len(),
            "unsupported_standard_cards": unsupported,
            "implemented_names": implemented,
            "complete": unsupported == 0,
        })
    );
}
