pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfTurnedFaceUp,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Tagged {
            production: &ManaProduction::Choice {
                mask: 31,
                amount: 2,
            },
            quality: &ManaQuality {
                restriction: ManaRestriction::Unrestricted,
                spell_bonus: ManaSpellBonus::None,
                retention: ManaRetention::Turn,
            },
        })],
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        sorcery: false,
        once_per_turn: false,
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 0, 0, 1,
        ]))],
    }],
    extras: &CardExtras {
        disguise: Some(DisguiseSpec::fixed(generic(3))),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
