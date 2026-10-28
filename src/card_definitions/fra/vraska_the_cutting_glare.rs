// Authored from pinned Oracle and complete XMage source; awaiting rules tests.
const SIX_LANDS: StepGuard = StepGuard::Expression {
    value: StatValue::Count(CountFilter {
        zone: CountZone::Battlefield,
        relation: 0,
        kind: 2,
        exclude: 0,
        subtype: None,
        basic: false,
        other: false,
        cards_only: false,
    }),
    minimum: Some(6),
    maximum: None,
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(4, 4),
    keywords: Keywords::DEATHTOUCH,
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::IntrinsicKeywords(
            Keywords::DEATHTOUCH,
        )],
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::InterveningIf {
            event: &TriggerCondition::SelfEnters,
            guard: SIX_LANDS,
        },
        targets: one(TargetRule::Object {
            zone: TargetZone::Battlefield,
            kinds: 0,
            relation: 1,
            other: false,
            nonland: false,
            tapped: false,
            min_mana: 0,
            max_mana: u32::MAX,
        }),
        program: &[Instruction::DestroyAndCreateForController {
            token: Card(5302),
            amount: 1,
        }],
    }],
    ..CardRules::EMPTY
};
