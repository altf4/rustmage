// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Any,
            ordinal: Some(2),
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Create two 1/1 white Human Soldier creature tokens.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::CreateTokens {
                    token: Card(5241),
                    amount: 2,
                    tapped: false,
                }],
            },
            SpellMode {
                text: "Put a +1/+1 counter on each creature you control.",
                targets: TargetPlan::Single(TargetSpec::NONE),
                program: &[Instruction::PutCreatureCounters {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    kind: CounterKind::PlusOne,
                    amount: 1,
                }],
            },
        ])],
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
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
