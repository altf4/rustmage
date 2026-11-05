// Directly authored from the complete pinned Oracle and XMage Renew implementation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [0, 0, 0, 0, 1, 0],
            generic: 4,
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        origin: ActivationOrigin::Graveyard,
        guard: ActivationGuard::Always,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
        program: &[Instruction::PutCountersMixed {
            subject: Subject::Targets,
            amounts: &[(CounterKind::PlusOne, 2), (CounterKind::Reach, 1)],
        }],
        sorcery: true,
        exhaust: false,
        cycling: false,
        once_per_turn: false,
    }],
    keywords: Keywords::REACH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::REACH)],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
