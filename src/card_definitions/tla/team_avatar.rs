// Direct typed behavior from the pinned Oracle and XMage sources.
const CREATURE_COUNT: AmountValue = AmountValue::Stat(StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 1,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: false,
}));
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ControlledCreatureAttacksAlone,
        targets: TargetPlan::NONE,
        program: &[Instruction::GrowScaled {
            subject: Subject::Entering,
            amount: CREATURE_COUNT,
            power: 1,
            toughness: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            colored: [1, 0, 0, 0, 0, 0],
            generic: 2,
            mana_component: true,
            discard: true,
            ..ActivationCost::ZERO
        },
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Hand,
        cycling: false,
        exhaust: false,
        sorcery: false,
        once_per_turn: false,
        targets: one(TargetRule::Creature),
        program: &[Instruction::DamageScaled(CREATURE_COUNT)],
    }],
    ..CardRules::EMPTY
};
