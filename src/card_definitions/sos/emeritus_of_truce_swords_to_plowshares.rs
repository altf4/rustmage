// Direct typed Oracle/XMage behavior; full printed obligations pinned in paired provenance.
pub(in crate::cards) const RULES: CardRules = CardRules {
    effect: Effect::Creature(3, 3),
    extras: &CardExtras {
        prepared_spell: Some(Card(5360)),
        ..CardExtras::EMPTY
    },
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: one(TargetRule::Player {
            opponent_only: false,
        }),
        program: &[
            Instruction::CreateTokensFor {
                players: PlayerSet::Targeted,
                token: Card(5315),
                amount: 1,
                tapped: false,
            },
            Instruction::If {
                guard: StepGuard::OpponentControlsMoreCreatures,
                skip: 1,
            },
            Instruction::PrepareSource,
        ],
    }],
    ..CardRules::EMPTY
};
