// Direct typed behavior from the paired Oracle and pinned XMage sources.
const CONTROLLED: BattlefieldFilter = BattlefieldFilter {
    kinds: 0,
    relation: 0,
    nonland: false,
    tapped: None,
    without: Keywords::NONE,
    subtype: None,
    max_mana: u32::MAX,
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::ControlledSubtypes(&["Scout", "Pirate", "Rogue"]),
                combat_only: true,
                opponent_only: false,
                one_or_more: true,
            },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::ExileTopMayCastPaidFrom(PlayerSet::DamagedPlayer),
                Instruction::IfOfferedCastCommitted { skip: 1 },
                Instruction::CreateTokens {
                    token: Card(5302),
                    amount: 1,
                    tapped: false,
                },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::NotOwnedByYou,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::PutPermanentGroupCounters {
                filters: &[
                    BattlefieldFilter {
                        subtype: Some("Scout"),
                        ..CONTROLLED
                    },
                    BattlefieldFilter {
                        subtype: Some("Pirate"),
                        ..CONTROLLED
                    },
                    BattlefieldFilter {
                        subtype: Some("Rogue"),
                        ..CONTROLLED
                    },
                ],
                kind: CounterKind::PlusOne,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
