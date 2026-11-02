// Physical cards in all exile zones count; departures are one before-world batch on our turn.
const RESTRICTION: CombatRestriction = CombatRestriction {
    subject: Subject::Source,
    attack: true,
    block: true,
    unless: Some(StepGuard::Expression {
        value: StatValue::Count(CountFilter {
            zone: CountZone::Exile,
            relation: -1,
            kind: 0,
            exclude: 0,
            subtype: None,
            basic: false,
            other: false,
            cards_only: true,
        }),
        minimum: Some(7),
        maximum: None,
    }),
};
const GROUPS: &[StaticAbilityGroup] = &[
    StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::MENACE
                .union(Keywords::LIFELINK)
                .union(Keywords::INDESTRUCTIBLE),
        )],
    },
    StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Rule,
        fragments: &[StaticAbilityFragment::CombatRestriction(0, RESTRICTION)],
    },
];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::MENACE
        .union(Keywords::LIFELINK)
        .union(Keywords::INDESTRUCTIBLE),
    static_groups: GROUPS,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::When {
            event: &TriggerCondition::CardsExiledFromBattlefieldOrGraveyard,
            guard: StepGuard::YourTurn,
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::Draw(PlayerSet::Controller, 1),
            Instruction::LoseLife(PlayerSet::Controller, 1),
        ],
    }],
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        combat_restriction: Some(RESTRICTION),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
