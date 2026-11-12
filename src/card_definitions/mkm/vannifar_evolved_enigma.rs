pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Modal(&[
            SpellMode {
                text: "Cloak a card from your hand.",
                targets: TargetPlan::NONE,
                program: &[Instruction::CloakFromHand],
            },
            SpellMode {
                text: "Put a +1/+1 counter on each colorless creature you control.",
                targets: TargetPlan::NONE,
                program: &[Instruction::PutPermanentCountersWithColors {
                    filters: &[BattlefieldFilter {
                        relation: 0,
                        kinds: 1,
                        nonland: false,
                        tapped: None,
                        without: Keywords::NONE,
                        subtype: None,
                        max_mana: u32::MAX,
                    }],
                    colors: 0,
                    kind: CounterKind::PlusOne,
                    amount: AmountValue::Fixed(1),
                }],
            },
        ])],
    }],
    ..CardRules::EMPTY
};
