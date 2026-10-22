//! Directly authored, compile-time card behavior. No rules text is interpreted here.
use super::InitialCounters as Counters;
use super::*;

pub(super) struct CardRules {
    pub deck_limit: Option<u8>,
    pub extras: &'static CardExtras,
    pub effect: Effect,
    pub triggers: &'static [TriggeredAbility],
    pub activated: &'static [ActivatedAbility],
    pub keywords: Keywords,
    pub enters: Counters,
    pub entry: EntryTapped,
    /// Fixed additional mana for each predeclared modal combination.
    pub mode_costs: Option<&'static [ManaCost]>,
    pub aura: Option<AuraSpec>,
    pub static_groups: &'static [StaticAbilityGroup],
}
impl CardRules {
    pub const EMPTY: Self = Self {
        deck_limit: Some(4),
        extras: &CardExtras::EMPTY,
        effect: Effect::Unsupported,
        triggers: &[],
        activated: &[],
        keywords: Keywords::NONE,
        enters: Counters {
            plus_one: 0,
            minus_one: 0,
            stun: 0,
            loyalty: 0,
            lore: 0,
        },
        entry: EntryTapped::Never,
        mode_costs: None,
        aura: None,
        static_groups: &[],
    };
    pub const fn checked_effect(&self) -> Effect {
        assert!(
            !matches!(self.effect, Effect::Unsupported),
            "complete authored card lacks behavior"
        );
        // A printed mana component opens the mana-ability payment window.
        // Reject zero-mana template flags copied into newly authored paid abilities.
        let mut index = 0;
        while index < self.activated.len() {
            let cost = &self.activated[index].cost;
            if !cost.mana_component {
                assert!(
                    cost.generic == 0,
                    "paid activation lacks mana payment window"
                );
                let mut color = 0;
                while color < 6 {
                    assert!(
                        cost.colored[color] == 0,
                        "paid activation lacks mana payment window"
                    );
                    color += 1;
                }
                let mut alternative = 0;
                while alternative < cost.alternatives.len() {
                    let mana = &cost.alternatives[alternative];
                    assert!(
                        mana.generic == 0,
                        "paid activation alternative lacks mana payment window"
                    );
                    let mut color = 0;
                    while color < 6 {
                        assert!(
                            mana.colored[color] == 0,
                            "paid activation alternative lacks mana payment window"
                        );
                        color += 1;
                    }
                    alternative += 1;
                }
            }
            index += 1;
        }
        if let Some(costs) = self.mode_costs {
            match self.effect {
                Effect::Modal(modes) => assert!(
                    costs.len() == modes.len(),
                    "modal additional-cost table mismatch"
                ),
                _ => panic!("modal costs require modal behavior"),
            }
        }
        if !self.extras.mode_optional_additional_paid.is_empty() {
            assert!(
                matches!(
                    self.extras.additional_cost,
                    Some(AdditionalCost {
                        kind: EffectChoiceKind::OptionalBlightCost(_),
                        ..
                    })
                ),
                "conditional modes require an optional additional cost"
            );
            match self.effect {
                Effect::Modal(modes) => assert!(
                    modes.len() == self.extras.mode_optional_additional_paid.len(),
                    "conditional modal intent table mismatch"
                ),
                _ => panic!("conditional modes require modal behavior"),
            }
        }
        self.effect
    }
}
const fn generic(amount: u32) -> ManaCost {
    ManaCost {
        colored: [0; 6],
        generic: amount,
    }
}
const fn one(rule: TargetRule) -> TargetPlan {
    TargetPlan::Single(TargetSpec::one(rule))
}
const fn object(kinds: u8) -> TargetRule {
    TargetRule::Object {
        zone: TargetZone::Battlefield,
        kinds,
        relation: -1,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    }
}
pub(super) struct CardExtras {
    pub transform: Option<TransformSpec>,
    /// Exact optional-additional-cost intent for each predeclared modal combination.
    pub mode_optional_additional_paid: &'static [bool],
    pub flash_if_control_subtype: Option<&'static str>,
    pub named_activation_lock: bool,
    pub zone_replacement: Option<ZoneReplacementRule>,
    pub opening_hand_battlefield: bool,
    pub activation_put_counters: &'static [(u16, CounterKind, u64)],
    pub fixed_entry_counters: &'static [(CounterKind, u64)],
    pub entry_choice: Option<EntryChoiceKind>,
    pub chosen_entry_counters: Option<CounterKind>,
    pub static_modifiers: &'static [StaticModifier],
    pub equipment: Option<EquipmentSpec>,
    pub aura: Option<AuraSpec>,
    pub static_ability_groups: &'static [StaticAbilityGroup],
    pub live_static_effects: &'static [LiveStaticEffect],
    pub tiered_costs: Option<&'static [ManaCost]>,
    pub base_stat_setter: Option<BaseStatSetter>,
    pub combat_restriction: Option<CombatRestriction>,
    pub role: Option<RoleKind>,
    pub plot_cost: Option<ManaCost>,
    pub adventure_identity: Option<&'static str>,
    pub adventure_spell: Option<Card>,
    pub adventure_parent: Option<Card>,
    pub omen_spell: Option<Card>,
    pub omen_parent: Option<Card>,
    pub warp_cost: Option<ManaCost>,
    pub vehicle_stats: Option<Stats>,
    pub replacement_effects: &'static [ReplacementSpec],
    pub attached_subtype: Option<&'static str>,
    pub attached_turn_keywords: Keywords,
    pub dynamic_stats: Option<DynamicStats>,
    pub scaling_modifiers: &'static [ScalingModifier],
    pub entry_trigger_suppression: u16,
    pub controlled_entry_untapped: u16,
    pub opponent_entry_tapped: u16,
    pub keyword_grants: &'static [KeywordGrant],
    pub additional_cost: Option<AdditionalCost>,
    pub graveyard_counter_cost: Option<u16>,
    pub collect_evidence: Option<(u16, u32)>,
    pub bargain_reduction: Option<u32>,
    pub x_symbols: u8,
    pub x_entry_counters: bool,
    pub kicker_cost: Option<ManaCost>,
    pub gift: Option<GiftSpec>,
    pub kicked_entry_counters: u16,
    pub counter_keyword_grants: &'static [(CounterKind, Keywords)],
    pub offspring_cost: Option<ManaCost>,
    pub offspring_copy: Option<Card>,
    pub flashback_cost: Option<ManaCost>,
    pub target_cost_reduction: Option<&'static TargetSpellReduction>,
    pub cast_timing_grants: &'static [CastFilter],
    pub cost_reductions: &'static [SpellReduction],
    pub evoke_costs: &'static [ManaCost],
    pub cost_alternatives: &'static [ManaCost],
    pub cost_alternative_guard: Option<StepGuard>,
    pub mana_alternative_replaces_additional: bool,
    pub kicked_targets: Option<TargetPlan>,
    pub cast_entry_counters: &'static [(CounterKind, u64)],
    pub hand_cast_entry_counters: &'static [(CounterKind, u64)],
    pub entry_counter_rules: &'static [EntryCounterRule],
    pub starting_loyalty: u64,
    pub exhaust_mask: u64,
    pub has_convoke: bool,
    pub static_cast_restriction: u8,
    pub uncounterable: bool,
    pub enchanted_modifiers: &'static [EnchantedModifier],
    pub flash_surcharge: Option<ManaCost>,
}
impl CardExtras {
    pub const EMPTY: Self = Self {
        transform: None,
        mode_optional_additional_paid: &[],
        flash_if_control_subtype: None,
        zone_replacement: None,
        opening_hand_battlefield: false,
        activation_put_counters: &[],
        fixed_entry_counters: &[],
        cast_entry_counters: &[],
        hand_cast_entry_counters: &[],
        named_activation_lock: false,
        entry_choice: None,
        chosen_entry_counters: None,
        static_modifiers: &[],
        equipment: None,
        aura: None,
        static_ability_groups: &[],
        live_static_effects: &[],
        tiered_costs: None,
        base_stat_setter: None,
        combat_restriction: None,
        role: None,
        plot_cost: None,
        adventure_identity: None,
        adventure_spell: None,
        adventure_parent: None,
        omen_spell: None,
        omen_parent: None,
        warp_cost: None,
        vehicle_stats: None,
        replacement_effects: &[],
        attached_subtype: None,
        attached_turn_keywords: Keywords::NONE,
        dynamic_stats: None,
        scaling_modifiers: &[],
        entry_trigger_suppression: 0,
        controlled_entry_untapped: 0,
        opponent_entry_tapped: 0,
        keyword_grants: &[],
        additional_cost: None,
        graveyard_counter_cost: None,
        collect_evidence: None,
        bargain_reduction: None,
        x_symbols: 0,
        x_entry_counters: false,
        kicker_cost: None,
        gift: None,
        kicked_entry_counters: 0,
        counter_keyword_grants: &[],
        offspring_cost: None,
        offspring_copy: None,
        flashback_cost: None,
        target_cost_reduction: None,
        cast_timing_grants: &[],
        cost_reductions: &[],
        evoke_costs: &[],
        cost_alternatives: &[],
        cost_alternative_guard: None,
        mana_alternative_replaces_additional: false,
        kicked_targets: None,
        entry_counter_rules: &[],
        starting_loyalty: 0,
        exhaust_mask: 0,
        has_convoke: false,
        static_cast_restriction: 0,
        uncounterable: false,
        enchanted_modifiers: &[],
        flash_surcharge: None,
    };
}

include!("../generated/authored_cards.rs");
