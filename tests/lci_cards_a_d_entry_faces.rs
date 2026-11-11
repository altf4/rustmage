//! The printed entry, transformation, exile permission and ward abilities.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

const BAT: Card = Card::from_catalogue_id(5276);
const GNOME: Card = Card::from_catalogue_id(5267);
const DINOSAUR: Card = Card::from_catalogue_id(5570);
const TREASURE: Card = Card::from_catalogue_id(5302);

fn candidate(game: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(game, owner, Card::from_name(name).unwrap(), zone)
}

fn passes(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
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

fn step(game: &mut Game, next: Step) {
    game.begin_step(next);
    game.prepare_triggers();
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

fn tokens(game: &Game, player: Player, card: Card) -> Vec<ObjectView> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == player
                && object.card == card
                && object.is_token()
        })
        .collect()
}

#[test]
fn awakening_keeps_its_standard_ban_and_adds_a_non_copiable_spirit_entry_form() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = candidate(&mut game, player, "Abuelo's Awakening", Zone::Hand);
    assert!(!game.object_view(spell).card.is_standard_legal());
    let equipment = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let enchantment = put(&mut game, player, "Impact Tremors", Zone::Graveyard);
    let aura = put(&mut game, player, "Pacifism", Zone::Graveyard);
    let foreign = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Graveyard,
    );
    put(
        &mut game,
        player.other(),
        "Authority of the Consuls",
        Zone::Battlefield,
    );
    put(&mut game, player, "Doubling Season", Zone::Battlefield);
    let initial = game.snapshot();
    for (value, permanent, kinds) in [(2, equipment, 17), (0, enchantment, 33)] {
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseX { value });
        reject(&mut game, target(aura));
        reject(&mut game, target(foreign));
        act(&mut game, target(permanent));
        pay(&mut game);
        settle(&mut game);
        let entered = game.object_view(permanent);
        assert_eq!(entered.zone, Zone::Battlefield);
        assert!(entered.tapped);
        assert_eq!(entered.kinds, kinds);
        assert!(entered.has_subtype("Spirit"));
        assert!(entered.keywords.contains(Keywords::FLYING));
        assert_eq!(
            game.counter_count(permanent, CounterKind::PlusOne),
            u64::from(value) * 2
        );
        let size = i128::from(value) * 2 + 1;
        assert_eq!(stats(&game, permanent), (size, size));
        // The creature-enchantment also sees its own entry and deals 1 damage.
        assert_eq!(
            game.state.players[player.other().index()].life,
            STARTING_LIFE + i128::from(permanent == equipment)
        );
        assert_eq!(game.copiable_recipe(entered.reference).kinds(), kinds ^ 1);
        game.move_object(permanent, Zone::Exile);
        game.move_object(permanent, Zone::Battlefield);
        assert!(!game.object_view(permanent).has_subtype("Spirit"));
        game.restore(&initial).unwrap();
    }
}

