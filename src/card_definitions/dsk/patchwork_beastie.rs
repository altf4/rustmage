// Directly authored from the paired Oracle and pinned XMage sources.
const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Source,
    attack: true,
    block: true,
    unless: Some(StepGuard::Expression {
        value: StatValue::CONTROLLER_GRAVEYARD_CARD_TYPES,
        minimum: Some(4),
        maximum: None,
    }),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(0, RESTRICTION)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayMill(1),
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(1)),
        ],
    }],
    extras: &CardExtras {
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
