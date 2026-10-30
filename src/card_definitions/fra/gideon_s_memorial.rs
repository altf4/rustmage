// Direct Oracle and pinned XMage rules; gated pending integration validation.
const TOKENS: CreatureFilter = CreatureFilter {
    relation: 0,
    other: false,
    subtype: None,
    keyword: Keywords::NONE,
    attacking: false,
    tokens: Some(true),
    artifact: false,
};
const MODIFIER: StaticModifier = StaticModifier {
    own_turn: false,
    requires_life_gain: false,
    filter: TOKENS,
    stats: Stats {
        power: 1,
        toughness: 0,
    },
};
const VIGILANCE: KeywordGrant = KeywordGrant {
    condition: None,
    requires_life_gain: false,
    filter: TOKENS,
    source_only: false,
    own_turn: false,
    keywords: Keywords::VIGILANCE,
};
const GROUPS: &[StaticAbilityGroup] = &[StaticAbilityGroup {
    line: 0,
    first_layer: AbilityLayer::Ability,
    fragments: &[
        StaticAbilityFragment::StaticModifier(0, MODIFIER),
        StaticAbilityFragment::KeywordGrant(0, VIGILANCE),
    ],
}];
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: GROUPS,
    extras: &CardExtras {
        static_ability_groups: GROUPS,
        static_modifiers: &[MODIFIER],
        keyword_grants: &[VIGILANCE],
        ..CardExtras::EMPTY
    },
    activated: &[
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            cycling: false,
            cost: ActivationCost {
                colored: [0; 6],
                generic: 0,
                alternatives: &[],
                additional: None,
                mana_component: false,
                tap: true,
                sacrifice: false,
                exile_self: false,
                discard: false,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Tagged {
                production: &ManaProduction::Choice {
                    mask: 31,
                    amount: 1,
                },
                quality: &ManaQuality {
                    restriction: ManaRestriction::PlaneswalkerSpell,
                    spell_bonus: ManaSpellBonus::None,
                    retention: ManaRetention::Step,
                },
            })],
            sorcery: false,
            once_per_turn: false,
        },
        ActivatedAbility {
            exhaust: false,
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Hand,
            cycling: false,
            cost: ActivationCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 1,
                alternatives: &[],
                additional: None,
                mana_component: true,
                tap: false,
                sacrifice: false,
                exile_self: false,
                discard: true,
                life: 0,
                remove: None,
                loyalty: None,
            },
            targets: one(TargetRule::Filtered {
                base: &TargetRule::Creature,
                predicate: &TargetPredicate::AttackingOrBlocking,
            }),
            program: &[Instruction::Damage(4)],
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..CardRules::EMPTY
};
