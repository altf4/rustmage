// The graveyard trigger pays one hybrid mana, then follows its exact source incarnation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: -1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        })),
        &[
            Instruction::Grow {
                subject: Subject::Targets,
                power: 1,
                toughness: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InZone {
            zone: TargetZone::Graveyard,
            event: &TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Creatures {
                    filter: CreatureFilter {
                        relation: 0,
                        other: false,
                        subtype: None,
                        keyword: Keywords::NONE,
                        attacking: false,
                        tokens: None,
                        artifact: false,
                    },
                    qualification: DamageQualification::Any,
                },
                combat_only: true,
                opponent_only: false,
                one_or_more: true,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayManaAlternatives {
                costs: &[
                    ManaCost {
                        colored: [1, 0, 0, 0, 0, 0],
                        generic: 0,
                    },
                    ManaCost {
                        colored: [0, 0, 1, 0, 0, 0],
                        generic: 0,
                    },
                ],
                skip: 1,
            },
            Instruction::ReturnSourceToHand(TargetZone::Graveyard),
        ],
    }],
    ..CardRules::EMPTY
};
