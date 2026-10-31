// Direct typed behavior from complete Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    entry: EntryTapped::Always,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfAttacks,
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Player { opponent_only: false })),
        program: &[Instruction::Mill(PlayerSet::Targeted, MillAmount::Fixed(4))],
    }],
    activated: &[
        ActivatedAbility {
            exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: true, sacrifice: false, exile_self: false, discard: false,
                life: 0, remove: None, loyalty: None },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([0, 1, 0, 0, 0, 0]))],
            sorcery: false, once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost { colored: [0; 6], generic: 0, alternatives: &[], additional: None,
                mana_component: false, tap: true, sacrifice: false, exile_self: false, discard: false,
                life: 0, remove: None, loyalty: None },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([0, 0, 1, 0, 0, 0]))],
            sorcery: false, once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost { colored: [0, 1, 1, 0, 0, 0], generic: 2, alternatives: &[], additional: None,
                mana_component: true, tap: false, sacrifice: false, exile_self: false, discard: false,
                life: 0, remove: None, loyalty: None },
            targets: TargetPlan::NONE,
            program: &[Instruction::AnimateSourceColored(AnimationSpec {
                artifact: false, creature_types: AnimationCreatureTypes::Named(&["Shark"]),
                base: Some((4, 4)), keywords: Keywords::DEATHTOUCH, duration: AnimationDuration::EndOfTurn,
            }, 2 | 4)],
            sorcery: false, once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
