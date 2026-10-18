// Directly authored from the paired complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    keywords: Keywords::FLYING,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Exile another target creature you control. Return that card under its owner's control at the beginning of the next end step.",
                targets: TargetPlan::Single(TargetSpec::one(TargetRule::Filtered {
                    base: &TargetRule::creature_you_control(),
                    predicate: &TargetPredicate::OtherThanSource,
                })),
                program: &[Instruction::BlinkUntilEndStep { under_owner: true }],
            },
            SpellMode {
                text: "Create a 1/1 white Spirit creature token with flying.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CreateTokens {
                    token: Card(5342),
                    amount: 1,
                    tapped: false,
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
