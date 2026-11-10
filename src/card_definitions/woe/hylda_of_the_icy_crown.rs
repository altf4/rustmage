// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 4),
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::OpponentCreatureTappedByYou { one_or_more: false },
            targets: TargetPlan::NONE,
            program: &[
                Instruction::MayPayMana {
                    cost: generic(1),
                    skip: 1,
                },
                Instruction::Reflexive { ability: 1 },
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::Reflexive,
            targets: TargetPlan::NONE,
            program: &[Instruction::Modal(&[
                SpellMode {
                    text: "Create a 4/4 white and blue Elemental creature token.",
                    targets: TargetPlan::NONE,
                    program: &[Instruction::CreateTokens {
                        token: Card(5547),
                        amount: 1,
                        tapped: false,
                    }],
                },
                SpellMode {
                    text: "Put a +1/+1 counter on each creature you control.",
                    targets: TargetPlan::NONE,
                    program: &[Instruction::PutPermanentCountersScaled {
                        filters: &[BattlefieldFilter {
                            kinds: 1,
                            relation: 0,
                            nonland: false,
                            tapped: None,
                            without: Keywords::NONE,
                            subtype: None,
                            max_mana: u32::MAX,
                        }],
                        kind: CounterKind::PlusOne,
                        amount: AmountValue::Fixed(1),
                    }],
                },
                SpellMode {
                    text: "Scry 2, then draw a card.",
                    targets: TargetPlan::NONE,
                    program: &[
                        Instruction::Scry(2),
                        Instruction::Draw(PlayerSet::Controller, 1),
                    ],
                },
            ])],
        },
    ],
    ..CardRules::EMPTY
};
