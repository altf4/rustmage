//! Rules scenarios for the authored Final Fantasy definitions in rows 235–312.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn trigger_target(game: &mut Game, target: Target) {
    let action = Action::ChooseTarget { target };
    for _ in 0..16 {
        if game.legal_actions().contains(&action) {
            act(game, action);
            if game.legal_actions().contains(&Action::FinishTargets) {
                act(game, Action::FinishTargets);
            }
            return;
        }
        choose(game, |choice| {
            matches!(choice, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("missing trigger target {target:?}");
}

#[test]
fn swallowed_by_leviathan_surveille_then_prices_the_exact_spell() {
    let caster = Player::P1;
    let payer = caster.other();
    let mut game = scenario(payer);
    let victim = put(&mut game, payer, "Opt", Zone::Hand);
    let counter = put(&mut game, caster, "Swallowed by Leviathan", Zone::Hand);
    let buried = put(&mut game, caster, "Island", Zone::Library);
    let kept = put(&mut game, caster, "Forest", Zone::Library);
    put(&mut game, caster, "Grizzly Bears", Zone::Graveyard);
    cast(&mut game, victim, None);
    let id = game.state.stack.last().unwrap().id;
    act(&mut game, Action::Pass);
    cast(&mut game, counter, Some(Target::Stack(id)));
    pass_pair(&mut game);
    assert!(game.observe(caster).library_choice.is_some());
    assert!(game.observe(payer).library_choice.is_none());
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: buried,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut game,
        Action::ArrangeLibrary {
            card: kept,
            destination: LibraryDestination::Top,
        },
    );
    let request = game.observe(payer).resolution_payment.unwrap();
    assert_eq!(request.cost.generic, 2);
    assert_eq!(request.victim, Some(id));
    assert_eq!(request.payer, payer);
    let mut decline = game.clone();
    act(&mut decline, Action::DeclineResolutionMana);
    assert_eq!(decline.object_view(victim).zone, Zone::Graveyard);
    choose(&mut game, |choice| {
        matches!(choice, Action::PayResolutionMana { .. })
    });
    assert_eq!(game.object_view(victim).zone, Zone::Stack);
    assert_eq!(game.object_view(counter).zone, Zone::Graveyard);
}

#[test]
fn tifa_doubles_only_signed_power_from_her_own_landfall() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tifa = put(&mut game, player, "Tifa Lockhart", Zone::Hand);
    cast(&mut game, tifa, None);
    settle(&mut game);
    assert!(game.keywords(tifa).contains(Keywords::TRAMPLE));
    assert_eq!(stats(&game, tifa), (1, 2));
    game.state.objects[tifa as usize].growth.power = -3;
    let their_land = put(&mut game, player.other(), "Forest", Zone::Hand);
    game.move_object(their_land, Zone::Battlefield);
    assert!(game.state.pending_triggers.is_empty());
    let land = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(stats(&game, tifa), (-4, 2));
    game.finish_cleanup();
    assert_eq!(stats(&game, tifa), (1, 2));
}

#[test]
fn tonberry_enters_stunned_and_has_the_knife_on_its_controllers_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let tonberry = put(&mut game, player, "Tonberry", Zone::Hand);
    cast(&mut game, tonberry, None);
    settle(&mut game);
    assert_eq!(stats(&game, tonberry), (2, 1));
    assert!(game.object_view(tonberry).tapped);
    assert_eq!(game.counter_count(tonberry, CounterKind::Stun), 1);
    let knife = Keywords::FIRST_STRIKE.union(Keywords::DEATHTOUCH);
    assert!(game.keywords(tonberry).contains(knife));
    game.state.active = player.other();
    assert!(!game.keywords(tonberry).contains(Keywords::FIRST_STRIKE));
    assert!(!game.keywords(tonberry).contains(Keywords::DEATHTOUCH));
    game.untap_objects(&[tonberry], untap_rules::UntapCause::Effect);
    assert_eq!(game.counter_count(tonberry, CounterKind::Stun), 0);
    assert!(game.object_view(tonberry).tapped);
    game.untap_objects(&[tonberry], untap_rules::UntapCause::Effect);
    assert!(!game.object_view(tonberry).tapped);
}

#[test]
fn town_greeter_offers_only_milled_lands_and_rewards_an_actual_town_return() {
    let player = Player::P1;
    let mut game = scenario(player);
    let old = put(
        &mut game,
        player,
        "Vector, Imperial Capital",
        Zone::Graveyard,
    );
    let filler = put(&mut game, player, "Shock", Zone::Library);
    let forest = put(&mut game, player, "Forest", Zone::Library);
    let town = put(&mut game, player, "Vector, Imperial Capital", Zone::Library);
    put(&mut game, player, "Swiftfoot Boots", Zone::Library);
    let greeter = put(&mut game, player, "Town Greeter", Zone::Hand);
    cast(&mut game, greeter, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.pending, Pending::MilledSelection { player });
    assert_eq!(game.object_view(filler).zone, Zone::Graveyard);
    let invalid = game.object_ref(old);
    reject(&mut game, Action::SelectMilled { object: invalid });
    let mut basic = game.clone();
    let object = basic.object_ref(forest);
    act(&mut basic, Action::SelectMilled { object });
    assert_eq!(basic.object_view(forest).zone, Zone::Hand);
    assert_eq!(basic.state.players[player.index()].life, 20);
    let object = game.object_ref(town);
    act(&mut game, Action::SelectMilled { object });
    assert_eq!(game.object_view(town).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, 22);
}

