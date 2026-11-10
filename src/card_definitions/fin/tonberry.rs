// Direct typed behavior from the pinned Oracle and XMage sources.
const KNIFE: KeywordGrant = KeywordGrant {
    condition: None,
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
    own_turn: true,
    keywords: Keywords::FIRST_STRIKE.union(Keywords::DEATHTOUCH),
};

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 1),
    entry: EntryTapped::Always,
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 1,
        loyalty: 0,
        lore: 0,
    },
    static_groups: &[StaticAbilityGroup {
        line: 1,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::KeywordGrant(0, KNIFE)],
    }],
    extras: &CardExtras {
        keyword_grants: &[KNIFE],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
