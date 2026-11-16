// Oracle and official TRK release notes reviewed; the pinned XMage class is absent.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    keywords: Keywords::HASTE,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfDies,
        targets: TargetPlan::NONE,
        program: &[Instruction::ExileTopPlay {
            amount: 1,
            next_turn: true,
        }],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        exhaust: false,
        sorcery: true,
        once_per_turn: false,
        cost: ActivationCost {
            generic: 2,
            colored: [0, 0, 0, 1, 0, 0],
            mana_component: true,
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateSourceCopyWithDelayedSacrifice],
    }],
    ..CardRules::EMPTY
};
