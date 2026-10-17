//! Compiled typed catalogue inspection, used for the one-time compiler removal proof.
use super::*;
#[test]
fn compiled_catalogue_snapshot() {
    let mut rows = Vec::new();
    for card in Card::catalogue() {
        let mut values = std::collections::BTreeMap::new();
        values.insert(
            "definition",
            format!("{:?}", generated::DEFINITIONS[card.index()]),
        );
        values.insert(
            "static_modifiers",
            format!("{:?}", generated::static_modifiers(card)),
        );
        values.insert("equipment", format!("{:?}", generated::equipment(card)));
        values.insert("aura", format!("{:?}", generated::aura(card)));
        values.insert(
            "static_ability_groups",
            format!("{:?}", generated::static_ability_groups(card)),
        );
        values.insert(
            "live_static_effects",
            format!("{:?}", generated::live_static_effects(card)),
        );
        values.insert(
            "tiered_costs",
            format!("{:?}", generated::tiered_costs(card)),
        );
        values.insert(
            "base_stat_setter",
            format!("{:?}", generated::base_stat_setter(card)),
        );
        values.insert(
            "combat_restriction",
            format!("{:?}", generated::combat_restriction(card)),
        );
        values.insert("role", format!("{:?}", generated::role(card)));
        values.insert("plot_cost", format!("{:?}", generated::plot_cost(card)));
        values.insert(
            "adventure_identity",
            format!("{:?}", generated::adventure_identity(card)),
        );
        values.insert(
            "adventure_spell",
            format!("{:?}", generated::adventure_spell(card)),
        );
        values.insert(
            "adventure_parent",
            format!("{:?}", generated::adventure_parent(card)),
        );
        values.insert("omen_spell", format!("{:?}", generated::omen_spell(card)));
        values.insert("omen_parent", format!("{:?}", generated::omen_parent(card)));
        values.insert("warp_cost", format!("{:?}", generated::warp_cost(card)));
        values.insert(
            "vehicle_stats",
            format!("{:?}", generated::vehicle_stats(card)),
        );
        values.insert(
            "replacement_effects",
            format!("{:?}", generated::replacement_effects(card)),
        );
        values.insert(
            "attached_subtype",
            format!("{:?}", generated::attached_subtype(card)),
        );
        values.insert(
            "attached_turn_keywords",
            format!("{:?}", generated::attached_turn_keywords(card)),
        );
        values.insert(
            "dynamic_stats",
            format!("{:?}", generated::dynamic_stats(card)),
        );
        values.insert(
            "scaling_modifiers",
            format!("{:?}", generated::scaling_modifiers(card)),
        );
        values.insert(
            "entry_trigger_suppression",
            format!("{:?}", generated::entry_trigger_suppression(card)),
        );
        values.insert(
            "controlled_entry_untapped",
            format!("{:?}", generated::controlled_entry_untapped(card)),
        );
        values.insert(
            "opponent_entry_tapped",
            format!("{:?}", generated::opponent_entry_tapped(card)),
        );
        values.insert(
            "keyword_grants",
            format!("{:?}", generated::keyword_grants(card)),
        );
        values.insert(
            "additional_cost",
            format!("{:?}", generated::additional_cost(card)),
        );
        values.insert(
            "collect_evidence",
            format!("{:?}", generated::collect_evidence(card)),
        );
        values.insert(
            "bargain_reduction",
            format!("{:?}", generated::bargain_reduction(card)),
        );
        values.insert("x_symbols", format!("{:?}", generated::x_symbols(card)));
        values.insert(
            "x_entry_counters",
            format!("{:?}", generated::x_entry_counters(card)),
        );
        values.insert("kicker_cost", format!("{:?}", generated::kicker_cost(card)));
        values.insert("gift", format!("{:?}", generated::gift(card)));
        values.insert(
            "kicked_entry_counters",
            format!("{:?}", generated::kicked_entry_counters(card)),
        );
        values.insert(
            "counter_keyword_grants",
            format!("{:?}", generated::counter_keyword_grants(card)),
        );
        values.insert(
            "offspring_cost",
            format!("{:?}", generated::offspring_cost(card)),
        );
        values.insert(
            "offspring_copy",
            format!("{:?}", generated::offspring_copy(card)),
        );
        values.insert(
            "flashback_cost",
            format!("{:?}", generated::flashback_cost(card)),
        );
        values.insert(
            "target_cost_reduction",
            format!("{:?}", generated::target_cost_reduction(card)),
        );
        values.insert(
            "cast_timing_grants",
            format!("{:?}", generated::cast_timing_grants(card)),
        );
        values.insert(
            "cost_reductions",
            format!("{:?}", generated::cost_reductions(card)),
        );
        values.insert(
            "cost_alternatives",
            format!("{:?}", generated::cost_alternatives(card)),
        );
        values.insert(
            "entry_counter_rules",
            format!("{:?}", generated::entry_counter_rules(card)),
        );
        values.insert(
            "starting_loyalty",
            format!("{:?}", generated::starting_loyalty(card)),
        );
        values.insert(
            "exhaust_mask",
            format!("{:?}", generated::exhaust_mask(card)),
        );
        values.insert("has_convoke", format!("{:?}", generated::has_convoke(card)));
        values.insert(
            "static_cast_restriction",
            format!("{:?}", generated::static_cast_restriction(card)),
        );
        values.insert(
            "uncounterable",
            format!("{:?}", generated::uncounterable(card)),
        );
        values.insert(
            "enchanted_modifiers",
            format!("{:?}", generated::enchanted_modifiers(card)),
        );
        values.insert(
            "flash_surcharge",
            format!("{:?}", generated::flash_surcharge(card)),
        );
        values.insert("effective_aura", format!("{:?}", card.aura()));
        values.insert(
            "effective_static_groups",
            format!("{:?}", card.static_ability_groups()),
        );
        values.insert("effective_mode_costs", format!("{:?}", card.tiered_costs()));
        rows.push(values);
    }
    let snapshot = serde_json::json!({
        "rows": rows,
        "colors": format!("{:?}", generated::COLORS),
        "aliases": format!("{:?}", generated::NAME_INDEX),
        "all": format!("{:?}", Card::ALL),
        "tokens": format!("{:?}", generated::TOKENS),
        "emblems": format!("{:?}", generated::EMBLEMS),
        "token_base": generated::TOKEN_BASE,
        "emblem_base": generated::EMBLEM_BASE,
        "creature_types": CREATURE_TYPES,
    });
    assert_eq!(rows.len(), Card::catalogue_len());
    if let Some(path) = std::env::var_os("RUSTMAGE_CATALOGUE_SNAPSHOT") {
        std::fs::write(path, serde_json::to_vec(&snapshot).unwrap()).unwrap();
    }
}
