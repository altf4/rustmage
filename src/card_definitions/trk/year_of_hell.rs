// Oracle and pinned XMage reviewed. Every chapter has the same ordered effects.
const CHAPTER: &[Instruction] = &[
    Instruction::Draw(PlayerSet::Each, 1),
    Instruction::DamagePlayers(PlayerSet::Each, 2),
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::NONE,
            program: CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: CHAPTER,
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: CHAPTER,
        },
    ],
    ..CardRules::EMPTY
};
