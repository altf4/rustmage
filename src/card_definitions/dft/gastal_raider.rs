// Complete direct typed behavior from full pinned Oracle and XMage sources.
const MAX_SPEED: StatValue = StatValue::AtLeast {
    value: &StatValue::ControllerSpeed,
    minimum: 4,
};
const BOOST: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: MAX_SPEED,
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::StartYourEngines],
    },
    StaticAbilityGroup {
        line: 2,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: MAX_SPEED,
                keywords: Keywords::MENACE,
            },
            StaticAbilityFragment::ScalingModifier(0, BOOST),
        ],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::NONE,
    static_groups: GROUPS,
    activated: &[],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[Instruction::RevealTargetHandChoose {
            kinds: 4 | 8,
            exile: false,
        }],
    }],

    extras: &CardExtras {
        static_ability_groups: GROUPS,
        scaling_modifiers: &[BOOST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
