// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5271),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords(0),
                        source_only: false,
                        kinds: 1,
                        nonland: false,
                        other: false,
                        tokens: None,
                        subtype: None,
                    },
                    amount: 1,
                    optional_skip: Some(1),
                },
                Instruction::Search(SearchSpec {
                    filter: SearchFilter::AnyOf(&[SearchFilter::Creature, SearchFilter::BasicLand]),
                    maximum: 1,
                    destination: SearchDestination::Hand,
                    tapped: false,
                    reveal: true,
                }),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: 2,
            }),
            program: &[
                Instruction::Grow {
                    subject: Subject::Targets,
                    power: 2,
                    toughness: 2,
                },
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords(256),
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
