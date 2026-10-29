// Direct Oracle and pinned XMage rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::PermanentEntersYou {
            kinds: 65, nontoken: false, subtype: None, include_source: false,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Mill(PlayerSet::Controller, MillAmount::Fixed(2))],
    }],
    activated: &[ActivatedAbility {
        exhaust: true, guard: ActivationGuard::Always, cycling: false,
        origin: ActivationOrigin::Battlefield,
        cost: ActivationCost { colored: [0, 0, 1, 0, 0, 0], generic: 5,
            alternatives: &[], additional: None, mana_component: true, tap: false,
            sacrifice: false, exile_self: false, discard: false, life: 0, remove: None, loyalty: None },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::Object {
            zone: TargetZone::Graveyard, kinds: 65, relation: 0, other: false,
            nonland: false, tapped: false, min_mana: 0, max_mana: u32::MAX,
        })),
        program: &[Instruction::ReturnToBattlefield,
            Instruction::PutCounters { subject: Subject::Source, kind: CounterKind::PlusOne, amount: 1 }],
        sorcery: true, once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
