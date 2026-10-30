// Directly authored from complete pinned Oracle and XMage sources; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 4,
            alternatives: &[],
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::ExileGraveyard,
                filter: SacrificeFilter {
                    keywords: Keywords::NONE,
                    source_only: false,
                    kinds: 1,
                    nonland: false,
                    other: true,
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
        program: &[Instruction::ReturnSourceToBattlefieldWithCounter {
            tapped: true,
            counter: CounterKind::PlusOne,
        }],
    }],
    ..CardRules::EMPTY
};
