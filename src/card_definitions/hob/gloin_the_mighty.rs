// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Main1,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            0, 0, 0, 2, 0, 0,
        ]))],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(0),
    extras: &CardExtras {
        adventure_identity: Some("Glóin the Mighty // Easy Pickings"),
        adventure_spell: Some(Card(5201)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
