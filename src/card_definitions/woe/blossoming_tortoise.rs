// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(3)),
            Instruction::ChooseOwnGraveyardLandReturn { tapped: true },
        ],
    }],
    static_groups: &[
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Rule,
            fragments: &[StaticAbilityFragment::ControlledActivationReduction {
                kinds: 2,
                amount: ActivationReductionAmount::Fixed(1),
                leave_one_mana: false,
            }],
        },
        StaticAbilityGroup {
            line: 2,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::ControlledCreatureKindBonus {
                kinds: 2,
                stats: Stats {
                    power: 1,
                    toughness: 1,
                },
            }],
        },
    ],
    ..CardRules::EMPTY
};
