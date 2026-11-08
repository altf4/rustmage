// Direct typed behavior from complete pinned Oracle/XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 1,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[
            Instruction::If {
                guard: StepGuard::SourceSneakedThisTurn,
                skip: 2,
            },
            Instruction::Exile,
            Instruction::Skip(1),
            Instruction::ExileUntilSourceLeaves,
        ],
    }],
    extras: &CardExtras {
        sneak_costs: &[ManaCost {
            colored: [1, 0, 1, 0, 0, 0],
            generic: 2,
        }],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
