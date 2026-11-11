// The attack batch captures the actual Dinosaur count before any leave combat.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::ControlledSubtypeAttackDeclaration("Dinosaur"),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokensScaled {
                token: Card::from_catalogue_id(5302),
                amount: AmountValue::DeclaredAttackerCount,
                tapped: false,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(&TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Artifact,
                ordinal: None,
            }),
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateTokens {
                token: Card::from_catalogue_id(5570),
                amount: 1,
                tapped: false,
            }],
        },
    ],
    ..CardRules::EMPTY
};
