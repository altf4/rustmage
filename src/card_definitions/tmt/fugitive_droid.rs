// Direct typed behavior from full pinned Oracle and XMage sources.
const GRANT: KeywordGrant = KeywordGrant {
    condition: Some(StatValue::ArtifactEnteredThisTurn),
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
    source_only: true,
    own_turn: false,
    keywords: Keywords::CANT_BE_BLOCKED,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(1, 1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, GRANT)],
    }],
    extras: &CardExtras {
        keyword_grants: &[GRANT],
        ..CardExtras::EMPTY
    },
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            colored: [0, 1, 0, 0, 0, 0],
            generic: 0,
            alternatives: &[],
            additional: None,
            mana_component: true,
            tap: false,
            sacrifice: true,
            exile_self: false,
            discard: false,
            life: 0,
            remove: None,
            loyalty: None,
        },
        targets: one(TargetRule::SpellTargetsControlledArtifactOrCreature),
        program: &[Instruction::Counter],
        sorcery: false,
        once_per_turn: false,
    }],
    ..CardRules::EMPTY
};
