//! Directly authored, compile-time card behavior. No rules text is interpreted here.
use super::InitialCounters as Counters;
use super::*;

#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub struct CardRules {
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
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
        if let Some(copy) = self.extras.entry_copy_options {
            assert!(
                self.extras.entry_copy.is_some(),
                "copy options need an entry copy"
            );
            if let EntryCopySource::GraveyardCreature { exile_trigger } = copy.source {
                let slot = exile_trigger as usize;
                assert!(
                    slot < self.triggers.len(),
                    "graveyard copy needs a linked trigger slot"
                );
                assert!(
                    matches!(self.triggers[slot].condition, TriggerCondition::Reflexive),
                    "graveyard copy needs a reflexive trigger"
                );
            }
        }
        if let Some(EntryChoiceKind::NumberRange { minimum, maximum }) = self.extras.entry_choice {
            assert!(minimum <= maximum, "empty entry number range");
        }
        let mode_count = match self.extras.entry_choice {
            Some(EntryChoiceKind::NamedModes(names)) => {
                assert!(!names.is_empty(), "named entry modes cannot be empty");
                assert!(
                    names.len() <= u16::MAX as usize + 1,
                    "too many named entry modes"
                );
                let mut index = 0;
                while index < names.len() {
                    assert!(!names[index].is_empty(), "named entry mode needs a label");
                    index += 1;
                }
                names.len()
            }
            _ => 0,
        };
        let mut index = 0;
        while index < self.extras.entry_mode_triggers.len() {
            let (slot, mode) = self.extras.entry_mode_triggers[index];
            assert!(
                (slot as usize) < self.triggers.len(),
                "entry mode lacks trigger slot"
            );
            assert!((mode as usize) < mode_count, "invalid trigger entry mode");
            let mut previous = 0;
            while previous < index {
                assert!(
                    self.extras.entry_mode_triggers[previous].0 != slot,
                    "duplicate trigger entry mode"
                );
                previous += 1;
            }
            index += 1;
        }
        let mut group = 0;
        while group < self.static_groups.len() {
            let mut has_mode_gate = false;
            let mut fragment = 0;
            while fragment < self.static_groups[group].fragments.len() {
                if matches!(
                    self.static_groups[group].fragments[fragment],
                    StaticAbilityFragment::EntryMode(_)
                ) {
                    has_mode_gate = true;
                }
                fragment += 1;
            }
            let mut fragment = 0;
            while fragment < self.static_groups[group].fragments.len() {
                // Only these consumers currently consult group availability in
                // both real and speculative worlds. New families require their
                // own consumer integration and focused evidence before use.
                assert!(
                    !has_mode_gate
                        || matches!(
                            self.static_groups[group].fragments[fragment],
                            StaticAbilityFragment::EntryMode(_)
                                | StaticAbilityFragment::SourceEntryModeActivation { .. }
                                | StaticAbilityFragment::StaticModifier(..)
                                | StaticAbilityFragment::KeywordGrant(..)
                                | StaticAbilityFragment::ControlledPermanentKeywords { .. }
                                | StaticAbilityFragment::PlayLandFromGraveyard
                                | StaticAbilityFragment::AdditionalTriggers { .. }
                        ),
                    "entry mode fragment consumer is not integrated"
                );
                match self.static_groups[group].fragments[fragment] {
                    StaticAbilityFragment::EntryMode(mode) => {
                        assert!((mode as usize) < mode_count, "invalid static entry mode");
                    }
                    StaticAbilityFragment::SourceEntryModeActivation { mode, ability } => {
                        assert!(
                            (mode as usize) < mode_count,
                            "invalid activation entry mode"
                        );
                        assert!(
                            (ability as usize) < self.activated.len(),
                            "entry mode lacks activation slot"
                        );
                        assert!(
                            matches!(
                                self.activated[ability as usize].origin,
                                ActivationOrigin::Battlefield
                            ),
                            "mode activation needs battlefield source"
                        );
                    }
                    StaticAbilityFragment::SourceCelebrationCreatureForm { activation, .. } => {
                        assert!(
                            matches!(self.static_groups[group].first_layer, AbilityLayer::Type),
                            "Celebration subtype replacement begins in the type layer"
                        );
                        assert!(
                            (activation as usize) < self.activated.len()
                                && matches!(
                                    self.activated[activation as usize].origin,
                                    ActivationOrigin::Battlefield
                                ),
                            "Celebration grant needs a battlefield activation slot"
                        );
                    }
                    StaticAbilityFragment::ControlledArtifactFood(ability) => {
                        assert!(
                            matches!(self.static_groups[group].first_layer, AbilityLayer::Type),
                            "Food subtype grant begins in the type layer"
                        );
                        assert!(
                            (ability as usize) < self.activated.len()
                                && matches!(
                                    self.activated[ability as usize].origin,
                                    ActivationOrigin::Battlefield
                                ),
                            "Food ability needs a battlefield activation slot"
                        );
                    }
                    _ => {}
                }
                fragment += 1;
            }
            group += 1;
        }
        assert!(
            self.activated.len() <= activation_slots::FIRST_RESERVED as usize,
            "authored activation slots overlap reserved engine templates"
        );
        if let Some(false) = self.extras.has_rules_abilities {
            assert!(
                self.triggers.is_empty()
                    && self.activated.is_empty()
                    && self.static_groups.is_empty()
                    && self.extras.static_ability_groups.is_empty()
                    && self.extras.entry_copy.is_none()
                    && self.extras.entry_exile_counters.is_none(),
                "abilityless definition has rules abilities"
            );
            assert!(
                self.keywords.0 == 0 && self.aura.is_none() && self.extras.aura.is_none(),
                "abilityless definition has keywords or enchant ability"
            );
        }
        assert!(
            !matches!(self.effect, Effect::Unsupported),
            "complete authored card lacks behavior"
        );
        let mut power_up_index = 0;
        while power_up_index < self.extras.power_up_abilities.len() {
            let slot = self.extras.power_up_abilities[power_up_index] as usize;
            assert!(
                slot < self.activated.len(),
                "Power-up slot lacks an activated ability"
            );
            assert!(
                !self.activated[slot].exhaust,
                "Power-up is distinct from Exhaust"
            );
            power_up_index += 1;
        }
        let mut reduction_index = 0;
        while reduction_index < self.extras.activation_target_reductions.len() {
            let (slot, _) = self.extras.activation_target_reductions[reduction_index];
            assert!(
                (slot as usize) < self.activated.len(),
                "target reduction lacks activated slot"
            );
            match self.activated[slot as usize].targets {
                TargetPlan::Single(spec) => assert!(
                    spec.min == 1 && spec.max == 1,
                    "target activation reduction requires one mandatory target"
                ),
                _ => panic!("target activation reduction requires one target group"),
            }
            reduction_index += 1;
        }
        let mut counter_reduction_index = 0;
        while counter_reduction_index < self.extras.activation_target_counter_discounts.len() {
            let (slot, _) =
                self.extras.activation_target_counter_discounts[counter_reduction_index];
            assert!(
                (slot as usize) < self.activated.len(),
                "target counter reduction lacks activated slot"
            );
            match self.activated[slot as usize].targets {
                TargetPlan::Single(spec) => assert!(
                    spec.min == 1 && spec.max == 1,
                    "target counter reduction requires one target"
                ),
                _ => panic!("target counter reduction requires one target group"),
            }
            counter_reduction_index += 1;
        }
        let mut color_reduction_index = 0;
        while color_reduction_index < self.extras.activation_target_color_discounts.len() {
            let (slot, _) = self.extras.activation_target_color_discounts[color_reduction_index];
            assert!(
                (slot as usize) < self.activated.len(),
                "target color reduction lacks activated slot"
            );
            match self.activated[slot as usize].targets {
                TargetPlan::Single(spec) => assert!(
                    spec.min == 1 && spec.max == 1,
                    "target color reduction requires one mandatory target"
                ),
                _ => panic!("target color reduction requires one target group"),
            }
            color_reduction_index += 1;
        }
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
        self.effect
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub const fn generic(amount: u32) -> ManaCost {
    ManaCost {
        colored: [0; 6],
        generic: amount,
    }
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub const fn one(rule: TargetRule) -> TargetPlan {
    TargetPlan::Single(TargetSpec::one(rule))
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub const fn object(kinds: u8) -> TargetRule {
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
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
pub struct CardExtras {
    /// Explicit source-audited absence/presence of rules abilities, including outside the battlefield.
    pub has_rules_abilities: Option<bool>,
    pub transforming_double_face: bool,
    pub meld_face: bool,
    pub as_transform: Option<AsTransformEffect>,
    pub targets_up_to_x: bool,
    pub life_x_cost: bool,
    pub spell_cannot_be_copied: bool,
    /// Subtype whose presence among the caster's permanents is recorded after costs.
    pub cast_controlled_subtype: Option<&'static str>,
    /// Minimum cards in the caster's graveyard after this spell leaves its origin.
    pub cast_graveyard_minimum: u8,
    /// Cannot cast this spell during the caster's first three turns.
    pub cast_after_first_three_turns: bool,
    pub prepared_spell: Option<Card>,
    pub enters_prepared: bool,
    pub prepared_identity: Option<&'static str>,
    pub borrows_exiled_activations: bool,
    /// Immutable activated templates acquired only through an explicit grant.
    pub self_granted_activation_templates: &'static [u16],
    /// Attachment-granted templates that exile their exact grantor as a cost.
    pub attachment_activation_exile_costs: &'static [u16],
    pub named_activation_lock: bool,
    pub zone_replacement: Option<ZoneReplacementRule>,
    pub opening_hand_battlefield: bool,
    /// Source-counter discounts for the specified nonmana activated slot.
    /// Slots bearing the Power-up keyword, independently limited per exact object.
    pub power_up_abilities: &'static [u16],
    pub power_up_additional_uses: u32,
    pub power_up_other_creature_reduction: u32,
    /// Other equivalents of the printed mana cost, for hybrid reduction choices.
    /// These are not alternative casting costs.
    pub power_up_mana_cost_alternatives: &'static [ManaCost],
    pub activation_target_reductions: &'static [(u16, TargetActivationReduction)],
    /// Generic cost reduced by the chosen target's counters of this kind.
    pub activation_target_counter_discounts: &'static [(u16, CounterKind)],
    /// Generic reduction per distinct color of the chosen permanent target.
    pub activation_target_color_discounts: &'static [(u16, u32)],
    pub activation_generic_reductions: &'static [(u16, ActivationGenericReduction)],
    pub activation_counter_discounts: &'static [(u16, CounterKind)],
    /// Reduce generic activation cost once per controlled permanent of this subtype.
    pub activation_subtype_discounts: &'static [(u16, &'static str)],
    pub activation_other_subtype_discounts: &'static [(u16, &'static str)],
    pub activation_put_counters: &'static [(u16, CounterKind, u64)],
    pub fixed_entry_counters: &'static [(CounterKind, u64)],
    pub battle_defense: u64,
    pub as_turned_face_up_counters: &'static [(CounterKind, u64)],
    pub entry_choice: Option<EntryChoiceKind>,
    /// (trigger slot, named-mode index), checked only at the triggering event.
    pub entry_mode_triggers: &'static [(u16, u16)],
    pub entry_copy: Option<EntryCopySpec>,
    pub entry_copy_options: Option<&'static EntryCopyOptions>,
    /// Copiable keyword exceptions added by this as-entry copy replacement.
    pub entry_copy_keywords: Keywords,
    pub entry_exile_counters: Option<EntryExileCountersSpec>,
    pub devour: Option<DevourSpec>,
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
    pub warp: Option<&'static WarpSpec>,
    pub disguise_cost: Option<ManaCost>,
    pub sneak_cost: Option<ManaCost>,
    pub sneak_costs: &'static [ManaCost],
    pub web_slinging_cost: Option<ManaCost>,
    /// Graveyard alternative with an optional power-based creature tap.
    pub harmonize_cost: Option<ManaCost>,
    /// Variable symbols in the printed harmonize alternative (independent of the mana cost).
    pub harmonize_x_symbols: u8,
    pub mayhem_cost: Option<ManaCost>,
    pub mayhem_cost_alternatives: &'static [ManaCost],
    pub mayhem_land: bool,
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
    pub x_complement_entry_counters: Option<(CounterKind, u32)>,
    pub teamwork: Option<TeamworkSpec>,
    pub blight: Option<BlightSpec>,
    pub kicker_cost: Option<ManaCost>,
    pub kicker_additional_cost: Option<AdditionalCost>,
    pub gift: Option<GiftSpec>,
    pub kicked_entry_counters: u16,
    pub counter_keyword_grants: &'static [(CounterKind, Keywords)],
    pub offspring_cost: Option<ManaCost>,
    pub offspring_copy: Option<Card>,
    pub flashback_cost: Option<ManaCost>,
    pub flashback_cost_alternatives: &'static [ManaCost],
    pub flashback_additional: Option<AdditionalCost>,
    pub target_cost_reduction: Option<&'static TargetSpellReduction>,
    pub target_cost_increase: Option<&'static TargetSpellIncrease>,
    pub behold_cast: Option<BeholdCastSpec>,
    pub cast_timing_grants: &'static [CastFilter],
    pub cast_timing_requires_opponent_spell: bool,
    pub self_cast_timing: Option<SelfCastTiming>,
    pub cast_timing_guard: StepGuard,
    pub cost_reductions: &'static [SpellReduction],
    pub cost_alternatives: &'static [ManaCost],
    /// Printed evoke routes, also present among the ordinary alternative costs.
    pub evoke_costs: &'static [ManaCost],
    pub cost_alternative_guard: Option<StepGuard>,
    pub mana_alternative_replaces_additional: bool,
    /// Independent additional mana route crossed with every printed hybrid base.
    pub additional_mana_alternative: Option<ManaCost>,
    /// Legacy reminder-text activation replaced by rules-derived basic-land mana.
    pub legacy_subtype_mana_activation: Option<u16>,
    pub kicked_targets: Option<TargetPlan>,
    /// The spell's complete target plan when its optional Bargain cost was paid.
    pub bargain_targets: Option<TargetPlan>,
    pub cast_entry_counters: &'static [(CounterKind, u64)],
    pub graveyard_cast_entry_counters: &'static [(CounterKind, u64)],
    pub hand_cast_entry_counters: &'static [(CounterKind, u64)],
    pub entry_counter_rules: &'static [EntryCounterRule],
    pub starting_loyalty: u64,
    pub exhaust_mask: u64,
    pub has_convoke: bool,
    pub has_improvise: bool,
    pub static_cast_restriction: u8,
    pub uncounterable: bool,
    /// This Aura may enchant an artifact or a non-Aura enchantment.
    pub aura_artifact_or_non_aura_enchantment: bool,
    /// Additional live subtype accepted by this Aura’s ordinary kind union.
    pub aura_kind_or_subtype: Option<&'static str>,
    pub enchant_basic: bool,
    pub enchanted_modifiers: &'static [EnchantedModifier],
    pub flash_surcharge: Option<ManaCost>,
}
#[cfg_attr(feature = "test-support", rustmage_test_access::fixture)]
impl CardExtras {
    pub const EMPTY: Self = Self {
        has_rules_abilities: None,
        transforming_double_face: false,
        meld_face: false,
        as_transform: None,
        targets_up_to_x: false,
        life_x_cost: false,
        spell_cannot_be_copied: false,
        cast_controlled_subtype: None,
        cast_graveyard_minimum: 0,
        cast_after_first_three_turns: false,
        prepared_spell: None,
        enters_prepared: false,
        prepared_identity: None,
        borrows_exiled_activations: false,
        self_granted_activation_templates: &[],
        attachment_activation_exile_costs: &[],
        zone_replacement: None,
        opening_hand_battlefield: false,
        power_up_abilities: &[],
        power_up_additional_uses: 0,
        power_up_other_creature_reduction: 0,
        power_up_mana_cost_alternatives: &[],
        activation_target_reductions: &[],
        activation_target_counter_discounts: &[],
        activation_target_color_discounts: &[],
        activation_generic_reductions: &[],
        activation_counter_discounts: &[],
        activation_subtype_discounts: &[],
        activation_other_subtype_discounts: &[],
        activation_put_counters: &[],
        fixed_entry_counters: &[],
        battle_defense: 0,
        as_turned_face_up_counters: &[],
        cast_entry_counters: &[],
        graveyard_cast_entry_counters: &[],
        hand_cast_entry_counters: &[],
        named_activation_lock: false,
        entry_choice: None,
        entry_mode_triggers: &[],
        entry_copy: None,
        entry_copy_options: None,
        entry_copy_keywords: Keywords::NONE,
        entry_exile_counters: None,
        devour: None,
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
        warp: None,
        disguise_cost: None,
        sneak_cost: None,
        sneak_costs: &[],
        web_slinging_cost: None,
        harmonize_cost: None,
        harmonize_x_symbols: 0,
        mayhem_cost: None,
        mayhem_cost_alternatives: &[],
        mayhem_land: false,
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
        x_complement_entry_counters: None,
        teamwork: None,
        blight: None,
        kicker_cost: None,
        kicker_additional_cost: None,
        gift: None,
        kicked_entry_counters: 0,
        counter_keyword_grants: &[],
        offspring_cost: None,
        offspring_copy: None,
        flashback_cost: None,
        flashback_cost_alternatives: &[],
        flashback_additional: None,
        target_cost_reduction: None,
        target_cost_increase: None,
        behold_cast: None,
        cast_timing_grants: &[],
        cast_timing_requires_opponent_spell: false,
        self_cast_timing: None,
        cast_timing_guard: StepGuard::Always,
        cost_reductions: &[],
        cost_alternatives: &[],
        evoke_costs: &[],
        cost_alternative_guard: None,
        mana_alternative_replaces_additional: false,
        additional_mana_alternative: None,
        legacy_subtype_mana_activation: None,
        kicked_targets: None,
        bargain_targets: None,
        entry_counter_rules: &[],
        starting_loyalty: 0,
        exhaust_mask: 0,
        has_convoke: false,
        has_improvise: false,
        static_cast_restriction: 0,
        uncounterable: false,
        aura_artifact_or_non_aura_enchantment: false,
        aura_kind_or_subtype: None,
        enchant_basic: false,
        enchanted_modifiers: &[],
        flash_surcharge: None,
    };
}

pub fn rules(card: Card) -> Option<&'static CardRules> {
    #[cfg(feature = "test-support")]
    if let Some(rules) = super::fixture_rules::rules(card) {
        return Some(rules);
    }
    super::generated::catalogue()
        .rules
        .get(card.index())
        .copied()
}
pub fn mode_costs(card: Card) -> Option<&'static [ManaCost]> {
    rules(card).and_then(|r| r.mode_costs)
}
