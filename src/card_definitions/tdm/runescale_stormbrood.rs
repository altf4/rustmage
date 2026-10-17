// Typed behavior lifted unchanged from the compiled catalogue. Edit this Rust directly.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 4),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::AnyOf(&[CastFilter::Noncreature, CastFilter::Subtype("Dragon")]),
            ordinal: None,
        },
        targets: TargetPlan::Single(TargetSpec::NONE),
        program: &[Instruction::Grow {
            subject: Subject::Source,
            power: 2,
            toughness: 0,
        }],
    }],
    activated: &[],
    enters: Counters {
        plus_one: 0,
        minus_one: 0,
        stun: 0,
        loyalty: 0,
        lore: 0,
    },
    entry: EntryTapped::Never,
    keywords: Keywords(1),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
    }],
    extras: &CardExtras {
        static_ability_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::IntrinsicKeywords(Keywords(1))],
        }],
        adventure_identity: Some("Runescale Stormbrood // Chilling Screech"),
        omen_spell: Some(Card(5231)),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
