//! Typed Final Fantasy card outcomes, rows 79 through 156.
mod support;
use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn announce_ability(game: &mut Game, card: CardId, ability: u16) {
    act(
        game,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

fn resolve_entries(game: &mut Game) {
    for _ in 0..32 {
        if game.state.stack.is_empty()
            && game.state.resolving.is_none()
            && game.state.pending == Pending::Priority
        {
            return;
        }
        choose(game, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::ChooseEntryTimestamp { .. }
                    | Action::ChooseCreatedToken { .. }
            )
        });
    }
    panic!("entry triggers exceeded the scenario budget");
}

fn declare(game: &mut Game, attackers: &[CardId]) {
    game.begin_step(Step::BeginCombat);
    pass_pair(game);
    for &card in attackers {
        act(game, Action::Attack { card });
    }
    act(game, Action::FinishAttackers);
}

#[test]
fn firion_copies_equipment_once_and_sacrifices_its_exact_token_next_upkeep() {
    let p = Player::P0;
    let mut game = scenario(p);
    let firion = put(&mut game, p, "Firion, Wild Rose Warrior", Zone::Battlefield);
    let bearer = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[bearer as usize].summoning_sick = true;
    assert!(!game.keywords(bearer).contains(Keywords::HASTE));
    let original = put(&mut game, p, "Monk's Fist", Zone::Hand);
    cast(&mut game, original, None);
    resolve_entries(&mut game);
    let observed = game.observe(p.other());
    let [delayed] = observed.delayed_sacrifices.as_slice() else {
        panic!("only the one nontoken Equipment entry may trigger");
    };
    let [copied] = delayed.tokens.as_slice() else {
        panic!("the pending sacrifice records one exact token");
    };
    assert!(delayed.next_upkeep);
    assert_eq!(
        game.object_view(copied.card).card,
        game.object_view(original).card
    );
    assert!(game.object_view(copied.card).is_token());
    assert_ne!(
        game.object_view(original).attached_to,
        game.object_view(copied.card).attached_to
    );
    game.state.players[p.index()].mana = [0; 6];
    choose(&mut game, |action| {
        matches!(action,
        Action::Activate { card, target: Some(Target::Card(id)), payment, .. }
            if *card == copied.card && *id == bearer && *payment == [0; 6])
    });
    settle(&mut game);
    assert_eq!(
        game.object_view(copied.card).attached_to,
        Some(game.object_ref(bearer))
    );
    assert!(game.keywords(bearer).contains(Keywords::HASTE));
    game.move_object(firion, Zone::Graveyard);
    assert!(!game.keywords(bearer).contains(Keywords::HASTE));
    game.state.step = Step::Main2;
    pass_pair(&mut game);
    assert_eq!(game.state.step, Step::End);
    assert_eq!(game.object_view(copied.card).zone, Zone::Battlefield);
    pass_pair(&mut game);
    assert_eq!(game.state.active, p.other());
    assert_eq!(game.state.step, Step::Upkeep);
    settle(&mut game);
    assert_ne!(game.object_view(copied.card).zone, Zone::Battlefield);
    assert_eq!(game.object_view(original).zone, Zone::Battlefield);
}

#[test]
fn freya_flies_on_her_controllers_turn_and_pays_equipment_or_equip_only() {
    let p = Player::P1;
    let mut game = scenario(p);
    let freya = put(&mut game, p, "Freya Crescent", Zone::Battlefield);
    let equip = put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    let equipment = put(&mut game, p, "Lavaspur Boots", Zone::Hand);
    let artifact = put(&mut game, p, "Candy Trail", Zone::Hand);
    let bolt = put(&mut game, p, "Lightning Bolt", Zone::Hand);
    assert!(game.keywords(freya).contains(Keywords::FLYING));
    let mut other_turn = game.clone();
    other_turn.state.active = p.other();
    assert!(!other_turn.keywords(freya).contains(Keywords::FLYING));
    game.state.players[p.index()].mana = [0; 6];
    let red = [0, 0, 0, 1, 0, 0];
    let tap = Action::ActivateMana {
        card: freya,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production: red,
    };
    game.state.objects[freya as usize].summoning_sick = true;
    reject(&mut game, tap.clone());
    game.state.objects[freya as usize].summoning_sick = false;
    act(&mut game, tap);
    assert_eq!(game.state.players[p.index()].mana, red);
    for card in [artifact, bolt] {
        reject(&mut game, Action::AnnounceCast { card });
    }
    let mut paid_equip = game.clone();
    announce_ability(&mut paid_equip, equip, 0);
    act(
        &mut paid_equip,
        Action::ChooseTarget {
            target: Target::Card(freya),
        },
    );
    pay(&mut paid_equip);
    settle(&mut paid_equip);
    assert_eq!(
        paid_equip.object_view(equip).attached_to,
        Some(paid_equip.object_ref(freya))
    );
    cast(&mut game, equipment, None);
    settle(&mut game);
    assert_eq!(game.object_view(equipment).zone, Zone::Battlefield);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
}

#[test]
fn graha_draws_once_from_the_first_controlled_creature_or_artifact_batch() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "G'raha Tia", Zone::Battlefield);
    let creature = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let artifact = put(&mut game, p, "Gold Pan", Zone::Battlefield);
    let later = put(&mut game, p, "Llanowar Elves", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Bear Cub", Zone::Battlefield);
    assert!(game.keywords(source).contains(Keywords::REACH));
    game.sacrifice_batch(&[(p.other(), enemy)]);
    assert!(game.state.pending_triggers.is_empty());
    game.sacrifice_batch(&[(p, creature), (p, artifact)]);
    game.prepare_triggers();
    assert_eq!(game.state.stack.len(), 1);
    settle(&mut game);
    game.sacrifice_batch(&[(p, later)]);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    assert_eq!(game.hand_count(p), 1);
}

#[test]
fn giott_can_decline_its_entry_and_discard_to_draw_for_an_equipment() {
    let p = Player::P0;
    let mut game = scenario(p);
    let giott = put(&mut game, p, "Giott, King of the Dwarves", Zone::Hand);
    let discard = put(&mut game, p, "Forest", Zone::Hand);
    cast(&mut game, giott, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert!(game.keywords(giott).contains(Keywords::DOUBLE_STRIKE));
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(game.object_view(discard).zone, Zone::Hand);
    let equipment = put(&mut game, p, "Swiftfoot Boots", Zone::Hand);
    cast(&mut game, equipment, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discard });
    settle(&mut game);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(p), 1);
}

