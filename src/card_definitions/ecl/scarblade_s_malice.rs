// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        one(TargetRule::creature_you_control()),
        &[
            Instruction::GrantKeywords {
                subject: Subject::Targets,
                keywords: Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
            },
            Instruction::DelayedDeathThisTurn { ability: 0 },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Reflexive,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: Card(5325),
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
