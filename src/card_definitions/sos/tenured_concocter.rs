// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTargeted {
            relation: CastRelation::Opponent,
            spells_only: false,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::May { skip: 1 },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords::VIGILANCE,
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                Keywords::VIGILANCE,
            )],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ScalingModifier(
                0,
                ScalingModifier {
                    subject: Subject::Source,
                    value: StatValue::LifeGainOccurredThisTurn,
                    stats: Stats {
                        power: 2,
                        toughness: 0,
                    },
                },
            )],
        },
    ],
    extras: &CardExtras {
        static_ability_groups: &[
            StaticAbilityGroup {
                line: 0,
                first_layer: AbilityLayer::Ability,
                fragments: &[StaticAbilityFragment::IntrinsicKeywords(
                    Keywords::VIGILANCE,
                )],
            },
            StaticAbilityGroup {
                line: 2,
                first_layer: AbilityLayer::Modify,
                fragments: &[StaticAbilityFragment::ScalingModifier(
                    0,
                    ScalingModifier {
                        subject: Subject::Source,
                        value: StatValue::LifeGainOccurredThisTurn,
                        stats: Stats {
                            power: 2,
                            toughness: 0,
                        },
                    },
                )],
            },
        ],
        scaling_modifiers: &[ScalingModifier {
            subject: Subject::Source,
            value: StatValue::LifeGainOccurredThisTurn,
            stats: Stats {
                power: 2,
                toughness: 0,
            },
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