#[test]
fn ochu_locks_the_resolution_count_of_only_own_permanent_cards() {
    let p = Player::P0;
    let mut game = scenario(p);
    let ochu = put(&mut game, p, "Gran Pulse Ochu", Zone::Battlefield);
    let removed = put(&mut game, p, "Forest", Zone::Graveyard);
    put(&mut game, p, "Bear Cub", Zone::Graveyard);
    put(&mut game, p, "Gold Pan", Zone::Graveyard);
    put(&mut game, p, "Opt", Zone::Graveyard);
    put(&mut game, p.other(), "Bear Cub", Zone::Graveyard);
    assert!(game.keywords(ochu).contains(Keywords::DEATHTOUCH));
    announce_ability(&mut game, ochu, 0);
    pay(&mut game);
    game.move_object(removed, Zone::Exile);
    settle(&mut game);
    assert_eq!(stats(&game, ochu), (3, 3));
    put(&mut game, p, "Forest", Zone::Graveyard);
    assert_eq!(stats(&game, ochu), (3, 3));
}

#[test]
fn haste_magic_buffs_a_legal_target_and_grants_paid_play_through_the_next_own_end_step() {
    let p = Player::P0;
    let mut game = scenario(p.other());
    let target = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let spell = put(&mut game, p, "Haste Magic", Zone::Hand);
    let top = game.top_library_card(p).unwrap();
    act(&mut game, Action::Pass);
    cast(&mut game, spell, Some(Target::Card(target)));
    let mut fizzled = game.clone();
    fizzled.move_object(target, Zone::Exile);
    settle(&mut fizzled);
    assert_eq!(fizzled.object_view(top).zone, Zone::Library);
    settle(&mut game);
    assert_eq!(stats(&game, target), (5, 3));
    assert!(game.keywords(target).contains(Keywords::HASTE));
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    game.begin_step(Step::End);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .any(|row| row.object.card == top)
    );
    game.state.active = p;
    game.begin_step(Step::End);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .all(|row| row.object.card != top)
    );
}

#[test]
fn hope_uses_life_gained_at_resolution_even_after_life_loss() {
    let p = Player::P1;
    let mut game = scenario(p);
    let hope = put(&mut game, p, "Hope Estheim", Zone::Battlefield);
    assert!(game.keywords(hope).contains(Keywords::LIFELINK));
    let count = game.state.players[p.other().index()].library.len();
    game.gain_life(p, 2);
    game.lose_life(p, 3);
    game.begin_step(Step::End);
    game.prepare_triggers();
    game.gain_life(p, 1);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 20);
    assert_eq!(
        game.state.players[p.other().index()].library.len(),
        count - 3
    );
}

#[test]
fn shopkeep_targets_only_attacking_creatures_with_a_live_equipment_link() {
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p, "Item Shopkeep", Zone::Battlefield);
    let equipped = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let plain = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    let equipment = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    assert!(game.attach_equipment(game.object_ref(equipment), game.object_ref(equipped)));
    declare(&mut game, &[equipped, plain]);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(plain),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(equipped),
        },
    );
    let mut detached = game.clone();
    detached.move_object(equipment, Zone::Exile);
    settle(&mut detached);
    assert!(!detached.keywords(equipped).contains(Keywords::MENACE));
    settle(&mut game);
    assert!(game.keywords(equipped).contains(Keywords::MENACE));
}

#[test]
fn jumbo_cactuar_gains_exactly_9999_power_on_declaration() {
    let p = Player::P0;
    let mut game = scenario(p);
    let jumbo = put(&mut game, p, "Jumbo Cactuar", Zone::Battlefield);
    assert_eq!(stats(&game, jumbo), (1, 7));
    declare(&mut game, &[jumbo]);
    settle(&mut game);
    assert_eq!(stats(&game, jumbo), (10000, 7));
}

fn unblocked_damage(game: &mut Game, attackers: &[CardId]) {
    declare(game, attackers);
    assert!(game.state.stack.is_empty());
    pass_pair(game);
    act(game, Action::FinishBlockers);
    pass_pair(game);
}

#[test]
fn garnet_removes_one_lore_from_each_chosen_controlled_saga() {
    let p = Player::P0;
    let mut game = scenario(p);
    let garnet = put(
        &mut game,
        p,
        "Garnet, Princess of Alexandria",
        Zone::Battlefield,
    );
    let name = "Awaken the Honored Dead";
    let own = put(&mut game, p, name, Zone::Battlefield);
    let rival = put(&mut game, p.other(), name, Zone::Battlefield);
    for id in [own, rival] {
        game.set_counters(id, Counters::of(CounterKind::Lore, 2));
    }
    assert!(game.keywords(garnet).contains(Keywords::LIFELINK));
    declare(&mut game, &[garnet]);
    pass_pair(&mut game);
    let choice = |game: &Game, card| Action::ChooseEffectObject {
        object: game.object_ref(card),
    };
    let invalid = choice(&game, rival);
    reject(&mut game, invalid);
    let valid = choice(&game, own);
    act(&mut game, valid.clone());
    reject(&mut game, valid);
    act(&mut game, Action::FinishEffectChoice);
    assert_eq!(game.counter_count(own, CounterKind::Lore), 1);
    assert_eq!(game.counter_count(rival, CounterKind::Lore), 2);
    assert_eq!(game.counter_count(garnet, CounterKind::PlusOne), 1);
}

#[test]
fn genji_untaps_its_original_attacker_and_adds_one_combat() {
    let p = Player::P0;
    let mut game = scenario(p);
    let glove = put(&mut game, p, "Genji Glove", Zone::Battlefield);
    let attacker = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let alternate = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    announce_ability(&mut game, glove, 0);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(attacker),
        },
    );
    act(
        &mut game,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 3],
        },
    );
    settle(&mut game);
    assert!(game.keywords(attacker).contains(Keywords::DOUBLE_STRIKE));
    declare(&mut game, &[attacker]);
    assert!(game.object_view(attacker).tapped);
    assert!(game.attach_equipment(game.object_ref(glove), game.object_ref(alternate)));
    game.state.objects[alternate as usize].tapped = true;
    settle(&mut game);
    assert!(!game.object_view(attacker).tapped);
    assert!(game.object_view(alternate).tapped);
    assert!(game.keywords(alternate).contains(Keywords::DOUBLE_STRIKE));
    assert_eq!(game.observe(p).additional_combats, 1);
}

