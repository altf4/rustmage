// Gated core consumer: whole-card admission awaits the reviewed consumer batch.
const ATTACKING_VAMPIRES: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: CreatureFilter {
        relation: 0,
        other: false,
        subtype: Some("Vampire"),
        keyword: Keywords::NONE,
        attacking: true,
        tokens: None,
        artifact: false,
    },
    source_only: false,
    own_turn: false,
    keywords: Keywords::DEATHTOUCH.union(Keywords::LIFELINK),
};
const STATIC_GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[StaticAbilityFragment::KeywordGrant(0, ATTACKING_VAMPIRES)],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(5, 5),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CreatureDies {
            relation: CastRelation::You,
            other: false,
            nontoken: false,
            subtype: Some("Vampire"),
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::MayPayControllerLife { amount: 2, skip: 2 },
            Instruction::LoseLife(PlayerSet::Controller, 2),
            Instruction::Draw(PlayerSet::Controller, 1),
        ],
    }],
    static_groups: STATIC_GROUPS,
    extras: &CardExtras {
        static_ability_groups: STATIC_GROUPS,
        keyword_grants: &[ATTACKING_VAMPIRES],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
