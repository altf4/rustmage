pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    keywords: Keywords::FLASH,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::NONE,
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
                kind: CounterKind::Impostor,
                amount: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::CreatureDiesWithCounter {
                relation: CastRelation::You,
                kind: CounterKind::Impostor,
            },
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::OtherThanEventObject {
                    base: &TargetRule::Object {
                        zone: TargetZone::Graveyard,
                        kinds: 1,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    excluded: None,
                },
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::ExileEventGraveyardCard,
                Instruction::ReturnToBattlefield,
            ],
        },
    ],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords::FLASH)],
    }],
    ..CardRules::EMPTY
};