#[test]
fn judgment_bolt_counts_controlled_equipment_at_resolution() {
    let p = Player::P1;
    let mut game = scenario(p);
    let target = put(&mut game, p.other(), "Hill Giant", Zone::Battlefield);
    game.state.objects[target as usize].growth.toughness = 3;
    put(&mut game, p, "Short Bow", Zone::Battlefield);
    put(&mut game, p, "Swiftfoot Boots", Zone::Battlefield);
    put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let removed = put(&mut game, p, "Gold Pan", Zone::Battlefield);
    let spell = put(&mut game, p, "Judgment Bolt", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    game.move_object(removed, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(target).damage, 5);
    assert_eq!(game.state.players[p.other().index()].life, 18);
}

#[test]
fn kain_gives_the_exact_permanent_away_before_the_damage_sized_rewards() {
    let p = Player::P0;
    let mut game = scenario(p);
    let kain = put(&mut game, p, "Kain, Traitorous Dragoon", Zone::Battlefield);
    assert!(game.keywords(kain).contains(Keywords::FLYING));
    unblocked_damage(&mut game, &[kain]);
    let mut gone = game.clone();
    gone.move_object(kain, Zone::Exile);
    settle(&mut gone);
    assert_eq!(gone.state.players[p.index()].life, 20);
    assert_eq!(gone.hand_count(p), 0);

    settle(&mut game);
    assert_eq!(game.object_view(kain).controller, p.other());
    assert!(!game.keywords(kain).contains(Keywords::FLYING));
    assert_eq!(game.state.players[p.index()].life, 18);
    assert_eq!(game.hand_count(p), 2);
    let treasures: Vec<_> = game
        .observe(p)
        .objects
        .into_iter()
        .filter(|view| {
            view.controller == p && view.zone == Zone::Battlefield && view.has_subtype("Treasure")
        })
        .collect();
    assert_eq!(treasures.len(), 2);
    assert!(treasures.iter().all(|view| view.tapped && view.is_token()));
}

#[test]
fn light_of_judgment_destroys_one_attached_equipment_before_lethal_sbas() {
    let p = Player::P1;
    let mut game = scenario(p);
    let target = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let attached = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let loose = put(&mut game, p, "Gold Pan", Zone::Battlefield);
    let attached_ref = game.object_ref(attached);
    assert!(game.attach_equipment(attached_ref, game.object_ref(target)));
    let spell = put(&mut game, p, "Light of Judgment", Zone::Hand);
    cast(&mut game, spell, Some(Target::Card(target)));
    pass_pair(&mut game);
    assert_eq!(game.object_view(target).damage, 6);
    assert_eq!(game.object_view(target).zone, Zone::Battlefield);
    let invalid = Action::ChooseEffectObject {
        object: game.object_ref(loose),
    };
    reject(&mut game, invalid);
    act(
        &mut game,
        Action::ChooseEffectObject {
            object: attached_ref,
        },
    );
    assert_eq!(game.object_view(target).zone, Zone::Graveyard);
    assert_eq!(game.object_view(attached).zone, Zone::Graveyard);
    assert_eq!(game.object_view(loose).zone, Zone::Battlefield);
}

#[test]
fn lightning_stagger_doubles_the_following_normal_strike_after_first_strike() {
    let p = Player::P0;
    let mut game = scenario(p);
    let lightning = put(&mut game, p, "Lightning, Army of One", Zone::Battlefield);
    let follower = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    assert!(
        game.keywords(lightning).contains(
            Keywords::FIRST_STRIKE
                .union(Keywords::TRAMPLE)
                .union(Keywords::LIFELINK)
        )
    );
    unblocked_damage(&mut game, &[lightning, follower]);
    assert_eq!(game.state.step, Step::FirstStrikeDamage);
    assert_eq!(game.state.players[p.index()].life, 23);
    assert_eq!(game.state.players[p.other().index()].life, 17);
    game.move_object(lightning, Zone::Graveyard);
    settle(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.state.players[p.other().index()].life, 13);
}

#[test]
fn security_sergeant_permission_ends_at_the_first_control_loss() {
    let p = Player::P1;
    let mut game = scenario(p);
    let lightning = put(
        &mut game,
        p,
        "Lightning, Security Sergeant",
        Zone::Battlefield,
    );
    let top = game.top_library_card(p).unwrap();
    assert!(game.keywords(lightning).contains(Keywords::MENACE));
    unblocked_damage(&mut game, &[lightning]);
    settle(&mut game);
    let exiled = game.object_ref(top);
    assert_eq!(game.object_view(top).zone, Zone::Exile);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .any(|row| row.object == exiled)
    );
    let exact = game.object_ref(lightning);
    game.record_control(exact, p.other(), None);
    game.record_control(exact, p, None);
    assert!(
        game.observe(p)
            .exile_permissions
            .iter()
            .all(|row| row.object != exiled)
    );
}

#[test]
fn louisoix_counters_with_legendary_sacrifice_or_the_two_mana_fee() {
    let p = Player::P0;
    let mut original = scenario(p);
    let legend = put(
        &mut original,
        p,
        "Giott, King of the Dwarves",
        Zone::Battlefield,
    );
    let plain = put(&mut original, p, "Bear Cub", Zone::Battlefield);
    let spell = put(&mut original, p, "Louisoix's Sacrifice", Zone::Hand);
    let victim = put(&mut original, p.other(), "Opt", Zone::Hand);
    act(&mut original, Action::Pass);
    cast(&mut original, victim, None);
    act(&mut original, Action::Pass);
    for fee in [false, true] {
        let mut game = original.clone();
        if fee {
            game.move_object(legend, Zone::Hand);
        }
        let payment = [0, 1, 0, 0, 0, if fee { 2 } else { 0 }];
        game.state.players[p.index()].mana = payment;
        act(&mut game, Action::AnnounceCast { card: spell });
        act(
            &mut game,
            Action::ChooseSpellCost {
                cost: u16::from(fee),
            },
        );
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Spell(victim),
            },
        );
        if !fee {
            let invalid = Action::ChooseAdditionalCost {
                object: game.object_ref(plain),
            };
            reject(&mut game, invalid);
            let exact = game.object_ref(legend);
            act(&mut game, Action::ChooseAdditionalCost { object: exact });
        }
        act(&mut game, Action::PayCosts { payment });
        settle(&mut game);
        assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(
            game.object_view(legend).zone,
            if fee { Zone::Hand } else { Zone::Graveyard }
        );
        assert_eq!(game.object_view(plain).zone, Zone::Battlefield);
    }
}

