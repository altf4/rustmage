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

#[test]
fn vincents_limit_break_pays_each_tier_and_returns_the_exact_creature_to_its_owner_tapped() {
    let p = Player::P0;
    for (mode, generic, power) in [(0, 1, 3), (1, 2, 5), (2, 4, 7)] {
        let mut game = scenario(p);
        let bear = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
        game.record_control(game.object_ref(bear), p, None);
        game.add_counter(bear, CounterKind::PlusOne, 1);
        let spell = put(&mut game, p, "Vincent's Limit Break", Zone::Hand);
        game.state.players[p.index()].mana = [0, 0, 1, 0, 0, generic];
        act(&mut game, Action::AnnounceCast { card: spell });
        act(&mut game, Action::ChooseMode { mode });
        select_target(&mut game, bear);
        pay(&mut game);
        assert_eq!(game.state.players[p.index()].mana, [0; 6]);
        settle(&mut game);
        assert_eq!(stats(&game, bear), (power + 1, 3));
        let original = game.object_ref(bear);
        game.move_object(bear, Zone::Graveyard);
        game.prepare_triggers();
        settle(&mut game);
        let returned = game.object_view(bear);
        assert_ne!(returned.reference, original);
        assert_eq!(returned.controller, p.other());
        assert_eq!(returned.zone, Zone::Battlefield);
        assert!(returned.tapped);
        assert_eq!(stats(&game, bear), (2, 2));
    }
}

#[test]
fn warriors_sword_and_white_mages_staff_transfer_their_job_grants() {
    let p = Player::P1;
    for (name, subtype, bonus) in [
        ("Warrior's Sword", "Warrior", (3, 2)),
        ("White Mage's Staff", "Cleric", (1, 1)),
    ] {
        let mut game = scenario(p);
        let equipment = put(&mut game, p, name, Zone::Hand);
        let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
        cast(&mut game, equipment, None);
        settle(&mut game);
        let hero = game.object_view(equipment).attached_to.unwrap().card;
        assert!(game.object_view(hero).has_subtype("Hero"));
        assert!(game.object_view(hero).has_subtype(subtype));
        assert_eq!(stats(&game, hero), (1 + bonus.0, 1 + bonus.1));
        announce_ability(&mut game, equipment);
        select_target(&mut game, bear);
        pay(&mut game);
        settle(&mut game);
        assert!(!game.object_view(hero).has_subtype(subtype));
        assert_eq!(stats(&game, hero), (1, 1));
        assert!(game.object_view(bear).has_subtype(subtype));
        assert_eq!(stats(&game, bear), (2 + bonus.0, 2 + bonus.1));
        if subtype == "Cleric" {
            game.begin_step(Step::BeginCombat);
            pass_pair(&mut game);
            act(&mut game, Action::Attack { card: bear });
            act(&mut game, Action::FinishAttackers);
            settle(&mut game);
            assert_eq!(game.state.players[p.index()].life, 21);
        }
    }
}

#[test]
fn wastes_and_windurst_offer_only_their_printed_mana() {
    let p = Player::P0;
    let mut game = scenario(p);
    let wastes = put(&mut game, p, "Wastes", Zone::Hand);
    act(&mut game, Action::PlayLand { card: wastes });
    let view = game.object_view(wastes);
    assert!(!view.tapped && view.card.is_basic_land());
    for basic in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        assert!(!view.has_subtype(basic));
    }
    game.state.players[p.index()].mana = [0; 6];
    let colorless = [0, 0, 0, 0, 0, 1];
    act(
        &mut game,
        Action::ActivateMana {
            card: wastes,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: colorless,
        },
    );
    assert_eq!(game.state.players[p.index()].mana, colorless);

    let mut game = scenario(p);
    let town = put(&mut game, p, "Windurst, Federation Center", Zone::Hand);
    act(&mut game, Action::PlayLand { card: town });
    assert!(game.object_view(town).tapped && game.object_view(town).has_subtype("Town"));
    game.state.objects[town as usize].tapped = false;
    game.state.players[p.index()].mana = [0; 6];
    for production in [[1, 0, 0, 0, 0, 0], [0, 0, 0, 0, 1, 0]] {
        let mut branch = game.clone();
        act(
            &mut branch,
            Action::ActivateMana {
                card: town,
                ability: 0,
                payment: [0; 6],
                counter_payment: Counters::default(),
                production,
            },
        );
        assert_eq!(branch.state.players[p.index()].mana, production);
    }
}

