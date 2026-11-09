//! Typed Final Fantasy card outcomes, rows 292 through 312.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn announce_ability(game: &mut Game, card: CardId) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
}

fn select_target(game: &mut Game, card: CardId) {
    act(
        game,
        Action::ChooseTarget {
            target: Target::Card(card),
        },
    );
}

fn drain_triggers(game: &mut Game) {
    for _ in 0..24 {
        if game.state.stack.is_empty()
            && game.state.pending_triggers.is_empty()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        game.prepare_triggers();
        choose(game, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::ChooseEntryTimestamp { .. }
            )
        });
    }
    panic!("the scenario exceeded its trigger budget");
}

#[test]
fn vivi_is_freeform_only_and_splits_live_power_without_tapping() {
    let p = Player::P0;
    let card = Card::from_name("Vivi Ornitier").unwrap();
    let mut decks = [vec![Card::Forest; 60], vec![Card::Forest; 60]];
    decks[p.index()].push(card);
    assert!(
        Game::new(Config {
            decks,
            seed: 41,
            starting_player: p,
            deck_rules: DeckRules::Standard
        })
        .is_err()
    );

    let mut game = scenario(p);
    let vivi = put(&mut game, p, "Vivi Ornitier", Zone::Hand);
    cast(&mut game, vivi, None);
    settle(&mut game);
    let zero = [0; 6];
    let start = Action::ActivateMana {
        card: vivi,
        ability: 0,
        payment: zero,
        counter_payment: Counters::default(),
        production: zero,
    };
    let mut empty = game.clone();
    empty.state.players[p.index()].mana = zero;
    act(&mut empty, start.clone());
    assert_eq!(empty.state.players[p.index()].mana, zero);
    assert!(empty.state.stack.is_empty());
    reject(&mut empty, start.clone());

    let mut opponent_turn = game.clone();
    opponent_turn.state.active = p.other();
    reject(&mut opponent_turn, start.clone());
    game.add_counter(vivi, CounterKind::PlusOne, 2);
    let spell = put(&mut game, p, "Lightning Bolt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Player(p.other())));
    pass_pair(&mut game);
    assert_eq!(game.counter_count(vivi, CounterKind::PlusOne), 3);
    assert_eq!(game.state.players[p.other().index()].life, 19);
    game.state.players[p.index()].mana = zero;
    act(&mut game, start.clone());
    let split = [0, 1, 0, 2, 0, 0];
    act(&mut game, Action::ChooseMana { production: split });
    assert_eq!(game.state.players[p.index()].mana, split);
    assert!(!game.object_view(vivi).tapped);
    reject(&mut game, start);
    settle(&mut game);
    assert_eq!(game.state.players[p.other().index()].life, 16);
}

#[test]
fn vendor_rechecks_equipment_and_announces_two_targets_only_after_payment() {
    let p = Player::P1;
    let mut game = scenario(p);
    let vendor = put(&mut game, p, "Weapons Vendor", Zone::Hand);
    let equipment = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let opposing = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let bearer = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, vendor, None);
    settle(&mut game);
    assert_eq!(game.hand_count(p), 1);
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    assert!(game.state.stack[0].targets.is_empty());

    let mut removed = game.clone();
    removed.move_object(equipment, Zone::Graveyard);
    settle(&mut removed);
    assert_eq!(removed.object_view(equipment).attached_to, None);
    pass_pair(&mut game);
    assert!(matches!(game.state.pending, Pending::ResolutionMana { .. }));
    choose(&mut game, |action| {
        matches!(action, Action::PayResolutionMana { .. })
    });
    assert!(matches!(game.state.pending, Pending::Targets { .. }));
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(opposing),
        },
    );
    select_target(&mut game, equipment);
    select_target(&mut game, bearer);
    assert_eq!(game.object_view(equipment).attached_to, None);
    settle(&mut game);
    assert_eq!(
        game.object_view(equipment).attached_to,
        Some(game.object_ref(bearer))
    );
}

