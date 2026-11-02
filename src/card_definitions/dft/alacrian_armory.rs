// Direct typed behavior from complete pinned Oracle and XMage sources.
const OWN: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: None,
    artifact: false,
};
const BONUS: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: OWN,
    stats: Stats {
        power: 0,
        toughness: 1,
    },
};
const VIGILANCE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: OWN,
    source_only: false,
    own_turn: false,
    keywords: Keywords::VIGILANCE,
};
const PERMANENT: TargetRule = TargetRule::Object {
    zone: TargetZone::Battlefield,
    kinds: 0,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::StaticModifier(0, BONUS),
            StaticAbilityFragment::KeywordGrant(0, VIGILANCE),
        ],
    }],
    extras: &CardExtras {
        static_modifiers: &[BONUS],
        keyword_grants: &[VIGILANCE],
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::BeginCombat,
            relation: CastRelation::You,
            guard: StepGuard::Always,
        },
        targets: TargetPlan::Single(TargetSpec {
            min: 0,
            max: 1,
            rule: TargetRule::Union(&[
                TargetRule::Filtered {
                    base: &PERMANENT,
                    predicate: &TargetPredicate::Subtype("Mount"),
                },
                TargetRule::Filtered {
                    base: &PERMANENT,
                    predicate: &TargetPredicate::Subtype("Vehicle"),
                },
            ]),
        }),
        program: &[
            Instruction::IfSubjectSubtype {
                subject: Subject::Targets,
                subtype: "Mount",
                skip: 1,
            },
            Instruction::MarkSaddled(Subject::Targets),
            Instruction::IfSubjectSubtype {
                subject: Subject::Targets,
                subtype: "Vehicle",
                skip: 1,
            },
            Instruction::AnimateTargets(AnimationSpec {
                artifact: true,
                creature_types: AnimationCreatureTypes::Unchanged,
                base: None,
                keywords: Keywords::NONE,
                duration: AnimationDuration::EndOfTurn,
            }),
        ],
    }],
    ..CardRules::EMPTY
};
