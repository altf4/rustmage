// Directly authored from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::Groups(&[
            TargetSpec::one(TargetRule::creature_you_control()),
            TargetSpec::one(TargetRule::Filtered {
                base: &TargetRule::Object {
                    zone: TargetZone::Battlefield,
                    kinds: 16,
                    relation: -1,
                    other: false,
                    nonland: false,
                    tapped: false,
                    min_mana: 0,
                    max_mana: u32::MAX,
                },
                predicate: &TargetPredicate::Subtype("Equipment"),
            }),
        ]),
        &[
            Instruction::SelectGroup(1),
            Instruction::GainControlUntilEndTurn,
            Instruction::AttachEquipmentByGroups {
                equipment: 1,
                recipient: 0,
            },
            Instruction::DelayTargetEvent {
                event: DelayedTargetEvent::ControlLostWhileAttached,
                ability: 0,
            },
        ],
    ),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::DelayedTargetControlLostWhileAttached,
        targets: TargetPlan::NONE,
        program: &[Instruction::UnattachEventObject],
    }],
    ..CardRules::EMPTY
};
