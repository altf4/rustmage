// Complete printed obligations from pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::OwnTurn,
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
                    kinds: 17,
                    nonland: false,
                    other: true,
                    tokens: None,
                    subtype: None,
                },
            }),
            mana_component: false,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: true,
        }],
        sorcery: false,
        once_per_turn: true,
    }],
    ..CardRules::EMPTY
};
