const RAT_BONUS: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Rat"),
        basic: false,
        other: true,
        cards_only: false,
    }),
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Modify,
    fragments: &[StaticAbilityFragment::ScalingModifier(0, RAT_BONUS)],
}];

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::InterveningIf {
                guard: StepGuard::Expression {
                    value: StatValue::GraveyardCards,
                    minimum: Some(7),
                    maximum: None,
                },
                event: &TriggerCondition::ControlledSubtypeAttackDeclaration("Rat"),
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayCosts {
                costs: &[ResolutionCost {
                    mana: ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 2,
                    },
                    ..ResolutionCost::FREE
                }],
                skip: 1,
            },
            Instruction::ReturnSourceTappedAttacking,
        ],
    }],
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        scaling_modifiers: &[RAT_BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