#[test]
fn scythe_requires_a_block_and_grants_first_strike_only_after_optional_attachment() {
    let p = Player::P0;
    let mut game = scenario(p);
    let target = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let scythe = put(&mut game, p, "Magitek Scythe", Zone::Hand);
    cast(&mut game, scythe, None);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_view(scythe).attached_to, None);
    assert_eq!(declined.block_requirement_weight(target), 0);
    act(&mut game, Action::ChooseOptional { accept: true });
    assert_eq!(
        game.object_view(scythe).attached_to,
        Some(game.object_ref(target))
    );
    assert_eq!(stats(&game, target), (4, 3));
    assert!(game.keywords(target).contains(Keywords::FIRST_STRIKE));
    assert_eq!(game.block_requirement_weight(target), 1);
}

fn reach_library_selection(game: &mut Game) {
    for _ in 0..12 {
        if matches!(game.state.pending, Pending::EffectLibrarySelection { .. }) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("resolution did not reach library selection");
}

fn arrange_top(game: &mut Game, destination: LibraryDestination) {
    choose(
        game,
        |action| matches!(action, Action::ArrangeLibrary { destination: option, .. } if *option == destination),
    );
}

#[test]
fn from_father_to_son_uses_the_cast_origin_for_the_vehicles_destination() {
    let p = Player::P0;
    for (origin, destination, spell_destination) in [
        (Zone::Hand, Zone::Hand, Zone::Graveyard),
        (Zone::Graveyard, Zone::Battlefield, Zone::Exile),
    ] {
        let mut game = scenario(p);
        let spell = put(&mut game, p, "From Father to Son", origin);
        let found = put(&mut game, p, "Broadcast Rambler", Zone::Library);
        let other = put(&mut game, p, "Swiftfoot Boots", Zone::Library);
        if origin == Zone::Hand {
            cast(&mut game, spell, None);
        } else {
            act(&mut game, Action::AnnounceFlashback { card: spell });
            pay(&mut game);
        }
        pass_pair(&mut game);
        assert!(game.observe(p).search_choice.is_some());
        assert!(game.observe(p.other()).search_choice.is_none());
        reject(&mut game, Action::ChooseSearch { card: other });
        act(&mut game, Action::ChooseSearch { card: found });
        act(&mut game, Action::FinishSearch);
        assert_eq!(game.object_view(found).zone, destination);
        assert_eq!(game.object_view(spell).zone, spell_destination);
    }
}

#[test]
fn gilgamesh_can_attach_only_an_equipment_from_his_current_cohort() {
    let p = Player::P1;
    let mut game = scenario(p);
    let source = put(&mut game, p, "Gilgamesh, Master-at-Arms", Zone::Hand);
    let old = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let equipment = put(&mut game, p, "Swiftfoot Boots", Zone::Library);
    let second = put(&mut game, p, "Lavaspur Boots", Zone::Library);
    let land = put(&mut game, p, "Island", Zone::Library);
    cast(&mut game, source, None);
    reach_library_selection(&mut game);
    assert!(game.observe(p.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::SelectLibrary { card: equipment });
    act(&mut game, Action::SelectLibrary { card: second });
    act(&mut game, Action::FinishLibrarySelection);
    while matches!(game.state.pending, Pending::EntryTimestamp { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseEntryTimestamp { .. })
        });
    }
    pass_pair(&mut game);
    let recipient = game.object_ref(source);
    act(&mut game, Action::ChooseEquipmentRecipient { recipient });
    let unrelated = game.object_ref(old);
    reject(
        &mut game,
        Action::ChooseEquipmentAttachment {
            equipment: Some(unrelated),
        },
    );
    let equipment_ref = game.object_ref(equipment);
    act(
        &mut game,
        Action::ChooseEquipmentAttachment {
            equipment: Some(equipment_ref),
        },
    );
    assert_eq!(game.object_view(equipment).attached_to, Some(recipient));
    assert_eq!(game.object_view(second).attached_to, None);
    assert_eq!(game.object_view(old).attached_to, None);
}

#[test]
fn golbez_surveils_for_an_artifact_and_checks_eight_again_at_resolution() {
    let p = Player::P0;
    let mut game = scenario(p);
    put(&mut game, p, "Golbez, Crystal Collector", Zone::Battlefield);
    let entrant = put(&mut game, p, "Short Bow", Zone::Hand);
    let top = game.top_library_card(p).unwrap();
    cast(&mut game, entrant, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    arrange_top(&mut game, LibraryDestination::Graveyard);
    assert_eq!(game.object_view(top).zone, Zone::Graveyard);
    let artifacts = [(); 7].map(|_| put(&mut game, p, "Short Bow", Zone::Battlefield));
    let creature = put(&mut game, p, "Hill Giant", Zone::Graveyard);
    game.begin_step(Step::End);
    game.prepare_triggers();
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(creature),
        },
    );
    let mut seven = game.clone();
    seven.move_object(artifacts[0], Zone::Graveyard);
    settle(&mut seven);
    assert_eq!(seven.object_view(creature).zone, Zone::Hand);
    assert_eq!(seven.state.players[p.other().index()].life, 20);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Hand);
    assert_eq!(game.state.players[p.other().index()].life, 17);
}

fn cycle_for_watery_grave(name: &str) {
    let p = Player::P0;
    let mut game = scenario(p);
    let source = put(&mut game, p, name, Zone::Hand);
    let found = put(&mut game, p, "Watery Grave", Zone::Library);
    let forest = game.state.players[p.index()].library[0];
    announce_ability(&mut game, source, 0);
    pay(&mut game);
    assert_eq!(game.object_view(source).zone, Zone::Graveyard);
    pass_pair(&mut game);
    reject(&mut game, Action::ChooseSearch { card: forest });
    act(&mut game, Action::ChooseSearch { card: found });
    act(&mut game, Action::FinishSearch);
    assert_eq!(game.object_view(found).zone, Zone::Hand);
}

#[test]
fn ice_flan_stuns_an_enemy_artifact_and_cycles_for_nonbasic_islands() {
    let p = Player::P0;
    let mut game = scenario(p);
    let flan = put(&mut game, p, "Ice Flan", Zone::Hand);
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let ally = put(&mut game, p, "Short Bow", Zone::Battlefield);
    cast(&mut game, flan, None);
    pass_pair(&mut game);
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(ally),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    settle(&mut game);
    assert!(game.object_view(artifact).tapped);
    assert_eq!(game.object_view(artifact).counters.stun, 1);
    cycle_for_watery_grave("Ice Flan");
}