#[test]
fn abuelo_wards_itself_and_returns_another_controlled_artifact_under_its_owner() {
    let player = Player::P0;
    let foe = player.other();
    let mut game = scenario(player);
    let abuelo = candidate(
        &mut game,
        player,
        "Abuelo, Ancestral Echo",
        Zone::Battlefield,
    );
    assert!(game.object_view(abuelo).keywords.contains(Keywords::FLYING));
    let gear = put(&mut game, foe, "Swiftfoot Boots", Zone::Battlefield);
    let gear_ref = game.object_ref(gear);
    game.record_control(gear_ref, player, None);
    let enemy = put(&mut game, foe, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut game, foe, "Shock", Zone::Hand);
    let initial = game.snapshot();
    for paid in [false, true] {
        act(&mut game, Action::Pass);
        cast(&mut game, shock, Some(Target::Card(abuelo)));
        advance(&mut game, |game| {
            game.legal_actions()
                .contains(&Action::DeclineResolutionMana)
        });
        assert_eq!(game.actor(), Some(foe));
        if paid {
            choose(&mut game, |action| {
                matches!(action, Action::PayResolutionMana { .. })
            });
        } else {
            act(&mut game, Action::DeclineResolutionMana);
        }
        settle(&mut game);
        assert_eq!(
            game.object_view(abuelo).zone,
            if paid {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        game.restore(&initial).unwrap();
    }
    act(&mut game, activate(abuelo, 0));
    reject(&mut game, target(abuelo));
    reject(&mut game, target(enemy));
    act(&mut game, target(gear));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(gear).zone, Zone::Exile);
    game.move_object(abuelo, Zone::Graveyard);
    step(&mut game, Step::End);
    settle(&mut game);
    let returned = game.object_view(gear);
    assert_eq!(returned.zone, Zone::Battlefield);
    assert_eq!(returned.controller, foe);
    assert_ne!(returned.reference, gear_ref);
}

#[test]
fn aclazotz_checks_the_discard_receipt_and_returns_as_its_owners_tapped_temple() {
    let player = Player::P0;
    let foe = player.other();
    let mut root = scenario(player);
    let god = candidate(
        &mut root,
        player,
        "Aclazotz, Deepest Betrayal // Temple of the Dead",
        Zone::Battlefield,
    );
    assert!(
        root.object_view(god)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
    );
    for discard in [None, Some("Forest"), Some("Shock")] {
        let mut game = root.clone();
        let discarded = discard.map(|name| put(&mut game, foe, name, Zone::Hand));
        game.state.step = Step::Attackers;
        game.state.pending = Pending::Attackers;
        act(&mut game, Action::Attack { card: god });
        act(&mut game, Action::FinishAttackers);
        if let Some(card) = discarded {
            advance(&mut game, |game| {
                matches!(game.state.pending, Pending::EffectDiscard { .. })
            });
            assert_eq!(game.actor(), Some(foe));
            act(&mut game, Action::Discard { card });
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            u64::from(discard.is_none())
        );
        let bats = tokens(&game, player, BAT);
        assert_eq!(bats.len(), usize::from(discard == Some("Forest")));
        if let Some(bat) = bats.first() {
            assert_eq!((bat.power, bat.toughness), (1, 1));
            assert!(bat.keywords.contains(Keywords::FLYING));
        }
    }

    let source = root.object_ref(god);
    root.record_control(source, foe, None);
    root.move_object(god, Zone::Graveyard);
    root.prepare_triggers();
    let mut departed = root.clone();
    departed.move_object(god, Zone::Exile);
    settle(&mut departed);
    assert_eq!(departed.object_view(god).zone, Zone::Exile);
    settle(&mut root);
    let temple = root.object_view(god);
    assert_eq!(temple.card.name(), "Temple of the Dead");
    assert_eq!(temple.controller, player);
    assert!(temple.tapped);
    reject(&mut root, activate(god, 1));
    root.untap_objects(&[god], untap_rules::UntapCause::Effect);
    let mana_before = root.state.players[player.index()].mana[2];
    let mut mana = root.clone();
    choose(
        &mut mana,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 1, 0, 0, 0], .. } if *card == god),
    );
    assert_eq!(mana.state.players[player.index()].mana[2], mana_before + 1);
    for owner in [player, foe] {
        for _ in 0..2 {
            put(&mut root, owner, "Forest", Zone::Hand);
        }
    }
    reject(&mut root, activate(god, 1));
    let hand = root
        .state
        .objects
        .iter()
        .enumerate()
        .find(|(_, object)| object.owner == foe && object.zone == Zone::Hand)
        .unwrap()
        .0 as CardId;
    root.move_object(hand, Zone::Graveyard);
    act(&mut root, activate(god, 1));
    pay(&mut root);
    put(&mut root, foe, "Forest", Zone::Hand);
    settle(&mut root);
    let returned = root.object_view(god);
    assert_eq!(returned.reference, temple.reference);
    assert_eq!(returned.kinds, 1);
    assert!(returned.tapped);
}

