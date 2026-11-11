const MERCENARY: Card = Card::from_catalogue_id(5612);

const fn outlaw_haste(subtype: &'static str) -> KeywordGrant {
    KeywordGrant {
        condition: None,
        requires_life_gain: false,
        filter: CreatureFilter {
            relation: 0,
            other: true,
            subtype: Some(subtype),
            keyword: Keywords::NONE,
            attacking: false,
            tokens: None,
            artifact: false,
        },
        source_only: false,
        own_turn: false,
        keywords: Keywords::HASTE,
    }
}

const ASSASSINS: KeywordGrant = outlaw_haste("Assassin");
const MERCENARIES: KeywordGrant = outlaw_haste("Mercenary");
const PIRATES: KeywordGrant = outlaw_haste("Pirate");
const ROGUES: KeywordGrant = outlaw_haste("Rogue");
const WARLOCKS: KeywordGrant = outlaw_haste("Warlock");

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: MERCENARY,
            amount: 2,
            tapped: false,
        }],
    }],
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[
            StaticAbilityFragment::KeywordGrant(0, ASSASSINS),
            StaticAbilityFragment::KeywordGrant(1, MERCENARIES),
            StaticAbilityFragment::KeywordGrant(2, PIRATES),
            StaticAbilityFragment::KeywordGrant(3, ROGUES),
            StaticAbilityFragment::KeywordGrant(4, WARLOCKS),
        ],
    }],
    extras: &CardExtras {
        keyword_grants: &[ASSASSINS, MERCENARIES, PIRATES, ROGUES, WARLOCKS],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
