// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Object {
                    zone: TargetZone::Graveyard,
                    kinds: 0,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                min: 0,
                max: 1,
            }),
            program: &[
                Instruction::ExileGraveTargets,
                Instruction::IfExiledCreature { skip: 1 },
                Instruction::CreateTokens {
                    token: Card(5296),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::CardDrawn {
                relation: CastRelation::You,
                ordinal: Some(2),
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::LoseLife(PlayerSet::Opponent, 1),
                Instruction::GainLife(PlayerSet::Controller, 1),
            ],
        },
    ],
    ..CardRules::EMPTY
};
