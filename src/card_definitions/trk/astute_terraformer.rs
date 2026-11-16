// Pinned Oracle and complete XMage class: only the generic activation cost is reduced.
const LAND_NAMES: StatValue = StatValue::DistinctNames(CountFilter {
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
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        activation_generic_reductions: &[(
            0,
            ActivationGenericReduction::Stat {
                value: LAND_NAMES,
                multiplier: 1,
            },
        )],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 6,
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 2,
            toughness: 2,
        }],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: true,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
