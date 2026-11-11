// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::OncePerTurn(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::AnyOf(&[CastFilter::Noncreature, CastFilter::Subtype("Otter")]),
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayExileTop { skip: 1 },
            Instruction::ExileTopPlay {
                amount: 1,
                next_turn: false,
            },
        ],
    }],
    ..CardRules::EMPTY
};
