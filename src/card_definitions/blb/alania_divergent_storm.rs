pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::FirstSpellKindOrOtherCreatureType {
            kinds: 4 | 8,
            subtype: "Otter",
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player {
            opponent_only: true,
        })),
        program: &[
            Instruction::MayDraw { skip: 2 },
            Instruction::Draw(PlayerSet::Targeted, 1),
            Instruction::CopyCapturedSpell {
                count: SpellCopyCount::Fixed(1),
                retarget: true,
            },
        ],
    }],
    ..CardRules::EMPTY
};