#[test]
fn white_auracite_produces_white_and_keeps_the_exile_link_only_while_present() {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, "White Auracite", Zone::Hand);
    let victim = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    let original = game.object_ref(victim);
    cast(&mut game, source, None);
    pass_pair(&mut game);
    select_target(&mut game, victim);
    let mut early = game.clone();
    early.move_object(source, Zone::Graveyard);
    settle(&mut early);
    assert_eq!(early.object_ref(victim), original);
    assert_eq!(early.object_view(victim).zone, Zone::Battlefield);

    game.state.players[p.index()].mana = [0; 6];
    let white = [1, 0, 0, 0, 0, 0];
    act(
        &mut game,
        Action::ActivateMana {
            card: source,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: white,
        },
    );
    assert_eq!(game.state.players[p.index()].mana, white);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Exile);
    game.move_object(source, Zone::Graveyard);
    assert_eq!(game.object_view(victim).zone, Zone::Battlefield);
    assert_ne!(game.object_ref(victim), original);
}

#[test]
fn world_map_searches_basic_or_any_land_for_the_printed_fee() {
    let p = Player::P0;
    for (ability, fee) in [(0, 1), (1, 3)] {
        let mut game = scenario(p);
        let map = put(&mut game, p, "World Map", Zone::Battlefield);
        let basic = put(&mut game, p, "Island", Zone::Library);
        let nonbasic = put(&mut game, p, "Windurst, Federation Center", Zone::Library);
        game.state.players[p.index()].mana = [0, 0, 0, 0, 0, fee];
        act(
            &mut game,
            Action::AnnounceActivate {
                card: map,
                ability,
                counter_payment: Counters::default(),
            },
        );
        pay(&mut game);
        assert_eq!(game.object_view(map).zone, Zone::Graveyard);
        assert_eq!(game.state.players[p.index()].mana, [0; 6]);
        pass_pair(&mut game);
        assert!(game.observe(p.other()).search_choice.is_none());
        if ability == 0 {
            reject(&mut game, Action::ChooseSearch { card: nonbasic });
        }
        let card = if ability == 0 { basic } else { nonbasic };
        act(&mut game, Action::ChooseSearch { card });
        act(&mut game, Action::FinishSearch);
        assert_eq!(game.object_view(card).zone, Zone::Hand);
        assert_eq!(game.observe(p.other()).known_hand.len(), 1);
    }
}

#[test]
fn xande_counts_noncreature_nonland_graveyard_cards_and_updates_live() {
    let p = Player::P0;
    let mut game = scenario(p);
    let xande = put(&mut game, p, "Xande, Dark Mage", Zone::Battlefield);
    let spell = put(&mut game, p, "Opt", Zone::Graveyard);
    put(&mut game, p, "Short Bow", Zone::Graveyard);
    put(&mut game, p, "Spelunking", Zone::Graveyard);
    put(&mut game, p, "Forest", Zone::Graveyard);
    put(&mut game, p, "Clockwork Percussionist", Zone::Graveyard);
    put(&mut game, p.other(), "Opt", Zone::Graveyard);
    assert!(game.keywords(xande).contains(Keywords::MENACE));
    assert_eq!(stats(&game, xande), (6, 6));
    game.move_object(spell, Zone::Exile);
    assert_eq!(stats(&game, xande), (5, 5));
}

#[test]
fn youre_not_alone_checks_own_creature_count_at_resolution_and_can_target_an_opponent() {
    let p = Player::P1;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "You're Not Alone", Zone::Hand);
    let own = [(); 3].map(|_| put(&mut game, p, "Grizzly Bears", Zone::Battlefield));
    let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    cast(&mut game, spell, Some(Target::Card(enemy)));
    let mut smaller = game.clone();
    smaller.move_object(own[0], Zone::Graveyard);
    settle(&mut smaller);
    settle(&mut game);
    assert_eq!(stats(&smaller, enemy), (4, 4));
    assert_eq!(stats(&game, enemy), (6, 6));
}
