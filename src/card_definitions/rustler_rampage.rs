// Complete printed behavior; mode combinations resolve in printed order.
// Base mana, identity and format metadata come from the pinned card facts.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Modal(&[
        SpellMode {
            text: "Untap selected player creatures",
            targets: one(TargetRule::Player {
                opponent_only: false,
            }),
            program: &[Instruction::UntapPlayerCreatures(PlayerSet::Targeted)],
        },
        SpellMode {
            text: "Double strike",
            targets: one(TargetRule::Creature),
            program: &[Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DOUBLE_STRIKE,
            }],
        },
        SpellMode {
            text: "Untap selected player creatures + Double strike",
            targets: TargetPlan::Groups(&[
                TargetSpec::one(TargetRule::Player {
                    opponent_only: false,
                }),
                TargetSpec::one(TargetRule::Creature),
            ]),
            program: &[
                Instruction::UntapPlayerCreatures(PlayerSet::Targeted),
                Instruction::SelectGroup(1),
                Instruction::GrantKeywords {
                    subject: Subject::Targets,
                    keywords: Keywords::DOUBLE_STRIKE,
                },
            ],
        },
    ]),
    mode_costs: Some(&[generic(1), generic(1), generic(2)]),
    ..CardRules::EMPTY
};