#[test]
fn the_dire_rat_creates_treasure_after_lethal_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let rat = put(&mut game, player, "Undercity Dire Rat", Zone::Hand);
    cast(&mut game, rat, None);
    settle(&mut game);
    assert_eq!(stats(&game, rat), (2, 2));
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(rat)));
    settle(&mut game);
    assert_eq!(game.object_view(rat).zone, Zone::Graveyard);
    assert_eq!(
        game.state
            .objects
            .iter()
            .filter(|object| {
                object.zone == Zone::Battlefield
                    && object.controller == player
                    && object.card == Card::from_catalogue_id(5302)
            })
            .count(),
        1
    );
}

#[test]
fn vaynes_treachery_checks_the_kicker_sacrifice_and_uses_the_alternate_amount() {
    let player = Player::P0;
    for kicked in [false, true] {
        let mut game = scenario(player);
        let target = put(
            &mut game,
            player.other(),
            "Ghalta, Primal Hunger",
            Zone::Battlefield,
        );
        let artifact = put(&mut game, player, "Short Bow", Zone::Battlefield);
        let land = put(&mut game, player, "Forest", Zone::Battlefield);
        let spell = put(&mut game, player, "Vayne's Treachery", Zone::Hand);
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseKicker { pay: kicked });
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if kicked {
            let invalid = game.object_ref(land);
            reject(&mut game, Action::ChooseAdditionalCost { object: invalid });
            let object = game.object_ref(artifact);
            act(&mut game, Action::ChooseAdditionalCost { object });
        }
        pay(&mut game);
        settle(&mut game);
        assert_eq!(stats(&game, target), if kicked { (6, 6) } else { (10, 10) });
        assert_eq!(
            game.object_view(artifact).zone,
            if kicked {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
    }
}

#[test]
fn yshtola_returns_to_the_owner_and_adds_only_the_first_extra_end_step() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let source = put(&mut game, player, "Y'shtola Rhul", Zone::Battlefield);
    let stolen = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[stolen as usize].controller = player;
    let exact = game.object_ref(stolen);
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    trigger_target(&mut game, Target::Card(stolen));
    settle(&mut game);
    assert_ne!(game.object_ref(stolen), exact);
    assert_eq!(game.object_view(stolen).controller, opponent);
    assert_eq!(
        game.observe(player).additional_end_steps.unwrap().pending,
        1
    );
    pass_pair(&mut game);
    trigger_target(&mut game, Target::Card(source));
    settle(&mut game);
    let extra = game.observe(player).additional_end_steps.unwrap();
    assert!(extra.current_is_additional);
    assert_eq!(extra.pending, 0);
}

#[test]
fn zell_grants_an_extra_land_counts_live_lands_and_returns_one_to_its_owner() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Zell Dincht", Zone::Hand);
    cast(&mut game, source, None);
    settle(&mut game);
    let first = put(&mut game, player, "Forest", Zone::Hand);
    let second = put(&mut game, player, "Forest", Zone::Hand);
    let third = put(&mut game, player, "Forest", Zone::Hand);
    act(&mut game, Action::PlayLand { card: first });
    act(&mut game, Action::PlayLand { card: second });
    reject(&mut game, Action::PlayLand { card: third });
    assert_eq!(stats(&game, source), (2, 3));

    game.state.step = Step::Main2;
    pass_pair(&mut game);
    pass_pair(&mut game);
    let object = game.object_ref(first);
    act(&mut game, Action::ChooseEffectObject { object });
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(stats(&game, source), (1, 3));
}

const SYNTHETIC_FACTORIES: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5531),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5532),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5533),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5534),
                amount: 1,
                tapped: false,
            },
            Instruction::CreateEmblem(Card::from_catalogue_id(5535)),
        ],
    ),
    ..CardRules::EMPTY
};

#[test]
fn the_pinned_tokens_and_emblem_supply_their_printed_characteristics_and_triggers() {
    let _rules =
        fixture_rules::activate(&[(Card::from_name("Opt").unwrap(), &SYNTHETIC_FACTORIES)]);
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    for (index, name, subtype, color, kinds, legendary, expected) in [
        (5531, "Wizard Token", "Wizard", 4, 1, false, (0, 1)),
        (5532, "Robot Token", "Warrior", 2, 17, false, (3, 3)),
        (5533, "Angelo", "Dog", 17, 1, true, (1, 1)),
        (5534, "Darkstar", "Dog", 5, 1, true, (2, 2)),
    ] {
        let token = game
            .state
            .objects
            .iter()
            .position(|object| {
                object.zone == Zone::Battlefield && object.card == Card::from_catalogue_id(index)
            })
            .unwrap() as CardId;
        let view = game.object_view(token);
        assert!(view.is_token() && view.has_subtype(subtype));
        assert_eq!((view.name, view.colors, view.kinds), (name, color, kinds));
        assert_eq!(view.card.is_legendary(), legendary);
        assert_eq!(stats(&game, token), expected);
    }
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(victim)));
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    pass_pair(&mut game);
    trigger_target(&mut game, Target::Player(opponent));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[opponent.index()].life, 18);
    assert_eq!(game.state.players[player.index()].life, 21);
}
