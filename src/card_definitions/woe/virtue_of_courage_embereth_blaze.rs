// Direct typed behavior from the pinned Oracle and XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PlayerDamaged {
            subject: DamageSubject::ControlledNoncombatSources,
            combat_only: false,
            opponent_only: true,
            one_or_more: false,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::May { skip: 1 },
            Instruction::ExileTopPlayScaled {
                amount: AmountValue::DamageEventAmount,
                next_turn: false,
            },
        ],
    }],
    extras: &CardExtras {
        adventure_spell: Some(Card(5563)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
