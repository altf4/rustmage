// Direct typed behavior from complete pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 1 },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
                zone: TargetZone::Graveyard,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            })),
            program: &[Instruction::ExileGraveyardAndCopy(CopyExceptions {
                nonlegendary: true,
                mutant: true,
                ..CopyExceptions::NONE
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 2 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyLinkedExiled(CopyExceptions {
                nonlegendary: true,
                mutant: true,
                ..CopyExceptions::NONE
            })],
        },
        TriggeredAbility {
            condition: TriggerCondition::Chapter { number: 3 },
            targets: TargetPlan::NONE,
            program: &[Instruction::CopyLinkedExiled(CopyExceptions {
                nonlegendary: true,
                mutant: true,
                ..CopyExceptions::NONE
            })],
        },
    ],
    ..CardRules::EMPTY
};
