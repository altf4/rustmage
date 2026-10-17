// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::GrowCreatures {
                filter: CreatureFilter {
                    relation: 1,
                    other: false,
                    subtype: None,
                    keyword: Keywords(0),
                    attacking: false,
                    tokens: None,
                    artifact: false,
                },
                power: -2,
                toughness: -2,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::Opponent,
                other: false,
                nontoken: false,
                subtype: None,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(8),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(8))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(8))],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
