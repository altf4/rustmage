const VILLAINS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some("Villain"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    stats: Stats {
        power: 2,
        toughness: 2,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 8),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::StaticModifier(0, VILLAINS)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::MaximumHandSize(8)],
        },
    ],
    extras: &CardExtras {
        static_modifiers: &[VILLAINS],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Threshold {
                value: StepValue::Hand,
                minimum: None,
                maximum: Some(7),
            },
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::DrawScaled(
            PlayerSet::Controller,
            AmountValue::ControllerHandDeficit(8),
        )],
    }],
    ..CardRules::EMPTY
};
