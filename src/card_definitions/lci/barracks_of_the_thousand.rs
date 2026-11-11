const GNOME_SOLDIER: Card = Card::from_catalogue_id(5573);

pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Land,
    activated: &[ActivatedAbility {
        exhaust: false,
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Battlefield,
        cycling: false,
        cost: ActivationCost {
            tap: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::NONE,
        program: &[Instruction::AddMana(ManaProduction::Fixed([
            1, 0, 0, 0, 0, 0,
        ]))],
        sorcery: false,
        once_per_turn: false,
    }],
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SpellCastUsingSourceMana(CastFilter::AnyOf(&[
            CastFilter::Artifact,
            CastFilter::Creature,
        ])),
        targets: TargetPlan::NONE,
        program: &[Instruction::CreateTokens {
            token: GNOME_SOLDIER,
            amount: 1,
            tapped: false,
        }],
    }],
    ..CardRules::EMPTY
};
