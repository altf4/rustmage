// Authored from the complete pinned Oracle and XMage implementations.
const fn loyalty(n: i32) -> ActivationCost {
    ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
        mana_component: false, tap: false, sacrifice: false, exile_self: false,
        discard: false, life: 0, remove: None, loyalty: Some(n) }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras { starting_loyalty: 4, ..CardExtras::EMPTY },
    activated: &[
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(1), targets: TargetPlan::NONE,
            program: &[Instruction::ExileTopMayCastPaid, Instruction::IfOfferedCastCommitted { skip: 1 }, Instruction::DamagePlayers(PlayerSet::Opponent, 2)],
            sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(1), targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([0, 0, 0, 2, 0, 0]))],
            sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(-3), targets: TargetPlan::Single(TargetSpec::one(TargetRule::Creature)),
            program: &[Instruction::Damage(4)], sorcery: true, once_per_turn: false },
        ActivatedAbility { exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false, cost: loyalty(-7), targets: TargetPlan::NONE,
            program: &[Instruction::CreateEmblem(Card(5465))], sorcery: true, once_per_turn: false },
    ],
    ..CardRules::EMPTY
};
