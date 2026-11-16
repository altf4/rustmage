// Pinned Oracle and complete XMage class: the Aura preserves card types while
// replacing creature subtypes. Threshold is checked at trigger and resolution.
const OWN_GRAVEYARD_CARDS: StatValue = StatValue::Count(CountFilter {
    zone: CountZone::Graveyard,
    relation: 0,
    kind: 0,
    exclude: 0,
    subtype: None,
    basic: false,
    other: false,
    cards_only: true,
});
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Aura(one(TargetRule::Creature)),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: -4,
                toughness: 0,
            },
            keywords: Keywords::CANT_BLOCK,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Type,
        fragments: &[StaticAbilityFragment::AttachedCharacteristics(
            &AttachedCharacteristics {
                name: None,
                form: PermanentForm::Preserve,
                creature_types: &["Salamander"],
                colors: &[],
                base: None,
            },
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::AtStep {
            step: TriggerStep::Upkeep,
            relation: CastRelation::Any,
            guard: StepGuard::Expression {
                value: OWN_GRAVEYARD_CARDS,
                minimum: Some(7),
                maximum: None,
            },
        },
        targets: TargetPlan::NONE,
        program: &[
            Instruction::IfAmountAtLeast {
                value: AmountValue::Stat(OWN_GRAVEYARD_CARDS),
                minimum: 7,
                skip: 1,
            },
            Instruction::ExileAttached,
        ],
    }],
    ..CardRules::EMPTY
};
