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