#[test]
fn tunnel_grinder_uses_actual_discards_and_tecutlan_mana_pays_for_a_permanent() {
    let player = Player::P0;
    let mut game = scenario(player);
    let grinder = candidate(
        &mut game,
        player,
        "Brass's Tunnel-Grinder // Tecutlan, the Searing Rift",
        Zone::Hand,
    );
    let bear = put(&mut game, player, "Bear Cub", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, grinder, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectChoice { .. })
    });
    let choose_discard = game.snapshot();
    for discarded in [&[][..], &[bear][..], &[bear, shock][..]] {
        for &card in discarded {
            act(&mut game, Action::Discard { card });
        }
        act(&mut game, Action::DeclineEffectPayment);
        settle(&mut game);
        assert_eq!(
            game.state.players[player.index()].cards_drawn_this_turn,
            discarded.len() as u64 + 1
        );
        for &card in discarded {
            assert_eq!(game.object_view(card).zone, Zone::Graveyard);
        }
        game.restore(&choose_discard).unwrap();
    }
    act(&mut game, Action::Discard { card: bear });
    act(&mut game, Action::DeclineEffectPayment);
    settle(&mut game);
    game.add_counter(grinder, CounterKind::Bore, 4);
    step(&mut game, Step::End);
    settle(&mut game);
    let rift = game.object_view(grinder);
    assert_eq!(rift.card.name(), "Tecutlan, the Searing Rift");
    assert_eq!(game.counter_count(grinder, CounterKind::Bore), 0);
    assert!(rift.has_subtype("Cave"));
    step(&mut game, Step::Main1);
    game.state.players[player.index()].mana = [0, 0, 0, 0, 0, 1];
    let top = put(&mut game, player, "Candy Trail", Zone::Library);
    let permanent = put(&mut game, player, "Goblin Piker", Zone::Hand);
    choose(
        &mut game,
        |action| matches!(action, Action::ActivateMana { card, production: [0, 0, 0, 1, 0, 0], .. } if *card == grinder),
    );
    cast(&mut game, permanent, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::ResolutionCasts { .. })
    });
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    assert_eq!(game.object_view(permanent).zone, Zone::Stack);
    act(&mut game, Action::FinishResolutionCasts);
    settle(&mut game);
    assert_eq!(game.object_view(top).zone, Zone::Hand);
    assert_eq!(game.object_view(permanent).zone, Zone::Battlefield);
}

#[test]
fn clay_fired_bricks_finds_a_basic_plains_and_crafts_a_kiln_that_boosts_its_gnomes() {
    let player = Player::P0;
    let mut game = scenario(player);
    let bricks = candidate(
        &mut game,
        player,
        "Clay-Fired Bricks // Cosmium Kiln",
        Zone::Hand,
    );
    let plains = put(&mut game, player, "Plains", Zone::Library);
    let wrong_basic = put(&mut game, player, "Island", Zone::Library);
    let nonbasic = candidate(&mut game, player, "Temple Garden", Zone::Library);
    let material = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let wrong = put(&mut game, player, "Bear Cub", Zone::Graveyard);
    let ally = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, bricks, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::EffectSearch { .. })
    });
    reject(&mut game, Action::ChooseSearch { card: wrong_basic });
    reject(&mut game, Action::ChooseSearch { card: nonbasic });
    let before = game.snapshot();
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 2);
    assert_eq!(game.object_view(plains).zone, Zone::Library);
    game.restore(&before).unwrap();
    act(&mut game, Action::ChooseSearch { card: plains });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(plains).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].life, STARTING_LIFE + 2);

    act(&mut game, activate(bricks, 0));
    act(&mut game, Action::BeginOrderedCosts);
    let additional = game
        .observe(player)
        .cast_costs
        .unwrap()
        .clauses
        .iter()
        .position(|row| row.clause == CastCostClause::Additional)
        .unwrap() as u16;
    act(&mut game, Action::ChooseCastCost { clause: additional });
    let invalid = Action::ChooseAdditionalCost {
        object: game.object_ref(wrong),
    };
    reject(&mut game, invalid);
    let choice = Action::ChooseAdditionalCost {
        object: game.object_ref(material),
    };
    act(&mut game, choice);
    while game.state.announcing.is_some() {
        match game.state.pending {
            Pending::CastCosts { .. } => choose(&mut game, |action| {
                matches!(action, Action::ChooseCastCost { .. })
            }),
            Pending::PayCosts { .. } => pay(&mut game),
            _ => panic!("unexpected Craft cost"),
        }
    }
    assert_eq!(game.object_view(material).zone, Zone::Exile);
    assert_eq!(game.object_view(bricks).zone, Zone::Exile);
    settle(&mut game);
    assert_eq!(game.object_view(bricks).card.name(), "Cosmium Kiln");
    let gnomes = tokens(&game, player, GNOME);
    assert_eq!(gnomes.len(), 2);
    assert!(
        gnomes
            .iter()
            .all(|gnome| gnome.kinds == 17 && (gnome.power, gnome.toughness) == (2, 2))
    );
    assert_eq!(stats(&game, ally), (3, 3));
    assert_eq!(stats(&game, enemy), (2, 2));
    game.move_object(bricks, Zone::Graveyard);
    assert_eq!(stats(&game, ally), (2, 2));
    assert_eq!(stats(&game, gnomes[0].id), (1, 1));
}

