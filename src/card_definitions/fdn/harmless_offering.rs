// Direct typed Oracle behavior; source and dependencies pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
            TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 0,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            }),
        ]),
        &[Instruction::GainControl {
            subject: Subject::Targets,
            controller: PlayerSet::Targeted,
        }],
    ),
    ..CardRules::EMPTY
};
