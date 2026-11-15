// Oracle and official Star Trek release notes; no pinned XMage class exists.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            guard: StepGuard::Expression {
                value: StatValue::NoncreatureSpellsCastThisTurn,
                minimum: Some(1),
                maximum: None,
            },
            event: &TriggerCondition::AtStep {
                step: TriggerStep::BeginCombat,
                relation: CastRelation::You,
                guard: StepGuard::Always,
            },
        },
        targets: one(TargetRule::creature_you_control()),
        program: &[
            Instruction::CreateCopyWithExceptionsAndTemporaryKeywordsAndDelayedSacrifice {
                exceptions: CopyExceptions {
                    remove_legendary: true,
                    ..CopyExceptions::NONE
                },
                keywords: Keywords::HASTE,
            },
        ],
    }],
    ..CardRules::EMPTY
};
