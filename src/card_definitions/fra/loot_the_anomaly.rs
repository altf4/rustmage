// Complete Oracle and release-note draft; pinned XMage implementation unavailable.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(-2, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0, first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::SourceNegativePowerCombatDamage],
    }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::GraveyardCardsAtLeast { kind: 0, minimum: 7 },
        origin: ActivationOrigin::Battlefield, cycling: false,
        cost: ActivationCost {
            colored: [0; 6], generic: 0, alternatives: &[],
            additional: Some(AdditionalCost { kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter { keywords: Keywords::NONE, source_only: false,
                    kinds: 1 | 64, nonland: false, other: true, tokens: None,
                    artifact_enchantment_or_token: false, subtype: None } }),
            mana_component: false, tap: false, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow { subject: Subject::Source, power: -2, toughness: 0 }],
        sorcery: false, once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
