// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: false,
            }),
            TargetSpec::one(TargetRule::creature_you_control()),
        ]),
        &[
            Instruction::CreateCopyTokenForPlayer {
                subject: Subject::Targets,
                amount: 1,
                player: PlayerSet::Targeted,
            },
            Instruction::Paradigm { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CastParadigmCopy],
    }],
    ..CardRules::EMPTY
};
