// Authored from pinned Oracle and XMage sources; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::MENACE,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::MENACE)],
    }],
    extras: &CardExtras {
        cast_graveyard_minimum: 7,
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Hand,
        cost: ActivationCost {
            colored: [0, 0, 1, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: true,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::Grow {
            subject: Subject::Targets,
            power: -3,
            toughness: -1,
        }],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
