//! Printed attachment grants, death receipts, and exact Crew contributors.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn candidate(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn activate(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn equip(game: &mut Game, gear: CardId, bearer: CardId, generic: u32) {
    act(game, activate(gear, 0));
    act(game, target(bearer));
    let actor = game.actor().unwrap();
    assert_eq!(
        game.observe(actor).payment_request.unwrap().generic,
        generic
    );
    pay(game);
    settle(game);
    assert_eq!(
        game.object_view(gear).attached_to,
        Some(game.object_ref(bearer))
    );
}

fn attack(game: &mut Game, attacker: CardId) {
    game.state.step = Step::Attackers;
    game.state.pending = Pending::Attackers;
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
}

fn advance(game: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..20 {
        if ready(game) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("scenario did not reach its next decision");
}

#[test]
fn sunfire_torch_pays_its_own_grantor_before_the_bearer_can_target() {
    let p = Player::P0;
    let mut root = scenario(p);
    let torch = candidate(&mut root, p, "Sunfire Torch", Zone::Battlefield);
    let spare = candidate(&mut root, p, "Sunfire Torch", Zone::Battlefield);
    let attacker = put(&mut root, p, "Healer's Hawk", Zone::Battlefield);
    equip(&mut root, torch, attacker, 1);
    assert_eq!(stats(&root, attacker), (2, 1));
    attack(&mut root, attacker);
    assert!(root.state.stack.last().unwrap().targets.is_empty());
    let grantor = root.object_ref(torch);
    let mut stolen = root.clone();
    stolen.record_control(grantor, p.other(), None);
    settle(&mut stolen);
    assert_eq!(stolen.object_view(torch).zone, Zone::Battlefield);
    assert_eq!(stolen.state.players[p.other().index()].life, STARTING_LIFE);

    advance(&mut root, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    let choice = Action::Sacrifice { object: grantor };
    let wrong = Action::Sacrifice {
        object: root.object_ref(spare),
    };
    reject(&mut root, wrong);
    assert!(
        !root
            .legal_actions()
            .iter()
            .any(|action| matches!(action, Action::ChooseTarget { .. }))
    );
    let mut declined = root.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    settle(&mut declined);
    assert_eq!(declined.object_view(torch).zone, Zone::Battlefield);
    assert_eq!(
        declined.state.players[p.other().index()].life,
        STARTING_LIFE
    );

    act(&mut root, choice);
    assert_eq!(root.object_view(torch).zone, Zone::Graveyard);
    assert_eq!(root.object_view(spare).zone, Zone::Battlefield);
    act(
        &mut root,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    assert_eq!(
        root.state.stack.last().unwrap().source.unwrap().object,
        root.object_ref(attacker)
    );
    root.move_object(attacker, Zone::Exile);
    settle(&mut root);
    assert_eq!(
        root.state.players[p.other().index()].life,
        STARTING_LIFE - 2
    );
    assert_eq!(root.state.players[p.index()].life, STARTING_LIFE + 2);
}

#[test]
fn swashbucklers_whip_grants_reach_and_two_abilities_with_the_bearers_tap_cost() {
    let p = Player::P1;
    let mut game = scenario(p);
    let whip = candidate(&mut game, p, "Swashbuckler's Whip", Zone::Battlefield);
    let bearer = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, p.other(), "Swiftfoot Boots", Zone::Battlefield);
    let enchantment = put(&mut game, p.other(), "Impact Tremors", Zone::Battlefield);
    let eligible = candidate(&mut game, p, "Omniscience", Zone::Library);
    let too_large = put(&mut game, p, "Ghalta, Primal Hunger", Zone::Library);
    let grantor = game.object_ref(whip);
    let tap = Action::AnnounceAttachedAbility {
        grantor,
        ability: 1,
    };
    let discover = Action::AnnounceAttachedAbility {
        grantor,
        ability: 2,
    };
    reject(&mut game, tap.clone());
    equip(&mut game, whip, bearer, 1);
    assert_eq!(stats(&game, bearer), (2, 2));
    assert!(game.object_view(bearer).keywords.contains(Keywords::REACH));
    assert!(!game.object_view(whip).keywords.contains(Keywords::REACH));
    reject(&mut game, activate(whip, 1));
    game.state.objects[bearer as usize].summoning_sick = true;
    reject(&mut game, tap.clone());
    reject(&mut game, discover.clone());
    game.state.objects[bearer as usize].summoning_sick = false;
    act(&mut game, tap);
    reject(&mut game, target(enchantment));
    act(&mut game, target(artifact));
    assert_eq!(game.observe(p).payment_request.unwrap().generic, 2);
    pay(&mut game);
    assert!(game.object_view(bearer).tapped);
    assert!(!game.object_view(whip).tapped);
    settle(&mut game);
    assert!(game.object_view(artifact).tapped);
    reject(&mut game, discover.clone());
    game.untap_objects(&[bearer], untap_rules::UntapCause::Effect);
    act(&mut game, discover);
    assert_eq!(game.observe(p).payment_request.unwrap().generic, 8);
    pay(&mut game);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::ResolutionCasts { .. })
    });
    assert_eq!(game.object_view(eligible).zone, Zone::Exile);
    assert_eq!(game.object_view(too_large).zone, Zone::Exile);
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(eligible).zone, Zone::Hand);
    assert_eq!(game.object_view(too_large).zone, Zone::Library);
    assert!(game.object_view(bearer).tapped);
    assert!(!game.object_view(whip).tapped);
}

