const NAMED_LANDS: StatValue = StatValue::DistinctNames(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 2,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    keywords: Keywords::FLYING.union(Keywords::HEXPROOF),
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: NAMED_LANDS,
                multiplier: 1,
            },
        )],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 10,
            sacrifice: true,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Any),
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
        ]),
        program: &[
            Instruction::SelectGroup(0),
            Instruction::Damage(3),
            Instruction::SelectGroup(1),
            Instruction::Draw(PlayerSet::Targeted, 3),
            Instruction::GainLife(PlayerSet::Targeted, 3),
        ],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        exhaust: false,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
