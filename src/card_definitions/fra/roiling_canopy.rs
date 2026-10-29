// Direct Oracle and pinned XMage behavior; gated pending integration tests.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SubtypeEntersYouWithOthers { subtype: "Forest", minimum: 5 },
        targets: one(TargetRule::creature_you_control()),
        program: &[Instruction::Grow { subject: Subject::Targets, power: 3, toughness: 3 }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield, cycling: false,
        cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None, mana_component: false, tap: true, sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
        targets: TargetPlan::NONE, program: &[Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 0, 1, 0]))], sorcery: false, once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
