// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CounterUnlessPay {
                cost: ManaCost {
                    colored: [0, 0, 0, 0, 0, 0],
                    generic: 2,
                },
                subject: CounterSubject::Related,
                exile: false,
                unpaid_skip: 0,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Targeted {
                    filter: &CastFilter::InstantSorcery,
                    target: CastTarget::Creature(CastRelation::Any),
                },
                ordinal: None,
            },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::Grow {
                    subject: Subject::Source,
                    power: 1,
                    toughness: 0,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Source,
                    keywords: Keywords(64),
                },
            ],
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
    keywords: Keywords(0),
    ..CardRules::EMPTY
};