#[test]
fn tarrians_soulcleaver_uses_each_qualifying_departure_and_the_current_bearer() {
    let p = Player::P0;
    let mut root = scenario(p);
    let gear = candidate(&mut root, p, "Tarrian's Soulcleaver", Zone::Battlefield);
    let first = put(&mut root, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut root, p, "Grizzly Bears", Zone::Battlefield);
    let artifact = put(&mut root, p, "Swiftfoot Boots", Zone::Battlefield);
    let creature = put(&mut root, p.other(), "Hill Giant", Zone::Battlefield);
    let both = put(
        &mut root,
        p.other(),
        "Cartographer's Companion",
        Zone::Battlefield,
    );
    let land = put(&mut root, p.other(), "Forest", Zone::Battlefield);
    equip(&mut root, gear, first, 2);
    assert!(
        root.object_view(first)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert!(
        !root
            .object_view(second)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    let mut itself = root.clone();
    itself.move_object(gear, Zone::Graveyard);
    itself.prepare_triggers();
    assert!(itself.state.pending_triggers.is_empty() && itself.state.stack.is_empty());

    let deaths = [artifact, creature, both, land].map(|card| root.object_ref(card));
    root.destroy_objects_outcome(deaths);
    root.prepare_triggers();
    let mut detached = root.clone();
    detached.move_object(first, Zone::Exile);
    advance(&mut detached, |game| {
        game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.pending == Pending::Priority
    });
    assert_eq!(detached.counter_count(second, CounterKind::PlusOne), 0);

    let source = root.object_ref(gear);
    let recipient = root.object_ref(second);
    assert!(root.attach_equipment(source, recipient));
    assert!(
        root.object_view(second)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    assert!(
        !root
            .object_view(first)
            .keywords
            .contains(Keywords::VIGILANCE)
    );
    advance(&mut root, |game| {
        game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.pending == Pending::Priority
    });
    assert_eq!(root.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(root.counter_count(second, CounterKind::PlusOne), 3);
    attack(&mut root, second);
    assert!(!root.object_view(second).tapped);
}

fn crew(game: &mut Game, vehicle: CardId, contributors: &[CardId]) {
    act(game, activate(vehicle, 0));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    for &card in contributors {
        let object = game.object_ref(card);
        act(game, Action::ChooseCrewer { object });
    }
    act(game, Action::FinishCrew);
    settle(game);
}

#[test]
fn subterranean_schooner_targets_only_a_creature_that_crewed_its_exact_incarnation() {
    let p = Player::P1;
    let mut game = scenario(p);
    let schooner = candidate(&mut game, p, "Subterranean Schooner", Zone::Battlefield);
    let cart = put(&mut game, p, "Careening Mine Cart", Zone::Battlefield);
    let first = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let second = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let wrong_crew = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let nonland = put(&mut game, p, "Shock", Zone::Library);
    crew(&mut game, cart, &[wrong_crew]);
    game.state.objects[first as usize].summoning_sick = true;
    crew(&mut game, schooner, &[first, second]);
    assert_eq!(stats(&game, schooner), (3, 4));
    assert_eq!(game.object_view(schooner).kinds, 17);
    assert!(game.object_view(first).tapped && game.object_view(second).tapped);
    attack(&mut game, schooner);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Targets { .. })
    });
    assert!(game.legal_actions().contains(&target(first)));
    reject(&mut game, target(wrong_crew));
    game.move_object(first, Zone::Exile);
    game.move_object(first, Zone::Battlefield);
    reject(&mut game, target(first));
    act(&mut game, target(second));
    advance(&mut game, |game| game.observe(p).explore_choice.is_some());
    let object = game.object_ref(nonland);
    act(
        &mut game,
        Action::ChooseExplore {
            object,
            graveyard: true,
        },
    );
    settle(&mut game);
    assert_eq!(game.counter_count(second, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(first, CounterKind::PlusOne), 0);
    assert_eq!(game.object_view(nonland).zone, Zone::Graveyard);
}
