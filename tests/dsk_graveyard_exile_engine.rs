//! Opponent-owned cards, former control, and exact linked replacement receipts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::{CardExtras, CardRules};
use rustmage_core::cards::*;
use rustmage_core::engine::ability_world::CharacteristicWorld;
use rustmage_core::engine::funding::{FundingGoal, FundingState};
use rustmage_core::engine::*;
use support::*;

const PLAIN: &str = "Rest in Peace";
const LINKED: &str = "Grizzly Bears";
const PLAIN_RULES: CardRules = CardRules {
    effect: Effect::Permanent,
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileOpponentGraveyardCards),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};
const LINKED_RULES: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    extras: &CardExtras {
        zone_replacement: Some(ZoneReplacementRule::ExileUncontrolledOpponentGraveyardCardsLinked),
        ..CardExtras::EMPTY
    },
    ..CardRules::EMPTY
};

fn replacement_rules() -> fixture_rules::Guard {
    fixture_rules::activate(&[
        (Card::from_name(PLAIN).unwrap(), &PLAIN_RULES),
        (Card::GrizzlyBears, &LINKED_RULES),
    ])
}

fn mill_each(game: &mut Game, controller: Player) {
    let card = put(game, controller, "Dread Summons", Zone::Hand);
    act(game, Action::AnnounceCast { card });
    act(game, Action::ChooseX { value: 1 });
    pay(game);
}

fn grant_play(game: &mut Game, card: CardId, player: Player) {
    game.grant_exile_play_permission_with_expiry(
        player,
        game.object_ref(card),
        None,
        ExilePlayExpiry::WhileExiled,
    )
    .unwrap();
}

#[test]
fn owner_gate_and_former_stack_controller_apply_to_physical_cards_from_anywhere() {
    let _rules = replacement_rules();
    for (name, controller, linked) in [(PLAIN, Player::P0, false), (LINKED, Player::P1, true)] {
        let opponent = controller.other();
        let mut game = scenario(controller);
        let source = put(&mut game, opponent, name, Zone::Battlefield);
        let provider = game.object_ref(source);
        game.change_controller(provider, controller);
        let milled = put(&mut game, opponent, "Bear Cub", Zone::Library);
        let own_milled = put(&mut game, controller, "Swamp", Zone::Library);
        mill_each(&mut game, controller);
        settle(&mut game);
        assert_eq!(game.object_view(milled).zone, Zone::Exile);
        assert_eq!(game.object_view(own_milled).zone, Zone::Graveyard);
        assert!(
            game.observe(controller)
                .objects
                .iter()
                .all(|object| !object.is_token())
        );

        let discarded = put(&mut game, opponent, "Opt", Zone::Hand);
        let duress = put(&mut game, controller, "Duress", Zone::Hand);
        cast(&mut game, duress, Some(Target::Player(opponent)));
        act(&mut game, Action::Pass);
        act(&mut game, Action::Pass);
        let object = game.object_ref(discarded);
        act(&mut game, Action::ChooseRevealedDiscard { object });
        settle(&mut game);
        assert_eq!(game.object_view(discarded).zone, Zone::Exile);
        assert_eq!(
            game.state.players[opponent.index()].cards_discarded_this_turn,
            1
        );

        let borrowed = put(&mut game, opponent, "Quick Study", Zone::Exile);
        grant_play(&mut game, borrowed, controller);
        act(&mut game, Action::AnnounceExile { card: borrowed });
        pay(&mut game);
        assert_eq!(game.object_view(borrowed).controller, controller);
        settle(&mut game);
        assert_eq!(
            game.object_view(borrowed).zone,
            if linked { Zone::Graveyard } else { Zone::Exile }
        );

        let spell = put(&mut game, opponent, "Shock", Zone::Hand);
        let counter = put(&mut game, controller, "Negate", Zone::Hand);
        act(&mut game, Action::Pass);
        cast(&mut game, spell, Some(Target::Player(controller)));
        if game.actor() != Some(controller) {
            act(&mut game, Action::Pass);
        }
        cast(&mut game, counter, Some(Target::Spell(spell)));
        settle(&mut game);
        assert_eq!(game.object_view(spell).zone, Zone::Exile);
        assert_eq!(game.object_view(counter).zone, Zone::Graveyard);
        assert_eq!(game.state.players[controller.index()].life, 20);

        let receipts = game.linked_exile_for_origin(provider, None);
        let expected = if linked {
            [milled, discarded, spell]
                .map(|id| game.object_ref(id))
                .to_vec()
        } else {
            Vec::new()
        };
        assert_eq!(receipts, expected);
    }
}

