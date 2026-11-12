const LINKED_THRESHOLD: StatValue = StatValue::AtLeast {
    value: &StatValue::LinkedExileCardCount,
    minimum: 3,
};
const BOOST: ScalingModifier = ScalingModifier {
    subject: Subject::Source,
    value: LINKED_THRESHOLD,
    stats: Stats {
        power: 3,
        toughness: 3,
    },
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SURVIVAL,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 0,
                relation: -1,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[Instruction::ExileTargetsLinkedToSource],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::SourceConditionalKeywords {
                condition: LINKED_THRESHOLD,
                keywords: Keywords::HEXPROOF,
            },
            StaticAbilityFragment::ScalingModifier(0, BOOST),
        ],
    }],
    extras: &CardExtras {
        scaling_modifiers: &[BOOST],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
