// Direct typed behavior from the complete pinned Oracle and XMage sources.
const BOOST: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: StatValue::AtLeast {
        value: &StatValue::ControllerSpeed,
        minimum: 4,
    },
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::StartYourEngines],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(0, BOOST)],
        },
    ],
    extras: &CardExtras {
        scaling_modifiers: &[BOOST],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5487),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
