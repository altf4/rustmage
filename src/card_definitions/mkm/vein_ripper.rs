// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 5),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Ward,
            targets: TargetPlan::Single(TargetSpec::NONE),
            program: &[
                Instruction::ChooseObjects {
                    kind: EffectChoiceKind::Sacrifice,
                    players: PlayerSet::Opponent,
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
                Instruction::Skip(1),
                Instruction::CounterRelated,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDies {
                relation: CastRelation::Any,
                other: false,
                nontoken: false,
                subtype: None,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            program: &[
                Instruction::LoseLife(PlayerSet::Targeted, 2),
                Instruction::GainLife(PlayerSet::Controller, 2),
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
