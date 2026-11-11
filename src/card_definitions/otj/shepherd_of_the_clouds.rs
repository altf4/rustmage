const OWN_PERMANENT_CARD: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: 3,
    },
    predicate: &TargetPredicate::Permanent,
};

const HAS_MOUNT: StepGuard = StepGuard::Expression {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 0,
        exclude: 0,
        subtype: Some("Mount"),
        basic: false,
        other: false,
        cards_only: false,
    }),
    minimum: Some(1),
    maximum: None,
};

const KEYWORDS: Keywords = Keywords::FLYING.union(Keywords::VIGILANCE);
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 3),
    keywords: KEYWORDS,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(KEYWORDS)],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(OWN_PERMANENT_CARD),
        program: &[
            Instruction::If {
                guard: HAS_MOUNT,
                skip: 2,
            },
            Instruction::ReturnToBattlefieldUnderControl,
            Instruction::Skip(1),
            Instruction::ReturnToHand,
        ],
    }],
    ..CardRules::EMPTY
};
