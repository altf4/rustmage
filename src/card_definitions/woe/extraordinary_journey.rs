// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec {
                rule: TargetRule::Creature,
                min: 0,
                max: usize::MAX,
            }),
            program: &[
                Instruction::UpToTargets(AmountValue::ChosenX),
                Instruction::ExileTargetsPlayForOwnersWhileExiled,
            ],
        },
        TriggeredAbility {
            condition: TriggerCondition::OncePerTurn(
                &TriggerCondition::NontokenCreaturesEnterFromExile,
            ),
            targets: TargetPlan::NONE,
            program: &[Instruction::Draw(PlayerSet::Controller, 1)],
        },
    ],
    extras: &CardExtras {
        x_symbols: 2,
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
