// Complete printed obligations from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::SourceConditionalKeywords {
            condition: StatValue::Count(CountFilter {
                zone: CountZone::Battlefield,
                relation: 0,
                kind: 0,
                exclude: 0,
                subtype: Some("Goblin"),
                basic: false,
                other: true,
                cards_only: false,
            }),
            keywords: Keywords::HASTE,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            colored: [0; 6],
            generic: 0,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::Sacrifice,
                filter: SacrificeFilter {
                    artifact_enchantment_or_token: false,
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 0,
                    nonland: false,
                    other: true,
                    tokens: None,
                    subtype: Some("Goblin"),
                },
            }),
            mana_component: false,
            tap: true,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 1, 1, 0, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
