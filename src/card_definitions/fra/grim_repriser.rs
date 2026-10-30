// Direct Oracle and pinned XMage behavior; gated pending validation.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Noncreature,
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 1,
            toughness: 1,
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::OpponentNoncombatDamageThisTurn,
        origin: ActivationOrigin::Graveyard,
        cycling: false,
        targets: TargetPlan::NONE,
        once_per_turn: false,
        sorcery: false,
        cost: ActivationCost {
            colored: [0, 0, 1, 1, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: false,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        program: &[Instruction::ReturnSourceToBattlefieldWithCounter {
            tapped: false,
            counter: CounterKind::Finality,
        }],
    }],
    ..CardRules::EMPTY
};