#[test]
fn yuna_grants_ward_only_on_her_turn_and_returns_an_enchantment_with_finality() {
    let p = Player::P0;
    let mut game = scenario(p);
    let yuna = put(&mut game, p, "Yuna, Hope of Spira", Zone::Battlefield);
    let live = put(&mut game, p, "Summon: Primal Garuda", Zone::Battlefield);
    let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let granted = Keywords::TRAMPLE.union(Keywords::LIFELINK);
    for card in [yuna, live] {
        assert!(game.keywords(card).contains(granted));
    }
    assert!(!game.keywords(bear).intersects(granted));
    let mut other_turn = game.clone();
    other_turn.state.active = p.other();
    assert!(!other_turn.keywords(yuna).intersects(granted));
    assert!(!other_turn.keywords(live).intersects(granted));

    let mut ward = game.clone();
    act(&mut ward, Action::Pass);
    let bounce = put(&mut ward, p.other(), "Unsummon", Zone::Hand);
    cast(&mut ward, bounce, Some(Target::Card(live)));
    assert_eq!(ward.state.stack.len(), 2);
    pass_pair(&mut ward);
    act(&mut ward, Action::DeclineResolutionMana);
    assert_eq!(ward.object_view(live).zone, Zone::Battlefield);
    assert_eq!(ward.object_view(bounce).zone, Zone::Graveyard);

    let returned = put(&mut game, p, "Summon: Shiva", Zone::Graveyard);
    let illegal = put(&mut game, p, "Bear Cub", Zone::Graveyard);
    let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    game.begin_step(Step::End);
    game.prepare_triggers();
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(illegal),
        },
    );
    select_target(&mut game, returned);
    pass_pair(&mut game);
    assert_eq!(game.counter_count(returned, CounterKind::Lore), 1);
    assert_eq!(game.counter_count(returned, CounterKind::Finality), 1);
    select_target(&mut game, enemy);
    settle(&mut game);
    game.move_object(returned, Zone::Graveyard);
    assert_eq!(game.object_view(returned).zone, Zone::Exile);
}

#[test]
fn zack_transfers_all_counters_and_one_exact_pre_sacrifice_equipment() {
    let p = Player::P0;
    let mut game = scenario(p);
    let zack = put(&mut game, p, "Zack Fair", Zone::Hand);
    let bearer = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let local = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let foreign = put(&mut game, p.other(), "Swiftfoot Boots", Zone::Battlefield);
    cast(&mut game, zack, None);
    settle(&mut game);
    assert_eq!(stats(&game, zack), (1, 2));
    game.add_counter(zack, CounterKind::Charge, 3);
    let zack_ref = game.object_ref(zack);
    for card in [local, foreign] {
        assert!(game.attach_equipment(game.object_ref(card), zack_ref));
    }
    announce_ability(&mut game, zack);
    select_target(&mut game, bearer);
    pay(&mut game);
    assert_eq!(game.object_view(zack).zone, Zone::Graveyard);
    pass_pair(&mut game);
    assert!(game.keywords(bearer).contains(Keywords::INDESTRUCTIBLE));
    assert_eq!(game.counter_count(bearer, CounterKind::PlusOne), 1);
    assert_eq!(game.counter_count(bearer, CounterKind::Charge), 3);
    reject(
        &mut game,
        Action::ChooseEquipmentAttachment { equipment: None },
    );
    let equipment = game.object_ref(foreign);
    act(
        &mut game,
        Action::ChooseEquipmentAttachment {
            equipment: Some(equipment),
        },
    );
    assert_eq!(
        game.object_view(foreign).attached_to,
        Some(game.object_ref(bearer))
    );
    assert_eq!(game.object_view(local).attached_to, None);
}

#[test]
fn lasting_fayth_samples_current_lands_and_permits_one_later_zanarkand_play() {
    let p = Player::P1;
    let mut game = scenario(p);
    let land = put(&mut game, p, "Zanarkand, Ancient Metropolis", Zone::Hand);
    let removed = put(&mut game, p, "Forest", Zone::Battlefield);
    put(&mut game, p, "Island", Zone::Battlefield);
    put(&mut game, p.other(), "Island", Zone::Battlefield);
    act(&mut game, Action::AnnounceAdventure { card: land });
    pay(&mut game);
    game.move_object(removed, Zone::Graveyard);
    settle(&mut game);
    let hero = game
        .observe(p)
        .objects
        .into_iter()
        .find(|object| {
            object.zone == Zone::Battlefield && object.controller == p && object.is_token()
        })
        .unwrap();
    assert!(hero.has_subtype("Hero"));
    assert_eq!(hero.colors, 0);
    assert_eq!(hero.counters.plus_one, 1);
    assert_eq!(stats(&game, hero.id), (2, 2));
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.object_view(land).tapped && game.object_view(land).has_subtype("Town"));
    let second = put(&mut game, p, "Forest", Zone::Hand);
    reject(&mut game, Action::PlayLand { card: second });
}

