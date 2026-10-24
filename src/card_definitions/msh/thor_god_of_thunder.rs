// Direct typed behavior from the complete pinned Oracle and XMage sources.
const GRAVE_SPELL: TargetRule = TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 12,
    relation: 0,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
};
const GRAVE_EQUIPMENT: TargetRule = TargetRule::Filtered {
    base: &TargetRule::Object {
        zone: TargetZone::Graveyard,
        kinds: 0,
        relation: 0,
        other: false,
        nonland: false,
        tapped: false,
        min_mana: 0,
        max_mana: u32::MAX,
    },
    predicate: &TargetPredicate::Subtype("Equipment"),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    keywords: Keywords::FLYING,
    triggers: &[
        TriggeredAbility {
            condition: TriggerCondition::SelfEnters,
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Union(&[
                GRAVE_SPELL,
                GRAVE_EQUIPMENT,
            ]))),
            program: &[Instruction::ExileTargetsPlay { next_turn: true }],
        },
        TriggeredAbility {
            condition: TriggerCondition::SpellCast {
                relation: CastRelation::You,
                filter: CastFilter::Noncreature,
                ordinal: None,
            },
            targets: TargetPlan::Single(TargetSpec::one(TargetRule::Any)),
            program: &[Instruction::DamageScaled(AmountValue::CastSpellManaValue)],
        },
    ],
    ..CardRules::EMPTY
};
