// Direct Oracle behavior; the death copy uses frozen copiable source facts.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 2),
    keywords: Keywords::FLASH.union(Keywords::FLYING),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfAttacks,
            targets: TargetPlan::NONE,
            program: &[
                Instruction::Draw(PlayerSet::Controller, 1),
                Instruction::Discard(PlayerSet::Controller, 1),
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::SelfNontokenDies,
            targets: TargetPlan::NONE,
            program: &[Instruction::CreateCopyToken {
                subject: Subject::Source,
                amount: 1,
            }],
        },
    ],
    ..CardRules::EMPTY
};
