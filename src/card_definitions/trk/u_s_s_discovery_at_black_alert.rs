// Oracle and pinned XMage reviewed. Discovery's entry and attack events share one trigger.
const BASE: BaseStatSetter = BaseStatSetter {
    power: Some(5), toughness: Some(5), own_turn: false,
    condition: Some(StatValue::AtLeast {
        value: &StatValue::SourceCounters(CounterKind::Charge), minimum: 6,
    }),
};
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    static_groups: &[StaticAbilityGroup {
        line: 0, first_layer: AbilityLayer::Type,
        fragments: &[
            StaticAbilityFragment::SourceCounterCreature {
                own_turn: false, kind: CounterKind::Charge, minimum: 6,
                form: SourceCreatureForm::AddCreature, subtypes: &[],
                keywords: Keywords::FLYING,
            },
            StaticAbilityFragment::BaseStatSetter(0, BASE),
        ],
    }],
    extras: &CardExtras { base_stat_setter: Some(BASE), ..CardExtras::EMPTY },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEntersOrAttacks,
        targets: TargetPlan::NONE,
        program: &[Instruction::Discover(5)],
    }],
    activated: &[ActivatedAbility {
        guard: ActivationGuard::Always, origin: ActivationOrigin::Battlefield,
        cycling: false, exhaust: false, sorcery: true, once_per_turn: false,
        cost: ActivationCost {
            additional: Some(AdditionalCost {
                kind: EffectChoiceKind::TapPermanents(1),
                filter: SacrificeFilter {
                    keywords: Keywords::NONE, source_only: false, kinds: 1,
                    nonland: false, other: true, tokens: None,
                    artifact_enchantment_or_token: false, subtype: None,
                },
            }),
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::PutCountersScaled {
            subject: Subject::Source, kind: CounterKind::Charge,
            amount: AmountValue::TappedCostCreaturePower,
        }],
    }],
    ..CardRules::EMPTY
};