#[test]
fn ignis_puts_a_land_in_tapped_and_creates_food_only_for_an_exiled_creature() {
    let p = Player::P1;
    let mut game = scenario(p);
    let ignis = put(&mut game, p, "Ignis Scientia", Zone::Hand);
    let land = put(&mut game, p, "Island", Zone::Library);
    let nonland = put(&mut game, p, "Opt", Zone::Library);
    cast(&mut game, ignis, None);
    reach_library_selection(&mut game);
    assert!(game.observe(p.other()).library_selection.is_none());
    reject(&mut game, Action::SelectLibrary { card: nonland });
    act(&mut game, Action::SelectLibrary { card: land });
    act(&mut game, Action::FinishLibrarySelection);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);
    game.state.objects[ignis as usize].summoning_sick = false;
    let creature = put(&mut game, p.other(), "Grizzly Bears", Zone::Graveyard);
    let spell = put(&mut game, p, "Opt", Zone::Graveyard);
    for (target, food) in [(creature, 1), (spell, 0)] {
        let mut branch = game.clone();
        announce_ability(&mut branch, ignis, 0);
        act(
            &mut branch,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut branch);
        settle(&mut branch);
        assert_eq!(branch.object_view(target).zone, Zone::Exile);
        assert_eq!(
            branch
                .state
                .objects
                .iter()
                .filter(|object| {
                    object.zone == Zone::Battlefield && object.card == Card::from_catalogue_id(5246)
                })
                .count(),
            food
        );
    }
}

#[test]
fn malboro_discard_life_loss_and_exile_are_ordered_and_swampcycling_finds_duals() {
    let p = Player::P0;
    let mut game = scenario(p);
    let victim = p.other();
    let malboro = put(&mut game, p, "Malboro", Zone::Hand);
    let discard = put(&mut game, victim, "Opt", Zone::Hand);
    let exiled = game.state.players[victim.index()]
        .library
        .iter()
        .rev()
        .take(3)
        .copied()
        .collect::<Vec<_>>();
    cast(&mut game, malboro, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(victim));
    assert_eq!(game.state.players[victim.index()].life, 20);
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.state.players[victim.index()].life, 18);
    for card in exiled {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    cycle_for_watery_grave("Malboro");
}

#[test]
fn matoya_draws_once_for_either_scry_or_surveil() {
    let p = Player::P1;
    for (spell, destination) in [
        ("Opt", LibraryDestination::Bottom),
        ("Dreams of Laguna", LibraryDestination::Graveyard),
    ] {
        let mut game = scenario(p);
        put(&mut game, p, "Matoya, Archon Elder", Zone::Battlefield);
        let spell = put(&mut game, p, spell, Zone::Hand);
        let before = game.state.players[p.index()].cards_drawn_this_turn;
        cast(&mut game, spell, None);
        pass_pair(&mut game);
        arrange_top(&mut game, destination);
        settle(&mut game);
        assert_eq!(
            game.state.players[p.index()].cards_drawn_this_turn,
            before + 2
        );
    }
}

#[test]
fn memories_returning_publicly_alternates_and_keeps_the_flashback_exit() {
    let p = Player::P0;
    let mut game = scenario(p);
    let spell = put(&mut game, p, "Memories Returning", Zone::Graveyard);
    let cards = [
        "Forest",
        "Opt",
        "Grizzly Bears",
        "Island",
        "Swiftfoot Boots",
    ]
    .map(|name| put(&mut game, p, name, Zone::Library));
    act(&mut game, Action::AnnounceFlashback { card: spell });
    pay(&mut game);
    pass_pair(&mut game);
    let cohort = game.observe(p).library_selection.unwrap();
    assert!(cohort.public);
    assert_eq!(
        game.observe(p.other())
            .library_selection
            .unwrap()
            .candidates,
        cohort.candidates
    );
    for (index, &card) in cards[..4].iter().enumerate() {
        assert_eq!(
            game.actor(),
            Some(if index % 2 == 0 { p } else { p.other() })
        );
        act(&mut game, Action::SelectLibrary { card });
    }
    assert_eq!(game.object_view(spell).zone, Zone::Exile);
    for card in [cards[0], cards[2], cards[4]] {
        assert_eq!(game.object_view(card).zone, Zone::Hand);
    }
    for card in [cards[1], cards[3]] {
        assert_eq!(game.object_view(card).zone, Zone::Library);
    }
}

#[test]
fn minwu_counts_cleric_permanents_even_when_they_arent_creatures() {
    let p = Player::P1;
    let mut game = scenario(p);
    let minwu = put(&mut game, p, "Minwu, White Mage", Zone::Battlefield);
    let cleric = put(&mut game, p, "Hinterland Sanctifier", Zone::Battlefield);
    let foe = put(
        &mut game,
        p.other(),
        "Hinterland Sanctifier",
        Zone::Battlefield,
    );
    let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let mut kindred = CopyRecipe::new(game.object_view(cleric).card);
    kindred.overrides = Some(CopyOverrides {
        kinds: Some(32 | 128),
        ..CopyOverrides::NONE
    });
    game.apply_temporary_copy(game.object_ref(cleric), kindred, CopyDeadline::EndOfTurn);
    assert!(!game.object_view(cleric).is_creature());
    assert!(game.object_view(cleric).has_subtype("Cleric"));
    game.gain_life(p, 3);
    game.prepare_triggers();
    settle(&mut game);
    for (card, count) in [(minwu, 1), (cleric, 1), (foe, 0), (bear, 0)] {
        assert_eq!(game.object_view(card).counters.plus_one, count);
    }
}

#[test]
fn exdeath_rechecks_six_permanents_and_neo_tracks_the_live_count() {
    let p = Player::P1;
    let mut game = scenario(p);
    let exdeath = put(&mut game, p, "Exdeath, Void Warlock", Zone::Hand);
    let front = game.object_view(exdeath).card;
    let permanent = put(&mut game, p, "Forest", Zone::Graveyard);
    for _ in 0..5 {
        put(&mut game, p, "Short Bow", Zone::Graveyard);
    }
    put(&mut game, p, "Opt", Zone::Graveyard);
    put(&mut game, p.other(), "Island", Zone::Graveyard);
    cast(&mut game, exdeath, None);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 23);
    game.begin_step(Step::End);
    game.prepare_triggers();
    let mut five = game.clone();
    five.move_object(permanent, Zone::Exile);
    settle(&mut five);
    assert_eq!(five.object_view(exdeath).card, front);
    let before = game.object_ref(exdeath);
    settle(&mut game);
    assert_eq!(game.object_view(exdeath).card, front.back_face().unwrap());
    assert_eq!(game.object_ref(exdeath), before);
    assert!(game.keywords(exdeath).contains(Keywords::TRAMPLE));
    assert_eq!(stats(&game, exdeath), (6, 3));
    game.move_object(permanent, Zone::Exile);
    assert_eq!(stats(&game, exdeath), (5, 3));
}

