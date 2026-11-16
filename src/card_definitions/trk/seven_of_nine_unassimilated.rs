// Complete Oracle behavior, checked against pinned XMage.
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
    effect: Effect::Creature(0, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(LAND_NAMES),
                minimum: 5,
                skip: 1,
            },
            Instruction::Skip(1),
            Instruction::Discard(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 6,
            colored: [0, 1, 0, 0, 0, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::SetBaseStatsPermanent {
            subject: Subject::Source,
            power: Some(7),
            toughness: Some(9),
        }],
        guard: ActivationGuard::StatAtLeast {
            value: LAND_NAMES,
            minimum: 7,
        },
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