#[test]
fn zenos_chooses_without_targeting_and_shinryu_records_the_transform_opponent() {
    let p = Player::P0;
    let mut game = scenario(p);
    let zenos = put(&mut game, p, "Zenos yae Galvus", Zone::Hand);
    let front = game.object_view(zenos).card;
    let chosen = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let other = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    game.grant_keyword_effects(&[chosen], game.object_ref(chosen), Keywords::HEXPROOF);
    cast(&mut game, zenos, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    let object = game.object_ref(chosen);
    act(&mut game, Action::ChooseLinkedCreature { object });
    assert_eq!(stats(&game, zenos), (4, 4));
    assert_eq!(stats(&game, chosen), (3, 3));
    assert_eq!(stats(&game, other), (1, 1));
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
    game.move_object(other, Zone::Hand);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    let before = game.object_ref(zenos);
    game.move_object(chosen, Zone::Hand);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.object_ref(zenos), before);
    assert_eq!(game.object_view(zenos).card, front.back_face().unwrap());
    assert!(game.keywords(zenos).contains(Keywords::FLYING));
    assert_eq!(
        game.object_view(zenos).face.unwrap().chosen_opponent,
        Some(p.other())
    );
    game.apply(p.other(), Action::Concede).unwrap();
    assert_eq!(game.outcome(), Some(Outcome::Winner(p)));
    assert_eq!(game.state.pending_triggers.len(), 1);
}

#[test]
fn zidane_steals_untaps_grants_keywords_and_sees_control_return_at_cleanup() {
    let p = Player::P1;
    let mut game = scenario(p);
    let zidane = put(&mut game, p, "Zidane, Tantalus Thief", Zone::Hand);
    let target = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    game.state.objects[target as usize].tapped = true;
    cast(&mut game, zidane, None);
    pass_pair(&mut game);
    select_target(&mut game, target);
    settle(&mut game);
    assert_eq!(game.object_view(target).controller, p);
    assert!(!game.object_view(target).tapped);
    let granted = Keywords::HASTE.union(Keywords::LIFELINK);
    assert!(game.keywords(target).contains(granted));
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::Cleanup);
    assert_eq!(game.object_view(target).controller, p.other());
    assert!(!game.keywords(target).intersects(granted));
    drain_triggers(&mut game);
    let treasures = game
        .observe(p)
        .objects
        .into_iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == p
                && object.card == Card::from_catalogue_id(5302)
        })
        .count();
    assert_eq!(treasures, 1);
}

#[test]
fn zodiark_excludes_gods_rounds_each_quota_down_and_counts_actual_sacrifices() {
    let p = Player::P0;
    let mut game = scenario(p);
    let zodiark = put(&mut game, p, "Zodiark, Umbral God", Zone::Hand);
    let own = [(); 3].map(|_| put(&mut game, p, "Grizzly Bears", Zone::Battlefield));
    let theirs = [(); 4].map(|_| put(&mut game, p.other(), "Hill Giant", Zone::Battlefield));
    let changeling = put(&mut game, p, "Three Tree Mascot", Zone::Battlefield);
    cast(&mut game, zodiark, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.keywords(zodiark).contains(Keywords::INDESTRUCTIBLE));
    for god in [zodiark, changeling] {
        let object = game.object_ref(god);
        reject(&mut game, Action::Sacrifice { object });
    }
    let object = game.object_ref(own[0]);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(own[0]).zone, Zone::Battlefield);
    assert_eq!(game.actor(), Some(p.other()));
    for card in [theirs[0], theirs[2]] {
        let object = game.object_ref(card);
        act(&mut game, Action::Sacrifice { object });
    }
    drain_triggers(&mut game);
    for card in [own[0], theirs[0], theirs[2]] {
        assert_eq!(game.object_view(card).zone, Zone::Graveyard);
    }
    for card in [own[1], own[2], theirs[1], theirs[3], changeling] {
        assert_eq!(game.object_view(card).zone, Zone::Battlefield);
    }
    assert_eq!(game.counter_count(zodiark, CounterKind::PlusOne), 3);
}