#[test]
fn a_simultaneous_departure_freezes_the_provider_and_each_former_controller() {
    let _rules = replacement_rules();
    let controller = Player::P0;
    let opponent = controller.other();
    for (name, linked) in [(PLAIN, false), (LINKED, true)] {
        let mut game = scenario(controller);
        let source = put(&mut game, controller, name, Zone::Battlefield);
        let provider = game.object_ref(source);
        let other_card = put(&mut game, opponent, "Bear Cub", Zone::Battlefield);
        let borrowed = put(&mut game, opponent, "Savannah Lions", Zone::Battlefield);
        let own_stolen = put(&mut game, controller, "Hill Giant", Zone::Battlefield);
        game.change_controller(game.object_ref(borrowed), controller);
        game.change_controller(game.object_ref(own_stolen), opponent);
        let token = game.state.objects.len() as CardId;
        game.create_tokens(opponent, Card::from_catalogue_id(5258), 1, false);
        let borrowed_destination = if linked { Zone::Graveyard } else { Zone::Exile };
        let objects =
            [source, other_card, borrowed, own_stolen, token].map(|id| game.object_ref(id));
        let before = game.snapshot();
        let goal = FundingGoal {
            actor: controller,
            required: [0; 6],
            generic: 0,
            reserve: None,
            source: None,
        };
        let world = FundingState::from_game(&game, &goal);
        assert!(
            world.cost_departures(&objects, Zone::Graveyard, &mut |next| {
                assert_eq!(next.object(source).zone, Zone::Graveyard);
                assert_eq!(next.object(other_card).zone, Zone::Exile);
                assert_eq!(next.object(borrowed).zone, borrowed_destination);
                assert_eq!(next.object(own_stolen).zone, Zone::Graveyard);
                assert_eq!(next.object(token).zone, Zone::Graveyard);
                assert_eq!(next.life(controller), 20);
                true
            })
        );
        assert_eq!(game.snapshot(), before);

        let wrath = put(&mut game, controller, "Day of Judgment", Zone::Hand);
        cast(&mut game, wrath, None);
        settle(&mut game);
        assert_eq!(
            game.object_view(source).zone,
            if linked {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        assert_eq!(game.object_view(other_card).zone, Zone::Exile);
        assert_eq!(game.object_view(borrowed).zone, borrowed_destination);
        assert_eq!(game.object_view(own_stolen).zone, Zone::Graveyard);
        assert_eq!(game.object_view(token).zone, Zone::Outside);
        assert_eq!(
            game.state.players[controller.index()].life_gain_events_this_turn,
            0
        );
        if linked {
            let exact = game.object_ref(other_card);
            assert_eq!(game.linked_exile_for_origin(provider, None), [exact]);
            let return_spell = put(&mut game, controller, "Zombify", Zone::Hand);
            cast(&mut game, return_spell, Some(Target::Card(source)));
            settle(&mut game);
            let returned = game.object_ref(source);
            assert_ne!(returned, provider);
            assert_eq!(game.object_view(source).zone, Zone::Battlefield);
            assert!(game.linked_exile_for_origin(returned, None).is_empty());
            assert_eq!(game.linked_exile_for_origin(provider, None), [exact]);
        } else {
            assert!(game.linked_exile_for_origin(provider, None).is_empty());
        }
    }
}

#[test]
fn competing_replacements_keep_a_milled_face_private_until_the_selected_exile() {
    let _rules = replacement_rules();
    let controller = Player::P1;
    let opponent = controller.other();
    let mut parent = scenario(controller);
    put(&mut parent, controller, PLAIN, Zone::Battlefield);
    let source = put(&mut parent, controller, LINKED, Zone::Battlefield);
    let provider = parent.object_ref(source);
    let card = put(&mut parent, opponent, "Bear Cub", Zone::Library);
    let original = parent.object_ref(card);
    mill_each(&mut parent, controller);
    let mut already_known = parent.clone();
    already_known.remember_library_top(opponent, original);
    for game in [&mut parent, &mut already_known] {
        act(game, Action::Pass);
        act(game, Action::Pass);
    }
    assert_eq!(
        already_known
            .observe(opponent)
            .zone_replacement_choice
            .unwrap()
            .card,
        Some(already_known.object_view(card).card)
    );

    let linked = ZoneReplacementKind::ExileLinked { source: provider };
    let plain = ZoneReplacementKind::Exile;
    assert_eq!(parent.actor(), Some(opponent));
    assert_eq!(parent.object_view(card).zone, Zone::Library);
    assert!(parent.observe(controller).zone_replacement_choice.is_none());
    let view = parent.observe(opponent).zone_replacement_choice.unwrap();
    assert_eq!(view.from, Zone::Library);
    assert_eq!(view.card, None);
    assert_eq!(view.choices, [plain, linked]);
    assert!(serde_json::to_value(view).unwrap().get("card").is_none());
    let link_action = Action::ChooseZoneReplacement { effect: linked };
    let index = parent
        .legal_actions()
        .iter()
        .position(|action| *action == link_action)
        .unwrap();
    let revision = parent.revision();
    let saved = parent.snapshot();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(
        raw.apply(controller, link_action.clone()),
        Err(Error::WrongActor)
    );
    assert_eq!(raw.snapshot(), saved);

    act(&mut raw, link_action);
    indexed.apply_index(opponent, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    let exact = raw.object_ref(card);
    assert_ne!(exact, original);
    assert_eq!(raw.object_view(card).zone, Zone::Exile);
    assert_eq!(raw.linked_exile_for_origin(provider, None), [exact]);
    assert!(!raw.exiled_face_down(exact));

    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(opponent, revision, index),
        Err(Error::StaleRevision)
    );
    act(
        &mut indexed,
        Action::ChooseZoneReplacement { effect: plain },
    );
    settle(&mut indexed);
    assert_eq!(indexed.object_view(card).zone, Zone::Exile);
    assert!(indexed.linked_exile_for_origin(provider, None).is_empty());
    assert_eq!(raw.linked_exile_for_origin(provider, None), [exact]);

    raw.move_object(card, Zone::Hand);
    raw.move_object(card, Zone::Exile);
    assert_ne!(raw.object_ref(card), exact);
    assert!(raw.linked_exile_for_origin(provider, None).is_empty());
}
