// Oracle and official release notes reviewed; no pinned XMage class exists.
const OFFICERS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Battlefield,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: Some("Officer"),
    basic: false,
    other: false,
    cards_only: false,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokensScaled {
            token: Card(6008),
            amount: AmountValue::Stat(StatValue::DilemmasFacedThisTurn),
            tapped: false,
        }],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            generic: 2,
            colored: [1, 1, 1, 1, 1, 0],
            mana_component: true,
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::ExileTopLibraryRemember {
                players: PlayerSet::Controller,
                amount: AmountValue::Fixed(5),
            },
            Instruction::CastRememberedExileFreeUpTo(OFFICERS),
        ],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        sorcery: false,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};
