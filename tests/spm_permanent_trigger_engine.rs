//! A graveyard activation grants a private combat trigger to its exact recipient.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const DEALT_TO_PLAYER: TriggerCondition = TriggerCondition::PlayerDamaged {
    subject: DamageSubject::Source,
    combat_only: true,
    opponent_only: false,
    one_or_more: false,
};

const PRIVATE_LOOK: &[Instruction] = &[Instruction::SelectLibraryTo(LibrarySelectionEffect {
    spec: LibrarySelectionSpec {
        look: 0,
        count: LibrarySelectionCount::Fixed(1),
        filter: LibrarySelectionFilter::Any,
        optional: false,
        reveal: false,
        remainder: LibraryRemainder::Graveyard,
    },
    look: LibraryLook::Amount(AmountValue::DamageEventAmount),
    destination: LibraryPickDestination::Hand,
})];

const GRAVE_GRANT: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::Granted(&DEALT_TO_PLAYER),
        targets: TargetPlan::NONE,
        program: PRIVATE_LOOK,
    }],
    activated: &[ActivatedAbility {
        cost: ActivationCost {
            exile_self: true,
            ..ActivationCost::ZERO
        },
        targets: TargetPlan::Single(TargetSpec::one(TargetRule::creature_you_control())),
        program: &[Instruction::GrantTargetPermanentTrigger(0)],
        guard: ActivationGuard::Always,
        origin: ActivationOrigin::Graveyard,
        sorcery: true,
        once_per_turn: false,
        cycling: false,
        exhaust: false,
    }],
    ..CardRules::EMPTY
};

fn hit(game: &mut Game, source: CardId, player: Player, combat: bool, amount: i128) {
    let event = DamageEvent {
        preventable: true,
        source: game.damage_source(source),
        target: Target::Player(player),
        amount,
        combat,
    };
    game.apply_damage_batch(&[event]);
    game.advance_replacements(false);
    game.prepare_triggers();
}

#[test]
fn exact_recipient_retains_the_grant_after_cleanup_and_its_controller_gets_the_private_look() {
    let _rules = fixture_rules::activate(&[(Card::HillGiant, &GRAVE_GRANT)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Hill Giant", Zone::Graveyard);
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    choose(
        &mut game,
        |action| matches!(action, Action::AnnounceActivate { card, ability: 0, .. } if *card == source),
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(recipient),
        },
    );
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Exile);
    settle(&mut game);

    let exact = game.object_ref(recipient);
    hit(&mut game, recipient, opponent, false, 2);
    assert!(game.state.stack.is_empty());

    game.begin_step(Step::End);
    game.give_priority();
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    game.record_control(exact, opponent, None);
    let milled = put(&mut game, opponent, "Island", Zone::Library);
    let kept = put(&mut game, opponent, "Opt", Zone::Library);
    hit(&mut game, recipient, player, true, 2);
    assert_eq!(game.state.stack.last().unwrap().controller, opponent);
    act(&mut game, Action::Pass);
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(opponent));
    assert!(game.observe(player).library_selection.is_none());
    assert!(game.observe(opponent).library_selection.is_some());
    assert!(
        game.legal_actions()
            .contains(&Action::SelectLibrary { card: kept })
    );
    assert!(
        game.legal_actions()
            .contains(&Action::SelectLibrary { card: milled })
    );
    act(&mut game, Action::SelectLibrary { card: kept });
    settle(&mut game);
    assert_eq!(game.object_view(kept).zone, Zone::Hand);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);

    game.move_object(recipient, Zone::Exile);
    game.move_object(recipient, Zone::Battlefield);
    assert_ne!(game.object_ref(recipient), exact);
    hit(&mut game, recipient, opponent, true, 2);
    assert!(game.state.stack.is_empty());
}
