// Direct typed behavior from complete pinned Oracle and XMage sources.
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: None,
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    stats: Stats {
        power: 1,
        toughness: 1,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::StaticModifier(0, BONUS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::End,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::Draw(PlayerSet::Controller, 1)],
    }],
    extras: &CardExtras {
        static_modifiers: &[BONUS],
        adventure_identity: Some("The Arkenstone // Seek the Heart"),
        adventure_spell: Some(Card(5411)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
