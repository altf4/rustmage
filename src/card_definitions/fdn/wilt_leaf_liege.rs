// Direct typed behavior from the complete pinned Oracle and XMage sources.
const GREEN: LiveStaticEffect = colored(ChoiceColor::Green);
const WHITE: LiveStaticEffect = colored(ChoiceColor::White);
const fn colored(color: ChoiceColor) -> LiveStaticEffect {
    LiveStaticEffect {
        recipients: LiveStatRecipients::OtherColoredCreaturesYouControl(color),
        condition: None,
        power: None,
        toughness: None,
        bonus_power: 1,
        bonus_toughness: 1,
        keywords: Keywords::NONE,
        cannot_block_subtype: None,
    }
}
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    static_groups: &[
        StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::LiveBonus(0, GREEN)],
        },
        StaticAbilityGroup {
            line: 1,
            first_layer: AbilityLayer::Modify,
            fragments: &[StaticAbilityFragment::LiveBonus(1, WHITE)],
        },
    ],
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::OpponentDiscardToBattlefield),
        live_static_effects: &[GREEN, WHITE],
        cost_alternatives: &[
            ManaCost {
                colored: [1, 0, 0, 0, 2, 0],
                generic: 1,
            },
            ManaCost {
                colored: [2, 0, 0, 0, 1, 0],
                generic: 1,
            },
            ManaCost {
                colored: [3, 0, 0, 0, 0, 0],
                generic: 1,
            },
        ],
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
