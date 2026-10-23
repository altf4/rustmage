pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[Instruction::CopyNextInstantSorcery { ability: 0 }],
    ),
    // The delayed trigger retains this definition after the instant leaves the stack.
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::InstantSorcery,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyCastSpell {
            prior_spells: false,
        }],
    }],
    extras: &CardExtras {
        cost_alternatives: &[
            ManaCost {
                colored: [0, 1, 0, 1, 0, 0],
                generic: 0,
            },
            ManaCost {
                colored: [0, 2, 0, 0, 0, 0],
                generic: 0,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
