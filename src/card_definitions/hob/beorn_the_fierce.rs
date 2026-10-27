// Complete Oracle obligations; see paired full-source provenance.
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: true,
        subtype: Some("Bear"),
        keyword: Keywords::NONE,
        attacking: false,
        tokens: None,
        artifact: false,
    },
    stats: Stats {
        power: 2,
        toughness: 2,
    },
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(6, 6),
    keywords: Keywords::TRAMPLE,
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Modify,
        fragments: &[StaticAbilityFragment::StaticModifier(0, BONUS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            rule: TargetRule::Object {
                zone: TargetZone::Battlefield,
                kinds: 1,
                relation: 0,
                other: false,
                nonland: false,
                tapped: false,
                min_mana: 0,
                max_mana: u32::MAX,
            },
            min: 0,
            max: 1,
        }),
        program: &[
            Instruction::PutCounters {
                subject: Subject::Targets,
                kind: CounterKind::Trample,
                amount: 1,
            },
            Instruction::AddCreatureTypes {
                subject: Subject::Targets,
                types: &["Bear"],
            },
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(StatValue::Count(CountFilter {
                    zone: CountZone::Battlefield,
                    relation: 0,
                    kind: 0,
                    exclude: 0,
                    subtype: Some("Bear"),
                    basic: false,
                    other: false,
                    cards_only: false,
                })),
                minimum: 3,
                skip: 1,
            },
            Instruction::Draw(PlayerSet::Controller, 2),
        ],
    }],
    extras: &CardExtras {
        static_modifiers: &[BONUS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
