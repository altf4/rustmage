pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::ExileCastForZeroOncePerTurn],
    }],
    extras: &CardExtras {
        room: Some(&RoomSpec {
            halves: [Card(5627), Card(5628)],
            triggers: &[(0, RoomDoor::Left)],
            static_groups: &[(0, RoomDoor::Right)],
        }),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
