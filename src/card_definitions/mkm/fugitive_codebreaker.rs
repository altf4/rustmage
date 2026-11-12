pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::HASTE,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::Grow {
                subject: Subject::Source,
                power: 1,
                toughness: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfTurnedFaceUp,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::DiscardHand(PlayerSet::Controller),
                Instruction::Draw(PlayerSet::Controller, 3),
            ],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::HASTE)],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec {
            mana: ManaCost {
                colored: [0, 0, 0, 1, 0, 0],
                generic: 5,
            },
            alternatives: &[],
            modifier: DisguiseCostModifier::GenericReduction(StatValue::Count(CountFilter {
                zone: CountZone::Graveyard,
                relation: 0,
                kind: 4 | 8,
                exclude: 0,
                subtype: None,
                basic: false,
                other: false,
                cards_only: true,
            })),
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
