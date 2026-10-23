const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 2,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::ConvokeCreatureSpells],
}];
// The spell grant is evaluated in actual and speculative mana worlds.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING.union(Keywords::LIFELINK),
    extras: &CardExtras {
        transform: Some(TransformSpec {
            front: Card(1273),
            back: Card(5350),
            front_name: "Eirdu, Carrier of Dawn",
        }),
        ..CardExtras::EMPTY
    },
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayMana {
                cost: ManaCost {
                    colored: [0, 0, 1, 0, 0, 0],
                    generic: 0,
                },
                skip: 1,
            },
            Instruction::TransformSource,
        ],
    }],
    ..CardRules::EMPTY
};
