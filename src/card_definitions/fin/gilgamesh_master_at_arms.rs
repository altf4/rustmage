pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEntersOrAttacks,
            targets: TargetPlan::NONE,
            program: &[Instruction::SelectLibraryThenReflexiveEquipment {
                effect: LibrarySelectionEffect {
                    spec: LibrarySelectionSpec {
                        look: 6,
                        count: LibrarySelectionCount::All,
                        filter: LibrarySelectionFilter::Subtype("Equipment"),
                        optional: true,
                        reveal: false,
                        remainder: LibraryRemainder::BottomRandom,
                    },
                    look: LibraryLook::Fixed(6),
                    destination: LibraryPickDestination::Battlefield,
                    public: false,
                    maximum_mana_value: None,
                },
                ability: 1,
            }],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::MayAttachReflexiveEquipment {
                recipient_subtype: "Samurai",
            }],
        },
    ],
    ..CardRules::EMPTY
};
