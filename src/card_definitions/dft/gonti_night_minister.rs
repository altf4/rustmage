// Complete Oracle/XMage: actual caster ownership and original damage-controller hidden entitlement.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::Any,
                filter: CastFilter::NotOwnedByCaster,
                ordinal: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensFor {
                players: PlayerSet::CastingPlayer,
                token: Card(5302),
                amount: 1,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::PlayerDamaged {
                subject: DamageSubject::Creatures {
                    filter: CreatureFilter {
                        relation: -1,
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
                opponent_only: true,
                one_or_more: false,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::ExileLibraryTopFaceDownPlay {
                library: PlayerSet::DamagedPlayer,
                beneficiary: PlayerSet::EventPlayer,
                amount: 1,
                spend_any_mana: true,
            }],
        },
    ],
    ..CardRules::EMPTY
};
