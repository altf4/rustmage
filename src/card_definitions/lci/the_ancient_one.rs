const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Source,
    attack: true,
    block: true,
    unless: Some(StepGuard::Expression {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Graveyard,
            relation: 0,
            kind: 1 | 2 | 16 | 32 | 64 | 256,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: true,
        }),
        minimum: Some(8),
        maximum: None,
    }),
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Rule,
    fragments: &[StaticAbilityFragment::CombatRestriction(0, RESTRICTION)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(8, 8),
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 1, 1, 0, 0, 0],
            mana_component: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::DiscardOneReflexiveManaValue(0),
        ],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: one(TargetRule::Player {
            opponent_only: false,
        }),
        program: &[Instruction::Mill(
            PlayerSet::Targeted,
            MillAmount::Amount(AmountValue::DiscardedCardManaValue),
        )],
    }],
    ..CardRules::EMPTY
};