#[test]
fn garland_surveils_and_chaos_returns_the_exact_card_to_its_owner_bottom() {
    let p = Player::P0;
    let mut game = scenario(p);
    let garland = put(
        &mut game,
        p,
        "Garland, Knight of Cornelia",
        Zone::Battlefield,
    );
    let front = game.object_view(garland).card;
    let opt = put(&mut game, p, "Opt", Zone::Hand);
    let milled = game.top_library_card(p).unwrap();
    cast(&mut game, opt, None);
    pass_pair(&mut game);
    arrange_top(&mut game, LibraryDestination::Graveyard);
    assert_eq!(game.object_view(milled).zone, Zone::Graveyard);
    pass_pair(&mut game);
    arrange_top(&mut game, LibraryDestination::Top);
    settle(&mut game);

    game.move_object(garland, Zone::Graveyard);
    let grave = game.object_ref(garland);
    announce_ability(&mut game, garland, 0);
    pay(&mut game);
    settle(&mut game);
    assert_ne!(game.object_ref(garland), grave);
    assert_eq!(game.object_view(garland).card, front.back_face().unwrap());
    assert!(game.keywords(garland).contains(Keywords::FLYING));
    game.record_control(game.object_ref(garland), p.other(), None);
    game.move_object(garland, Zone::Graveyard);
    game.prepare_triggers();
    let mut stale = game.clone();
    stale.move_object(garland, Zone::Exile);
    stale.move_object(garland, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.object_view(garland).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(garland).card, front);
    assert_eq!(game.state.players[p.index()].library[0], garland);
}

fn advance_chapter(game: &mut Game) {
    game.begin_step(Step::Main1);
    game.prepare_triggers();
}

#[test]
fn jecht_can_decline_or_return_for_his_owner_and_run_braskas_chapters() {
    let p = Player::P0;
    let owner = p.other();
    let mut game = scenario(p);
    let jecht = put(
        &mut game,
        owner,
        "Jecht, Reluctant Guardian",
        Zone::Battlefield,
    );
    game.record_control(game.object_ref(jecht), p, None);
    game.state.objects[jecht as usize].summoning_sick = false;
    let front = game.object_view(jecht).card;
    let old = game.object_ref(jecht);
    let discard = put(&mut game, p, "Opt", Zone::Hand);
    assert!(game.keywords(jecht).contains(Keywords::MENACE));
    unblocked_damage(&mut game, &[jecht]);
    pass_pair(&mut game);
    let mut declined = game.clone();
    act(&mut declined, Action::ChooseOptional { accept: false });
    assert_eq!(declined.object_ref(jecht), old);
    assert_eq!(declined.object_view(jecht).controller, p);
    act(&mut game, Action::ChooseOptional { accept: true });
    let braska = game.object_view(jecht);
    assert_ne!(braska.reference, old);
    assert_eq!(braska.card, front.back_face().unwrap());
    assert_eq!(braska.controller, owner);
    assert_eq!(braska.counters.lore, 1);
    assert!(game.keywords(jecht).contains(Keywords::MENACE));
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discard });
    assert_eq!(game.hand_count(owner), 1);

    game.state.active = owner;
    game.state.priority = owner;
    let second = put(&mut game, p, "Forest", Zone::Hand);
    advance_chapter(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: second });
    assert_eq!(game.hand_count(owner), 2);
    let picks = ["Grizzly Bears", "Bear Cub", "Hill Giant"]
        .map(|name| put(&mut game, p, name, Zone::Battlefield));
    advance_chapter(&mut game);
    pass_pair(&mut game);
    for card in [picks[0], picks[2]] {
        let object = game.object_ref(card);
        act(&mut game, Action::Sacrifice { object });
    }
    assert_eq!(game.object_view(picks[0]).zone, Zone::Graveyard);
    assert_eq!(game.object_view(picks[1]).zone, Zone::Battlefield);
    assert_eq!(game.object_view(picks[2]).zone, Zone::Graveyard);
    assert_eq!(game.object_view(jecht).zone, Zone::Graveyard);
    assert_eq!(game.object_view(jecht).card, front);
}

#[test]
fn overture_mills_half_rounded_down_then_jidoor_can_enter_tapped() {
    let p = Player::P1;
    let mut game = scenario(p);
    put(&mut game, p.other(), "Island", Zone::Library);
    let land = put(&mut game, p, "Jidoor, Aristocratic Capital", Zone::Hand);
    act(&mut game, Action::AnnounceAdventure { card: land });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(p.other()),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[p.other().index()].library.len(), 5);
    assert_eq!(game.state.players[p.index()].library.len(), 8);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.object_view(land).tapped);
    assert!(game.object_view(land).has_subtype("Town"));
}

#[test]
fn jill_bounces_another_nonland_and_shiva_reenters_front_after_tapping_enemy_lands() {
    let p = Player::P0;
    let mut game = scenario(p);
    let jill = put(&mut game, p, "Jill, Shiva's Dominant", Zone::Hand);
    let front = game.object_view(jill).card;
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Battlefield);
    let attacker = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let own_land = put(&mut game, p, "Island", Zone::Battlefield);
    let enemy_land = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    cast(&mut game, jill, None);
    pass_pair(&mut game);
    for id in [jill, enemy_land] {
        reject(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(artifact),
        },
    );
    settle(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Hand);
    game.state.objects[jill as usize].summoning_sick = false;
    announce_ability(&mut game, jill, 0);
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(jill).card, front.back_face().unwrap());
    assert_eq!(game.object_view(jill).counters.lore, 1);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(attacker),
        },
    );
    settle(&mut game);
    assert!(game.keywords(attacker).contains(Keywords::CANT_BE_BLOCKED));
    advance_chapter(&mut game);
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(jill),
        },
    );
    settle(&mut game);
    assert!(game.keywords(jill).contains(Keywords::CANT_BE_BLOCKED));

    let shiva = game.object_ref(jill);
    advance_chapter(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.object_view(jill).card, front);
    assert_ne!(game.object_ref(jill), shiva);
    assert!(game.object_view(enemy_land).tapped);
    assert!(!game.object_view(own_land).tapped);
    act(&mut game, Action::FinishTargets);
    settle(&mut game);
}

