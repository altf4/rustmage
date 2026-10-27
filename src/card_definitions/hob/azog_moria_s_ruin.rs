pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: -1,
                other: true,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::Destroy,
            Instruction::EnsureGoblinArmyFor(PlayerSet::TargetController),
            Instruction::AmassGoblinsScaled {
                player: PlayerSet::TargetController,
                amount: AmountValue::TargetCreaturePower,
            },
            Instruction::DrawIfTargetController,
        ],
    }],
    ..CardRules::EMPTY
};
