// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::ManaSpent {
            minimum: 0,
            increment: false,
            event: &TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::InstantSorcery,
                ordinal: None,
            },
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[
            Instruction::IfManaSpent {
                minimum: None,
                maximum: Some(4),
                skip: 1,
            },
            Instruction::Grow {
                subject: Subject::Source,
                power: 3,
                toughness: 0,
            },
            Instruction::IfManaSpent {
                minimum: Some(5),
                maximum: None,
                skip: 1,
            },
            Instruction::PutCounters {
                subject: Subject::Source,
                kind: CounterKind::PlusOne,
                amount: 3,
            },
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
    keywords: Keywords(1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