#[test]
fn phoenix_lifelink_damage_and_rebirth_use_the_actual_chapter_and_mana_budget() {
    let p = Player::P1;
    let mut game = scenario(p);
    let joshua = put(&mut game, p, "Joshua, Phoenix's Dominant", Zone::Hand);
    let front = game.object_view(joshua).card;
    let discard = put(&mut game, p, "Opt", Zone::Hand);
    let kept = put(&mut game, p, "Island", Zone::Hand);
    cast(&mut game, joshua, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(&mut game, Action::Discard { card: discard });
    act(&mut game, Action::DeclineEffectPayment);
    assert_eq!(game.object_view(discard).zone, Zone::Graveyard);
    assert_eq!(game.object_view(kept).zone, Zone::Hand);
    assert_eq!(game.hand_count(p), 2);
    game.state.objects[joshua as usize].summoning_sick = false;
    announce_ability(&mut game, joshua, 0);
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(joshua).card, front.back_face().unwrap());
    assert!(game.keywords(joshua).contains(Keywords::FLYING));
    assert_eq!(game.state.players[p.index()].life, 22);
    assert_eq!(game.state.players[p.other().index()].life, 18);
    advance_chapter(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[p.index()].life, 24);
    assert_eq!(game.state.players[p.other().index()].life, 16);

    let four = put(&mut game, p, "Hill Giant", Zone::Graveyard);
    let two = put(&mut game, p, "Grizzly Bears", Zone::Graveyard);
    let one = put(&mut game, p, "Llanowar Elves", Zone::Graveyard);
    advance_chapter(&mut game);
    for id in [four, two] {
        act(
            &mut game,
            Action::ChooseTarget {
                target: Target::Card(id),
            },
        );
    }
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(one),
        },
    );
    act(&mut game, Action::FinishTargets);
    pass_pair(&mut game);
    while matches!(game.state.pending, Pending::EntryTimestamp { .. }) {
        choose(&mut game, |action| {
            matches!(action, Action::ChooseEntryTimestamp { .. })
        });
    }
    assert_eq!(game.object_view(four).zone, Zone::Battlefield);
    assert_eq!(game.object_view(two).zone, Zone::Battlefield);
    assert_eq!(game.object_view(one).zone, Zone::Graveyard);
    assert_eq!(game.object_view(joshua).card, front);
    pass_pair(&mut game);
    act(&mut game, Action::DeclineEffectPayment);
}

#[test]
fn kefka_draws_distinct_discard_types_then_ruler_counts_opponent_life_loss() {
    let p = Player::P0;
    let mut game = scenario(p);
    let kefka = put(&mut game, p, "Kefka, Court Mage", Zone::Hand);
    let front = game.object_view(kefka).card;
    let artifact_creature = put(&mut game, p, "Clockwork Percussionist", Zone::Hand);
    let artifact = put(&mut game, p.other(), "Short Bow", Zone::Hand);
    let sacrifice = put(&mut game, p.other(), "Forest", Zone::Battlefield);
    cast(&mut game, kefka, None);
    pass_pair(&mut game);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::Discard {
            card: artifact_creature,
        },
    );
    assert_eq!(game.object_view(artifact_creature).zone, Zone::Hand);
    assert_eq!(game.actor(), Some(p.other()));
    act(&mut game, Action::Discard { card: artifact });
    assert_eq!(game.object_view(artifact_creature).zone, Zone::Graveyard);
    assert_eq!(game.object_view(artifact).zone, Zone::Graveyard);
    assert_eq!(game.hand_count(p), 2);
    announce_ability(&mut game, kefka, 0);
    pay(&mut game);
    pass_pair(&mut game);
    assert_eq!(game.actor(), Some(p.other()));
    let object = game.object_ref(sacrifice);
    act(&mut game, Action::Sacrifice { object });
    assert_eq!(game.object_view(sacrifice).zone, Zone::Graveyard);
    assert_eq!(game.object_view(kefka).card, front.back_face().unwrap());
    assert!(game.keywords(kefka).contains(Keywords::FLYING));

    game.lose_life(p.other(), 3);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.hand_count(p), 5);
    game.state.active = p.other();
    game.lose_life(p.other(), 2);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.hand_count(p), 5);
}

#[test]
fn fang_triggers_once_for_a_graveyard_departure_batch_and_resets_next_turn() {
    let p = Player::P1;
    let opponent = p.other();
    let mut game = scenario(p);
    put(&mut game, p, "Fang, Fearless l'Cie", Zone::Battlefield);
    let own = ["Forest", "Opt"].map(|name| put(&mut game, p, name, Zone::Graveyard));
    let their = put(&mut game, opponent, "Forest", Zone::Graveyard);
    let lantern = put(&mut game, opponent, "Soul-Guide Lantern", Zone::Battlefield);
    game.move_object(their, Zone::Exile);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    act(&mut game, Action::Pass);
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 0, .. } if *card == lantern),
    );
    settle(&mut game);
    assert_eq!(game.hand_count(p), 1);
    assert_eq!(game.state.players[p.index()].life, 19);
    for card in own {
        assert_eq!(game.object_view(card).zone, Zone::Exile);
    }
    let later = put(&mut game, p, "Island", Zone::Graveyard);
    game.move_object(later, Zone::Exile);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    game.state.step = Step::End;
    pass_pair(&mut game);
    assert_eq!(game.state.active, opponent);
    game.move_object(later, Zone::Graveyard);
    game.move_object(later, Zone::Exile);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.hand_count(p), 2);
    assert_eq!(game.state.players[p.index()].life, 18);
}

#[test]
fn gogo_pays_two_per_x_and_copies_a_frozen_trigger_twice_with_independent_targets() {
    let p = Player::P0;
    let mut game = scenario(p);
    let gogo = put(&mut game, p, "Gogo, Master of Mimicry", Zone::Battlefield);
    let bear = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let lion_heart = put(&mut game, p, "Lion Heart", Zone::Hand);
    cast(&mut game, lion_heart, None);
    pass_pair(&mut game);
    let original = Target::Player(p.other());
    act(&mut game, Action::ChooseTarget { target: original });
    let ability = game.state.stack.last().unwrap().id;
    game.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    announce_ability(&mut game, gogo, 0);
    reject(&mut game, Action::ChooseX { value: 0 });
    reject(&mut game, Action::ChooseX { value: 3 });
    act(&mut game, Action::ChooseX { value: 2 });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Stack(ability),
        },
    );
    pay(&mut game);
    assert_eq!(game.state.players[p.index()].mana, [0; 6]);
    assert!(game.object_view(gogo).tapped);
    pass_pair(&mut game);
    act(
        &mut game,
        Action::SetStackTarget {
            index: 0,
            target: Target::Card(bear),
        },
    );
    act(&mut game, Action::FinishStackTargets);
    assert_eq!(
        game.observe(p).retarget.unwrap().item.targets[0].target,
        original
    );
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(game.state.players[p.other().index()].life, 16);
}

