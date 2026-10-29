// Direct Oracle and pinned XMage rules; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast { relation: CastRelation::You, filter: CastFilter::PreparedSpell, ordinal: None },
        targets: TargetPlan::NONE,
        program: &[Instruction::CopyPreparedCastSpell],
    }],
    activated: &[ActivatedAbility {
        exhaust: false, guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
        cycling: false, targets: TargetPlan::NONE, once_per_turn: false, sorcery: false,
        cost: ActivationCost { colored: [1, 1, 1, 1, 1, 0], generic: 0, alternatives: &[], additional: None,
            mana_component: true, tap: true, sacrifice: false, exile_self: false,
            discard: false, life: 0, remove: None, loyalty: None },
        program: &[Instruction::PrepareControlledCreatures],
    }],
    ..CardRules::EMPTY
};
