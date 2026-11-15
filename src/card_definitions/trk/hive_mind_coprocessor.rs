// Pinned XMage HiveMindCoprocessor: each controlled artifact creature acquires its own trigger.
const COMBAT_PLAYER: TriggerCondition = TriggerCondition::PlayerDamaged {
    subject: DamageSubject::Source,
    combat_only: true,
    opponent_only: false,
    one_or_more: false,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::ControlledArtifactCreatureTrigger { ability: 0 }],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&COMBAT_PLAYER),
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayControllerLife { amount: 1, skip: 2 },
            Instruction::LoseLife(PlayerSet::Controller, 1),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