#[test]
fn jenova_places_live_or_lki_power_counters_and_draws_for_a_real_mutant_death() {
    let p = Player::P1;
    let mut game = scenario(p);
    let jenova = put(&mut game, p, "Jenova, Ancient Calamity", Zone::Battlefield);
    let bear = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[jenova as usize].growth.power = 2;
    game.begin_step(Step::BeginCombat);
    game.prepare_triggers();
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(jenova),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Card(bear),
        },
    );
    let mut without_source = game.clone();
    without_source.move_object(jenova, Zone::Exile);
    settle(&mut without_source);
    assert_eq!(without_source.object_view(bear).counters.plus_one, 3);
    assert!(without_source.object_view(bear).has_subtype("Mutant"));

    settle(&mut game);
    assert_eq!(stats(&game, bear), (5, 5));
    assert!(game.object_view(bear).has_subtype("Bear"));
    assert!(game.object_view(bear).has_subtype("Mutant"));
    let mut opponent_turn = game.clone();
    opponent_turn.state.active = p.other();
    opponent_turn.move_object(bear, Zone::Graveyard);
    opponent_turn.prepare_triggers();
    assert!(opponent_turn.state.stack.is_empty());
    game.move_object(bear, Zone::Graveyard);
    game.prepare_triggers();
    settle(&mut game);
    assert_eq!(game.hand_count(p), 5);
}

#[test]
fn judge_counts_each_other_controlled_creature_or_artifact_death_once() {
    let p = Player::P0;
    let mut game = scenario(p);
    let judge = put(&mut game, p, "Judge Magister Gabranth", Zone::Battlefield);
    let artifact = put(&mut game, p, "Short Bow", Zone::Battlefield);
    let artifact_creature = put(&mut game, p, "Clockwork Percussionist", Zone::Battlefield);
    let creature = put(&mut game, p, "Grizzly Bears", Zone::Battlefield);
    let enemy = put(&mut game, p.other(), "Grizzly Bears", Zone::Battlefield);
    let exiled = put(&mut game, p, "Bear Cub", Zone::Battlefield);
    assert!(game.keywords(judge).contains(Keywords::MENACE));
    game.move_object(exiled, Zone::Exile);
    game.prepare_triggers();
    assert!(game.state.stack.is_empty());
    game.sacrifice_batch(&[
        (p, artifact),
        (p, artifact_creature),
        (p, creature),
        (p.other(), enemy),
    ]);
    game.prepare_triggers();
    resolve_entries(&mut game);
    assert_eq!(game.object_view(judge).counters.plus_one, 3);
    assert_eq!(stats(&game, judge), (5, 5));
}

fn black_wizard(game: &Game, player: Player) -> ObjectView {
    let token = Card::from_catalogue_id(5531);
    let mut found = game.state.objects.iter().enumerate().filter(|(_, object)| {
        object.zone == Zone::Battlefield && object.controller == player && object.card == token
    });
    let (index, _) = found.next().expect("one Black Wizard token");
    assert!(found.next().is_none());
    let wizard = game.object_view(index as CardId);
    assert!(wizard.is_token() && wizard.has_subtype("Wizard"));
    assert_eq!(wizard.colors, 4);
    assert_eq!(stats(game, wizard.id), (0, 1));
    wizard
}

#[test]
fn kuja_counts_the_new_tapped_wizard_before_transforming_and_doubles_its_damage() {
    let p = Player::P0;
    let mut game = scenario(p);
    let kuja = put(&mut game, p, "Kuja, Genome Sorcerer", Zone::Battlefield);
    let first = put(&mut game, p, "Ghitu Lavarunner", Zone::Battlefield);
    put(&mut game, p, "Mysidian Elder", Zone::Battlefield);
    put(&mut game, p.other(), "Ghitu Lavarunner", Zone::Battlefield);
    let front = game.object_view(kuja).card;
    game.begin_step(Step::End);
    game.prepare_triggers();
    let mut short = game.clone();
    short.move_object(first, Zone::Exile);
    settle(&mut short);
    assert_eq!(short.object_view(kuja).card, front);
    settle(&mut game);
    assert_eq!(game.object_view(kuja).card, front.back_face().unwrap());
    for (branch, expected_life) in [(&mut short, 19), (&mut game, 18)] {
        assert!(black_wizard(branch, p).tapped);
        let opt = put(branch, p, "Opt", Zone::Hand);
        cast(branch, opt, None);
        pass_pair(branch);
        assert_eq!(branch.state.players[p.other().index()].life, expected_life);
        pass_pair(branch);
        arrange_top(branch, LibraryDestination::Top);
    }
}

#[test]
fn mage_siege_is_an_off_turn_adventure_and_the_town_can_later_enter_tapped() {
    let active = Player::P0;
    let caster = active.other();
    let mut game = scenario(active);
    let land = put(
        &mut game,
        caster,
        "Lindblum, Industrial Regency",
        Zone::Hand,
    );
    act(&mut game, Action::Pass);
    act(&mut game, Action::AnnounceAdventure { card: land });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
    assert!(!black_wizard(&game, caster).tapped);
    assert_eq!(game.state.players[active.index()].life, 20);
    reject(&mut game, Action::PlayLand { card: land });
    game.state.active = caster;
    game.state.priority = caster;
    act(&mut game, Action::PlayLand { card: land });
    assert!(game.object_view(land).tapped);
    assert!(game.object_view(land).has_subtype("Town"));
    game.state.objects[land as usize].tapped = false;
    game.state.players[caster.index()].mana = [0; 6];
    act(&mut game, Action::TapMana { card: land });
    assert_eq!(game.state.players[caster.index()].mana, [0, 0, 0, 1, 0, 0]);
}

#[test]
fn mysidian_elders_wizard_triggers_before_a_spell_that_is_countered() {
    let p = Player::P1;
    let mut game = scenario(p);
    let elder = put(&mut game, p, "Mysidian Elder", Zone::Hand);
    cast(&mut game, elder, None);
    settle(&mut game);
    let token = black_wizard(&game, p);
    assert!(!token.tapped);
    let spell = put(&mut game, p, "Opt", Zone::Hand);
    cast(&mut game, spell, None);
    let trigger = game.state.stack.last().unwrap();
    assert_eq!(trigger.source().unwrap().object, token.reference);
    act(&mut game, Action::Pass);
    let negate = put(&mut game, p.other(), "Negate", Zone::Hand);
    cast(&mut game, negate, Some(Target::Spell(spell)));
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(game.state.players[p.other().index()].life, 19);
    assert_eq!(game.hand_count(p), 0);
}
