// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 0,
                    relation: 1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::OwnedByController,
            }),
            program: &[Instruction::GainControl {
                subject: Subject::Targets,
                controller: PlayerSet::Controller,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfTurnedFaceUp,
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: true,
                }),
                TargetSpec {
                    rule: TargetRule::Object {
                        zone: TargetZone::Battlefield,
                        kinds: 0,
                        relation: 0,
                        other: false,
                        nonland: false,
                        tapped: false,
                        min_mana: 0,
                        max_mana: u32::MAX,
                    },
                    min: 0,
                    max: usize::MAX,
                },
            ]),
            program: &[
                Instruction::GainControlRememberCount {
                    subject: Subject::Targets,
                    controller: PlayerSet::Targeted,
                },
                Instruction::DrawScaled(PlayerSet::Controller, AmountValue::MovedCount),
            ],
        },
    ],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(ManaCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 1,
        })),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
