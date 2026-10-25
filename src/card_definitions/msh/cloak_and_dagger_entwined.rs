// Direct full Oracle behavior, including independent partial target legality.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    keywords: Keywords::LIFELINK.union(Keywords::DEATHTOUCH),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            }),
            TargetSpec {
                rule: TargetRule::Grouped {
                    base: &TargetRule::Creature,
                    constraint: TargetGroupConstraint::ControllerOfPlayerGroup(0),
                },
                min: 0,
                max: 1,
            },
        ]),
        program: &[Instruction::RevealTargetHandExileUntilSourceLeaves],
    }],
    ..CardRules::EMPTY
};
