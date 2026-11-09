// Direct typed behavior from the paired Oracle and pinned XMage sources.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::FirstQualifiedCreature(CreatureCastQualifier::Human),
            ordinal: None,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddCastEntryCountersForControlledSubtypes {
            kind: CounterKind::PlusOne,
            subtypes: &["Dog", "Wolf"],
        }],
    }],
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::any_color(1))],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