#[test]
fn dracosaur_grants_both_play_permissions_and_makes_each_appropriate_token_once() {
    let player = Player::P0;
    let mut game = scenario(player);
    let dragon = candidate(&mut game, player, "Bonehoard Dracosaur", Zone::Battlefield);
    assert!(
        game.object_view(dragon)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::FIRST_STRIKE))
    );
    let land = put(&mut game, player, "Forest", Zone::Library);
    let nonland = put(&mut game, player, "Shock", Zone::Library);
    step(&mut game, Step::Upkeep);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert_eq!(game.object_view(nonland).zone, Zone::Exile);
    let dinosaurs = tokens(&game, player, DINOSAUR);
    assert_eq!(dinosaurs.len(), 1);
    assert_eq!((dinosaurs[0].power, dinosaurs[0].toughness), (3, 1));
    assert!(dinosaurs[0].has_subtype("Dinosaur"));
    assert_eq!(tokens(&game, player, TREASURE).len(), 1);
    reject(&mut game, Action::PlayLand { card: land });
    game.move_object(dragon, Zone::Graveyard);
    step(&mut game, Step::Main1);
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    act(&mut game, Action::PlayLand { card: land });
    act(&mut game, Action::AnnounceExile { card: nonland });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert_eq!(
        game.state.players[player.other().index()].life,
        STARTING_LIFE - 2
    );
}

#[test]
fn deep_cavern_bat_privately_looks_and_links_only_a_nonland_while_it_stays() {
    let player = Player::P1;
    let foe = player.other();
    let mut game = scenario(player);
    let bat = candidate(&mut game, player, "Deep-Cavern Bat", Zone::Hand);
    let land = put(&mut game, foe, "Forest", Zone::Hand);
    let card = put(&mut game, foe, "Bear Cub", Zone::Hand);
    cast(&mut game, bat, None);
    advance(&mut game, |game| {
        matches!(game.state.pending, Pending::Targets { .. })
    });
    assert!(
        game.object_view(bat)
            .keywords
            .contains(Keywords::FLYING.union(Keywords::LIFELINK))
    );
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(foe),
        },
    );
    let mut departed = game.clone();
    departed.move_object(bat, Zone::Graveyard);
    advance(&mut departed, |game| {
        matches!(game.state.pending, Pending::HandCardChoice { .. })
    });
    let view = departed.observe(player).hand_choice.unwrap();
    assert_eq!(view.inspected_cards.unwrap().len(), 2);
    assert!(view.candidates.is_empty());
    act(&mut departed, Action::ChooseHandCard { object: None });
    assert_eq!(departed.object_view(card).zone, Zone::Hand);

    passes(&mut game);
    let view = game.observe(player).hand_choice.unwrap();
    let exact = game.object_ref(card);
    assert_eq!(view.inspected_cards.unwrap().len(), 2);
    assert_eq!(view.candidates, [exact]);
    assert!(game.observe(foe).hand_choice.is_none());
    let invalid = Action::ChooseHandCard {
        object: Some(game.object_ref(land)),
    };
    reject(&mut game, invalid);
    let action = Action::ChooseHandCard {
        object: Some(exact),
    };
    let saved = game.snapshot();
    let revision = game.revision();
    let index = game
        .legal_actions()
        .iter()
        .position(|candidate| *candidate == action)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, action.clone());
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(game.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    act(&mut indexed, action);
    assert_eq!(game.snapshot(), indexed.snapshot());
    assert_eq!(game.object_view(card).zone, Zone::Exile);
    game.move_object(bat, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(card).zone, Zone::Hand);
}
