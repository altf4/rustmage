// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[Instruction::CreateTokens {
                token: Card(5246),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Controller,
                    filter: SacrificeFilter {
                        artifact_enchantment_or_token: false,
                        keywords: Keywords::NONE,
                        source_only: false,
                        kinds: 16,
                        nonland: false,
                        other: false,
                        tokens: None,
                        subtype: Some("Food"),
                    },
                    amount: 1,
                    optional_skip: Some(2),
                },
                Instruction::GrowCreatures {
                    filter: CreatureFilter {
                        relation: -1,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: true,
                        tokens: None,
                        artifact: false,
                    },
                    power: 1,
                    toughness: 1,
                },
                Instruction::GrantCreatures {
                    filter: CreatureFilter {
                        relation: -1,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: true,
                        tokens: None,
                        artifact: false,
                    },
                    keywords: Keywords::TRAMPLE,
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
    keywords: Keywords::NONE,
    ..CardRules::EMPTY
};
