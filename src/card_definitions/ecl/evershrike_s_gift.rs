// Direct typed behavior; complete Oracle body and pinned XMage sources reviewed.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 1,
                toughness: 0,
            },
            keywords: Keywords::FLYING,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::AttachmentStats(Stats {
                power: 1,
                toughness: 0,
            }),
            StaticAbilityFragment::AttachmentKeywords(Keywords::FLYING),
        ],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 1,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::BlightCost(2),
                filter: SacrificeFilter {
                    kinds: 1,
                    keywords: Keywords::NONE,
                    source_only: false,
                    nonland: false,
                    other: false,
                    tokens: None,
                    artifact_enchantment_or_token: false,
                    subtype: None,
                },
            }),
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::ReturnSourceToHand(TargetZone::Graveyard)],
        sorcery: true,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
