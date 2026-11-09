//! Native scenarios for the first 78 Final Fantasy cards.
mod support;

use rustmage as _;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn reach(game: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..48 {
        if game.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(game, |action| {
            matches!(action, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("scenario action was never offered");
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn announce_ability(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}

fn mana(card: CardId, production: Mana) -> Action {
    Action::ActivateMana {
        card,
        ability: 0,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    }
}

#[test]
fn ambrosia_flashes_in_and_bounces_to_the_owner_then_benefits_from_landfall() {
    let player = Player::P1;
    let owner = player.other();
    let mut game = scenario(owner);
    let bird = put(&mut game, player, "Ambrosia Whiteheart", Zone::Hand);
    let stolen = put(&mut game, owner, "Grizzly Bears", Zone::Battlefield);
    game.record_control(game.object_ref(stolen), player, None);
    game.state.objects[stolen as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
    act(&mut game, Action::Pass);
    assert_eq!(game.actor(), Some(player));
    cast(&mut game, bird, None);
    let bounce = Action::ChooseEffectObject {
        object: game.object_ref(stolen),
    };
    reach(&mut game, |a| *a == bounce);
    let own_source = Action::ChooseEffectObject {
        object: game.object_ref(bird),
    };
    reject(&mut game, own_source);
    let mut decline = game.clone();
    act(&mut decline, Action::ChooseOptional { accept: false });
    assert_eq!(decline.object_view(stolen).zone, Zone::Battlefield);

    act(&mut game, bounce);
    assert_eq!(game.object_view(stolen).zone, Zone::Hand);
    assert!(
        game.observe(owner)
            .objects
            .iter()
            .any(|card| card.id == stolen)
    );
    assert_eq!(stats(&game, bird), (2, 2));

    game.state.active = player;
    game.state.priority = player;
    let land = put(&mut game, player, "Plains", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    settle(&mut game);
    assert_eq!(stats(&game, bird), (3, 2));
}

#[test]
fn cloud_of_darkness_counts_only_own_graveyard_permanents() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cloud = put(&mut game, player, "Cloud of Darkness", Zone::Hand);
    let own = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let opposing = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    put(&mut game, player, "Forest", Zone::Graveyard);
    put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    put(&mut game, player, "Shock", Zone::Graveyard);
    put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);

    cast(&mut game, cloud, None);
    reach(&mut game, |a| *a == target(opposing));
    reject(&mut game, target(own));
    act(&mut game, target(opposing));
    settle(&mut game);
    assert_eq!(stats(&game, opposing), (1, 1));
    assert_eq!(stats(&game, cloud), (3, 3));
    assert!(game.keywords(cloud).contains(Keywords::FLYING));
}

#[test]
fn crossroads_chooses_as_it_enters_and_only_produces_that_color() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Crossroads Village", Zone::Hand);
    act(&mut game, Action::PlayLand { card: land });
    assert!(matches!(game.state.pending, Pending::EntryValue { .. }));
    assert_eq!(
        game.legal_actions()
            .iter()
            .filter(|action| { matches!(action, Action::ChooseEntryValue { .. }) })
            .count(),
        5,
    );
    let selection = Action::ChooseEntryValue {
        value: EntryChoiceValue::Color(ChoiceColor::Red),
    };
    let index = game
        .legal_actions()
        .iter()
        .position(|action| *action == selection)
        .unwrap();
    let mut indexed = game.clone();
    act(&mut game, selection);
    let revision = indexed.revision();
    indexed.apply_index(player, revision, index).unwrap();
    assert_eq!(indexed.snapshot(), game.snapshot());
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    assert!(game.object_view(land).tapped);

    let red = [0, 0, 0, 1, 0, 0];
    reject(&mut game, mana(land, red));
    game.state.objects[land as usize].tapped = false;
    game.state.players[player.index()].mana = [0; 6];
    reject(&mut game, mana(land, [1, 0, 0, 0, 0, 0]));
    act(&mut game, mana(land, red));
    assert_eq!(game.state.players[player.index()].mana, red);
}

#[test]
fn deadly_embrace_draws_for_the_destroyed_creature_and_prior_deaths() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Deadly Embrace", Zone::Hand);
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    let earlier = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let victim = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    cast(&mut game, shock, Some(Target::Card(earlier)));
    settle(&mut game);
    assert_eq!(game.object_view(earlier).zone, Zone::Graveyard);
    cast(&mut game, spell, Some(Target::Card(victim)));
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn eden_mills_before_its_reflexive_trigger_and_cannot_return_itself() {
    let player = Player::P0;
    let mut game = scenario(player);
    let land = put(&mut game, player, "Eden, Seat of the Sanctum", Zone::Hand);
    let instant = put(&mut game, player, "Shock", Zone::Library);
    let permanent = put(&mut game, player, "Grizzly Bears", Zone::Library);
    act(&mut game, Action::PlayLand { card: land });
    let mut tap = game.clone();
    tap.state.players[player.index()].mana = [0; 6];
    let colorless = [0, 0, 0, 0, 0, 1];
    act(&mut tap, mana(land, colorless));
    assert_eq!(tap.state.players[player.index()].mana, colorless);

    act(&mut game, announce_ability(land, 1));
    pay(&mut game);
    let sacrifice = Action::ChooseOptional { accept: true };
    reach(&mut game, |a| *a == sacrifice);
    assert_eq!(game.object_view(permanent).zone, Zone::Graveyard);
    assert_eq!(game.object_view(land).zone, Zone::Battlefield);
    act(&mut game, sacrifice);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    reject(&mut game, target(land));
    reject(&mut game, target(instant));
    act(&mut game, target(permanent));
    settle(&mut game);
    assert_eq!(game.object_view(permanent).zone, Zone::Hand);
}

#[test]
fn elixir_enters_tapped_exiles_as_cost_and_gains_life_for_nonlands_only() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Elixir", Zone::Hand);
    let spell = put(&mut game, player, "Shock", Zone::Graveyard);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Graveyard);
    let land = put(&mut game, player, "Forest", Zone::Graveyard);
    let life = game.state.players[player.index()].life;
    cast(&mut game, artifact, None);
    settle(&mut game);
    assert!(game.object_view(artifact).tapped);
    reject(&mut game, announce_ability(artifact, 0));
    game.state.objects[artifact as usize].tapped = false;
    act(&mut game, announce_ability(artifact, 0));
    pay(&mut game);
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    assert_eq!(game.object_view(spell).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.object_view(spell).zone, Zone::Library);
    assert_eq!(game.object_view(creature).zone, Zone::Library);
    assert_eq!(game.object_view(land).zone, Zone::Graveyard);
    assert_eq!(game.state.players[player.index()].life, life + 2);
}

#[test]
fn ether_funds_the_next_spell_and_its_copy_can_have_a_different_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let artifact = put(&mut game, player, "Ether", Zone::Hand);
    let first = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let second = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    let spell = put(&mut game, player, "Unsummon", Zone::Hand);
    cast(&mut game, artifact, None);
    settle(&mut game);
    game.state.players[player.index()].mana = [0; 6];

    let blue = [0, 1, 0, 0, 0, 0];
    act(&mut game, mana(artifact, blue));
    assert_eq!(game.object_view(artifact).zone, Zone::Exile);
    assert_eq!(game.state.players[player.index()].mana, blue);
    assert!(game.state.stack.is_empty());
    cast(&mut game, spell, Some(Target::Card(first)));
    let retarget = Action::SetStackTarget {
        index: 0,
        target: Target::Card(second),
    };
    reach(&mut game, |a| *a == retarget);
    act(&mut game, retarget);
    act(&mut game, Action::FinishStackTargets);
    settle(&mut game);
    assert_eq!(game.object_view(first).zone, Zone::Hand);
    assert_eq!(game.object_view(second).zone, Zone::Hand);
    assert_eq!(game.state.players[player.index()].spells_cast_this_turn, 2);
}

#[test]
fn evil_reawakened_adds_two_counters_to_the_creatures_normal_entry() {
    let player = Player::P0;
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Evil Reawakened", Zone::Hand);
    let creature = put(&mut game, player, "District Mascot", Zone::Graveyard);
    let foreign = put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);

    act(&mut game, Action::AnnounceCast { card: spell });
    reject(&mut game, target(foreign));
    act(&mut game, target(creature));
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(creature).zone, Zone::Battlefield);
    assert!(!game.object_view(creature).tapped);
    assert_eq!(game.object_view(creature).counters.plus_one, 3);
}

fn pass_pair(game: &mut Game) {
    act(game, Action::Pass);
    act(game, Action::Pass);
}

fn attack(game: &mut Game, attacker: CardId) {
    reach(game, |action| *action == Action::Attack { card: attacker });
    act(game, Action::Attack { card: attacker });
    act(game, Action::FinishAttackers);
    settle(game);
    pass_pair(game);
    assert_eq!(game.state.pending, Pending::Blockers);
}

fn equip(game: &mut Game, equipment: CardId, creature: CardId) {
    act(game, announce_ability(equipment, 0));
    act(game, target(creature));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(game);
    }
    settle(game);
}

#[test]
fn a_realm_reborn_grants_other_permanents_fundable_colored_mana() {
    let player = Player::P0;
    let mut game = scenario(player);
    let realm = put(&mut game, player, "A Realm Reborn", Zone::Hand);
    let recipient = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let creature = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[creature as usize].summoning_sick = true;
    let opposing = put(
        &mut game,
        player.other(),
        "Swiftfoot Boots",
        Zone::Battlefield,
    );
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, realm, None);
    settle(&mut game);
    game.state.players[player.index()].mana = [0; 6];

    let grantor = game.object_ref(realm);
    let tap = |card| Action::TapGrantedMana {
        card,
        grantor,
        production: [0, 0, 0, 1, 0, 0],
    };
    for illegal in [realm, creature, opposing] {
        reject(&mut game, tap(illegal));
    }
    act(&mut game, Action::AnnounceCast { card: shock });
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player.other()),
        },
    );
    act(&mut game, tap(recipient));
    pay(&mut game);
    settle(&mut game);
    assert!(game.object_view(recipient).tapped);
    assert_eq!(game.state.players[player.other().index()].life, 18);
}

#[test]
fn absolute_virtue_resists_a_counter_and_protects_its_controller() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let virtue = put(&mut game, player, "Absolute Virtue", Zone::Hand);
    let cancel = put(&mut game, opponent, "Cancel", Zone::Hand);
    let shock = put(&mut game, opponent, "Shock", Zone::Hand);
    cast(&mut game, virtue, None);
    act(&mut game, Action::Pass);
    cast(&mut game, cancel, Some(Target::Spell(virtue)));
    settle(&mut game);
    assert_eq!(game.object_view(virtue).zone, Zone::Battlefield);
    assert!(game.keywords(virtue).contains(Keywords::FLYING));

    if game.actor() != Some(opponent) {
        act(&mut game, Action::Pass);
    }
    act(&mut game, Action::AnnounceCast { card: shock });
    reject(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(player),
        },
    );
    act(
        &mut game,
        Action::ChooseTarget {
            target: Target::Player(opponent),
        },
    );
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.state.players[opponent.index()].life, 18);
}

#[test]
fn aerith_rescue_mission_creates_heroes_or_stuns_one_surviving_stair_target() {
    let player = Player::P0;
    let mut game = scenario(player);
    let mission = put(&mut game, player, "Aerith Rescue Mission", Zone::Hand);
    let first = put(
        &mut game,
        player.other(),
        "Grizzly Bears",
        Zone::Battlefield,
    );
    let removed = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);

    act(&mut game, Action::AnnounceCast { card: mission });
    let mut elevator = game.clone();
    act(&mut elevator, Action::ChooseMode { mode: 0 });
    pay(&mut elevator);
    settle(&mut elevator);
    let heroes: Vec<_> = elevator
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == Card(5328))
        .collect();
    assert_eq!(heroes.len(), 3);
    for hero in heroes {
        assert_eq!(hero.colors, 0);
        assert_eq!(stats(&elevator, hero.id), (1, 1));
    }

    act(&mut game, Action::ChooseMode { mode: 1 });
    act(&mut game, target(first));
    act(&mut game, target(removed));
    act(&mut game, Action::FinishTargets);
    pay(&mut game);
    game.move_object(removed, Zone::Exile);
    let choice = Action::ChooseEffectObject {
        object: game.object_ref(first),
    };
    reach(&mut game, |action| *action == choice);
    assert!(game.object_view(first).tapped);
    let invalid = Action::ChooseEffectObject {
        object: game.object_ref(removed),
    };
    reject(&mut game, invalid);
    act(&mut game, choice);
    assert_eq!(game.object_view(first).counters.stun, 1);
}

#[test]
fn blitzball_draw_activation_requires_an_opponent_hit_by_a_legendary_creature() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ball = put(&mut game, player, "Blitzball", Zone::Hand);
    let attacker = put(&mut game, player, "Adelbert Steiner", Zone::Battlefield);
    cast(&mut game, ball, None);
    settle(&mut game);
    reject(&mut game, announce_ability(ball, 1));
    let mut mana_branch = game.clone();
    mana_branch.state.players[player.index()].mana = [0; 6];
    let green = [0, 0, 0, 0, 1, 0];
    act(&mut mana_branch, mana(ball, green));
    assert_eq!(mana_branch.state.players[player.index()].mana, green);

    attack(&mut game, attacker);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[player.other().index()].life, 18);
    choose(
        &mut game,
        |action| matches!(action, Action::Activate { card, ability: 1, .. } if *card == ball),
    );
    assert_eq!(game.object_view(ball).zone, Zone::Graveyard);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
}

#[test]
fn cactuar_returns_on_a_later_own_end_step_but_stays_on_its_entry_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cactuar = put(&mut game, player, "Cactuar", Zone::Hand);
    cast(&mut game, cactuar, None);
    settle(&mut game);
    assert!(game.keywords(cactuar).contains(Keywords::TRAMPLE));
    for active in [player, player.other(), player] {
        assert_eq!(game.state.active, active);
        game.state.step = Step::Main2;
        pass_pair(&mut game);
        assert_eq!(game.state.step, Step::End);
        if active == player && game.state.turn > 3 {
            assert_eq!(game.state.stack.len(), 1);
            settle(&mut game);
            assert_eq!(game.object_view(cactuar).zone, Zone::Hand);
        } else {
            assert!(game.state.stack.is_empty());
            assert_eq!(game.object_view(cactuar).zone, Zone::Battlefield);
            pass_pair(&mut game);
        }
    }
}

#[test]
fn cargo_ship_crews_for_one_and_its_colorless_mana_funds_equip() {
    let player = Player::P0;
    let mut game = scenario(player);
    let ship = put(&mut game, player, "Cargo Ship", Zone::Hand);
    let pilot = put(&mut game, player, "Llanowar Elves", Zone::Battlefield);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let giant = put(&mut game, player, "Hill Giant", Zone::Hand);
    cast(&mut game, ship, None);
    settle(&mut game);

    let mut crewed = game.clone();
    act(&mut crewed, announce_ability(ship, 1));
    if matches!(crewed.state.pending, Pending::PayCosts { .. }) {
        pay(&mut crewed);
    }
    let object = crewed.object_ref(pilot);
    act(&mut crewed, Action::ChooseCrewer { object });
    act(&mut crewed, Action::FinishCrew);
    settle(&mut crewed);
    assert!(crewed.object_view(pilot).tapped);
    assert!(crewed.object_view(ship).is_creature());
    assert_eq!(stats(&crewed, ship), (2, 3));
    assert!(
        crewed
            .keywords(ship)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );

    let colorless = [0, 0, 0, 0, 0, 1];
    let mut restricted = game.clone();
    restricted.state.objects[pilot as usize].tapped = true;
    restricted.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
    act(&mut restricted, mana(ship, colorless));
    reject(&mut restricted, Action::AnnounceCast { card: giant });

    game.state.players[player.index()].mana = [0; 6];
    act(&mut game, mana(ship, colorless));
    equip(&mut game, boots, pilot);
    assert_eq!(game.state.players[player.index()].mana, [0; 6]);
    assert_eq!(
        game.object_view(boots).attached_to,
        Some(game.object_ref(pilot))
    );
}

#[test]
fn clouds_exact_equip_target_gets_a_discount_and_bonuses_only_on_its_turn() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cloud = put(&mut game, player, "Cloud, Planet's Champion", Zone::Hand);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let other = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let bonuses = Keywords::DOUBLE_STRIKE.union(Keywords::INDESTRUCTIBLE);
    cast(&mut game, cloud, None);
    settle(&mut game);
    assert!(!game.keywords(cloud).contains(bonuses));
    game.state.players[player.index()].mana = [0; 6];

    act(&mut game, announce_ability(boots, 0));
    reject(&mut game, target(other));
    act(&mut game, target(cloud));
    if matches!(game.state.pending, Pending::PayCosts { .. }) {
        pay(&mut game);
    }
    settle(&mut game);
    assert_eq!(
        game.object_view(boots).attached_to,
        Some(game.object_ref(cloud))
    );
    assert!(game.keywords(cloud).contains(bonuses));
    game.state.active = player.other();
    assert!(!game.keywords(cloud).contains(bonuses));
}

#[test]
fn diamond_weapon_discounts_only_permanents_and_prevents_combat_damage() {
    let player = Player::P0;
    let mut game = scenario(player);
    let weapon = put(&mut game, player, "Diamond Weapon", Zone::Hand);
    put(&mut game, player, "Forest", Zone::Graveyard);
    put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    put(&mut game, player, "Shock", Zone::Graveyard);
    put(&mut game, player.other(), "Grizzly Bears", Zone::Graveyard);

    game.state.players[player.index()].mana = [0, 0, 0, 0, 2, 4];
    reject(&mut game, Action::AnnounceCast { card: weapon });
    game.state.players[player.index()].mana = [0, 0, 0, 1, 2, 5];
    cast(&mut game, weapon, None);
    settle(&mut game);
    assert_eq!(stats(&game, weapon), (8, 8));
    assert!(game.keywords(weapon).contains(Keywords::REACH));
    game.state.players[player.index()].mana = [0, 0, 0, 1, 0, 0];
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Card(weapon)));
    settle(&mut game);
    assert_eq!(game.object_view(weapon).damage, 2);

    game.state.active = player.other();
    game.state.priority = player.other();
    let attacker = put(&mut game, player.other(), "Hill Giant", Zone::Battlefield);
    attack(&mut game, attacker);
    act(
        &mut game,
        Action::Block {
            blocker: weapon,
            attacker,
        },
    );
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.object_view(weapon).damage, 2);
    assert_eq!(game.object_view(attacker).zone, Zone::Graveyard);
}

#[test]
fn excalibur_tracks_gain_events_and_carries_its_live_bonus_to_new_bearers() {
    let player = Player::P0;
    let mut game = scenario(player);
    let blade = put(&mut game, player, "Excalibur II", Zone::Hand);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    cast(&mut game, blade, None);
    settle(&mut game);
    equip(&mut game, blade, first);
    assert_eq!(stats(&game, first), (2, 2));

    let inn = put(&mut game, player, "Adventurer's Inn", Zone::Hand);
    act(&mut game, Action::PlayLand { card: inn });
    settle(&mut game);
    assert_eq!(game.object_view(blade).counters.get(CounterKind::Charge), 1);
    assert_eq!(stats(&game, first), (3, 3));

    let menu = put(&mut game, player, "Battle Menu", Zone::Hand);
    act(&mut game, Action::AnnounceCast { card: menu });
    act(&mut game, Action::ChooseMode { mode: 3 });
    pay(&mut game);
    settle(&mut game);
    assert_eq!(game.object_view(blade).counters.get(CounterKind::Charge), 2);

    equip(&mut game, blade, second);
    assert_eq!(stats(&game, first), (2, 2));
    assert_eq!(stats(&game, second), (5, 5));
}

#[test]
fn adamantoise_redirects_damage_retains_it_and_creates_tapped_treasures_on_death() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let turtle = put(&mut game, player, "Ancient Adamantoise", Zone::Hand);
    cast(&mut game, turtle, None);
    settle(&mut game);
    assert!(game.keywords(turtle).contains(Keywords::VIGILANCE));

    let redirected = put(&mut game, opponent, "Shock", Zone::Hand);
    act(&mut game, Action::Pass);
    cast(&mut game, redirected, Some(Target::Player(player)));
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.object_view(turtle).damage, 2);
    game.state.step = Step::End;
    game.state.priority = player;
    pass_pair(&mut game);
    assert_eq!(game.object_view(turtle).damage, 2);

    let direct = put(&mut game, opponent, "Shock", Zone::Hand);
    game.state.players[opponent.index()].mana = [8; 6];
    assert_eq!(game.actor(), Some(opponent));
    cast(&mut game, direct, Some(Target::Card(turtle)));
    reach(&mut game, |action| *action == Action::DeclineResolutionMana);
    act(&mut game, Action::DeclineResolutionMana);
    settle(&mut game);
    assert_eq!(game.object_view(turtle).damage, 2);

    if game.actor() != Some(player) {
        act(&mut game, Action::Pass);
    }
    game.state.players[player.index()].mana = [8; 6];
    let murder = put(&mut game, player, "Murder", Zone::Hand);
    cast(&mut game, murder, Some(Target::Card(turtle)));
    settle(&mut game);
    assert_eq!(game.object_view(turtle).zone, Zone::Exile);
    let treasures: Vec<_> = game
        .observe(player)
        .objects
        .into_iter()
        .filter(|object| object.zone == Zone::Battlefield && object.card == Card(5302))
        .collect();
    assert_eq!(treasures.len(), 10);
    assert!(treasures.iter().all(|treasure| treasure.tapped));
}

#[test]
fn barret_counts_each_equipped_creature_once_at_resolution() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let barret = put(&mut game, player, "Barret Wallace", Zone::Battlefield);
    let first = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Battlefield);
    let bow = put(&mut game, player, "Short Bow", Zone::Battlefield);
    equip(&mut game, boots, first);
    equip(&mut game, bow, first);
    assert!(game.keywords(barret).contains(Keywords::REACH));
    reach(&mut game, |a| *a == Action::Attack { card: barret });
    act(&mut game, Action::Attack { card: barret });
    act(&mut game, Action::FinishAttackers);
    let mut moved = game.clone();
    assert!(moved.attach_equipment(moved.object_ref(bow), moved.object_ref(second)));
    settle(&mut game);
    settle(&mut moved);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    assert_eq!(moved.state.players[opponent.index()].life, 18);
}

#[test]
fn beatrix_can_attach_any_number_of_controlled_equipment_without_targeting_them() {
    let player = Player::P0;
    let mut game = scenario(player);
    let beatrix = put(
        &mut game,
        player,
        "Beatrix, Loyal General",
        Zone::Battlefield,
    );
    let recipient = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let first = put(&mut game, player, "Short Bow", Zone::Battlefield);
    let second = put(&mut game, player, "Gold Pan", Zone::Battlefield);
    let foreign = put(&mut game, player.other(), "Short Bow", Zone::Battlefield);
    game.state.objects[first as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
    assert!(game.keywords(beatrix).contains(Keywords::VIGILANCE));

    reach(&mut game, |a| *a == target(recipient));
    act(&mut game, target(recipient));
    let pick = |equipment| Action::ChooseEquipmentAttachment {
        equipment: Some(equipment),
    };
    let [first, second, foreign] = [first, second, foreign].map(|id| game.object_ref(id));
    let finish = Action::ChooseEquipmentAttachment { equipment: None };
    reach(&mut game, |a| *a == pick(first));
    reject(&mut game, pick(foreign));
    let mut declined = game.clone();
    act(&mut declined, finish.clone());
    assert_eq!(declined.object_view(first.card).attached_to, None);
    act(&mut game, pick(first));
    act(&mut game, pick(second));
    act(&mut game, finish);
    if game.observe(player).equipment_attachment.is_some() {
        act(&mut game, pick(first));
    }
    for equipment in [first, second] {
        assert_eq!(
            game.object_view(equipment.card).attached_to,
            Some(game.object_ref(recipient))
        );
    }
}

#[test]
fn buster_sword_uses_combat_damage_and_can_cast_the_card_just_drawn() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let sword = put(&mut game, player, "Buster Sword", Zone::Hand);
    let bearer = put(&mut game, player, "Grizzly Bears", Zone::Battlefield);
    let expensive = put(&mut game, player, "Meteor Golem", Zone::Hand);
    let drawn = put(&mut game, player, "Hill Giant", Zone::Library);
    cast(&mut game, sword, None);
    settle(&mut game);
    equip(&mut game, sword, bearer);
    assert_eq!(stats(&game, bearer), (5, 4));
    attack(&mut game, bearer);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    assert_eq!(game.state.players[opponent.index()].life, 15);

    let free = |object| Action::CastFreeDuringResolution {
        object,
        face: CastFace::Normal,
    };
    reach(
        &mut game,
        |a| matches!(a, Action::CastFreeDuringResolution { object, face: CastFace::Normal } if object.card == drawn),
    );
    let drawn_action = free(game.object_ref(drawn));
    assert_eq!(game.object_view(drawn).zone, Zone::Hand);
    assert!(game.observe(opponent).resolution_free_casts.is_none());
    let expensive_action = free(game.object_ref(expensive));
    reject(&mut game, expensive_action);
    act(&mut game, drawn_action);
    pay(&mut game);
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(drawn).zone, Zone::Battlefield);
}

fn tokens(game: &Game, player: Player, definition: Card) -> Vec<CardId> {
    game.observe(player)
        .objects
        .into_iter()
        .filter(|object| {
            object.controller == player
                && object.zone == Zone::Battlefield
                && object.card == definition
        })
        .map(|object| object.id)
        .collect()
}

#[test]
fn circle_of_power_boosts_existing_and_new_wizards_with_lifelink() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    game.create_tokens(player, Card(5531), 1, false);
    game.create_tokens(opponent, Card(5531), 1, false);
    let spell = put(&mut game, player, "Circle of Power", Zone::Hand);
    cast(&mut game, spell, None);
    settle(&mut game);
    assert_eq!(game.state.players[player.index()].cards_drawn_this_turn, 2);
    assert_eq!(game.state.players[player.index()].life, 18);
    assert_eq!(game.state.players[opponent.index()].life, 19);
    let wizards = tokens(&game, player, Card(5531));
    assert_eq!(wizards.len(), 2);
    for wizard in wizards {
        assert_eq!(stats(&game, wizard), (1, 1));
        assert!(game.keywords(wizard).contains(Keywords::LIFELINK));
    }
    let opposing_wizard = tokens(&game, opponent, Card(5531))[0];
    assert_eq!(stats(&game, opposing_wizard), (0, 1));
    let shock = put(&mut game, player, "Shock", Zone::Hand);
    cast(&mut game, shock, Some(Target::Player(opponent)));
    settle_triggers(&mut game);
    assert_eq!(game.state.players[player.index()].life, 20);
    assert_eq!(game.state.players[opponent.index()].life, 15);
}

fn settle_triggers(game: &mut Game) {
    for _ in 0..48 {
        if game.state.pending == Pending::Priority
            && game.state.pending_triggers.is_empty()
            && game.state.ordering_triggers.is_empty()
            && game.state.stack.is_empty()
            && game.state.resolving.is_none()
        {
            return;
        }
        choose(game, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
        });
    }
    panic!("triggers did not resolve");
}

#[test]
fn clives_hideaway_checks_legendary_count_when_the_activation_resolves() {
    let player = Player::P0;
    let mut game = scenario(player);
    let hideaway = put(&mut game, player, "Clive's Hideaway", Zone::Hand);
    let hidden = put(&mut game, player, "Hill Giant", Zone::Library);
    for legend in [
        "Yargle, Glutton of Urborg",
        "Kellan, Planar Trailblazer",
        "Alesha, Who Laughs at Fate",
    ] {
        put(&mut game, player, legend, Zone::Battlefield);
    }
    act(&mut game, Action::PlayLand { card: hideaway });
    assert!(!game.object_view(hideaway).tapped);
    let select = Action::SelectLibrary { card: hidden };
    reach(&mut game, |a| *a == select);
    act(&mut game, select);
    if game
        .legal_actions()
        .contains(&Action::FinishLibrarySelection)
    {
        act(&mut game, Action::FinishLibrarySelection);
    }
    settle(&mut game);
    assert_eq!(game.object_view(hidden).zone, Zone::Exile);
    assert!(!game.can_view_hideaway(player.other(), game.object_ref(hidden)));
    let mut tap = game.clone();
    tap.state.players[player.index()].mana = [0; 6];
    act(&mut tap, mana(hideaway, [0, 0, 0, 0, 0, 1]));
    assert_eq!(tap.state.players[player.index()].mana[5], 1);

    act(&mut game, announce_ability(hideaway, 1));
    pay(&mut game);
    let mut three = game.clone();
    settle(&mut three);
    assert_eq!(three.object_view(hidden).zone, Zone::Exile);
    put(
        &mut game,
        player,
        "Tatyova, Benthic Druid",
        Zone::Battlefield,
    );
    let free = Action::CastExiledDuringResolution {
        object: game.object_ref(hidden),
        face: CastFace::Normal,
    };
    reach(&mut game, |a| *a == free);
    act(&mut game, free);
    pay(&mut game);
    if game
        .legal_actions()
        .contains(&Action::FinishResolutionCasts)
    {
        act(&mut game, Action::FinishResolutionCasts);
    }
    settle(&mut game);
    assert_eq!(game.object_view(hidden).zone, Zone::Battlefield);
}

#[test]
fn midgar_cloud_tutors_equipment_and_doubles_the_attached_swords_trigger() {
    let player = Player::P0;
    let mut game = scenario(player);
    let cloud = put(&mut game, player, "Cloud, Midgar Mercenary", Zone::Hand);
    let sword = put(
        &mut game,
        player,
        "Sword of Wealth and Power",
        Zone::Library,
    );
    cast(&mut game, cloud, None);
    let find = Action::ChooseSearch { card: sword };
    reach(&mut game, |a| *a == find);
    act(&mut game, find);
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert_eq!(game.object_view(sword).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(sword))
    );
    cast(&mut game, sword, None);
    settle(&mut game);
    equip(&mut game, sword, cloud);

    game.state.objects[cloud as usize].summoning_sick = false;
    attack(&mut game, cloud);
    act(&mut game, Action::FinishBlockers);
    pass_pair(&mut game);
    settle_triggers(&mut game);
    assert_eq!(tokens(&game, player, Card(5302)).len(), 2);
}

#[test]
fn cornered_by_black_mages_requires_the_opponents_sacrifice_and_still_creates_a_wizard() {
    let player = Player::P0;
    let opponent = player.other();
    let mut game = scenario(player);
    let spell = put(&mut game, player, "Cornered by Black Mages", Zone::Hand);
    let victim = put(&mut game, opponent, "Grizzly Bears", Zone::Battlefield);
    game.state.objects[victim as usize].gained_keywords = Keywords::PROTECTION_EVERYTHING;
    let owned = put(&mut game, player, "Hill Giant", Zone::Battlefield);
    cast(&mut game, spell, Some(Target::Player(opponent)));
    let mut empty = game.clone();
    empty.move_object(victim, Zone::Exile);
    settle(&mut empty);
    assert_eq!(tokens(&empty, player, Card(5531)).len(), 1);
    let sacrifice = Action::Sacrifice {
        object: game.object_ref(victim),
    };
    reach(&mut game, |a| *a == sacrifice);
    assert_eq!(game.actor(), Some(opponent));
    let illegal = Action::Sacrifice {
        object: game.object_ref(owned),
    };
    reject(&mut game, illegal);
    act(&mut game, sacrifice);
    settle(&mut game);
    assert_eq!(game.object_view(victim).zone, Zone::Graveyard);
    assert_eq!(tokens(&game, player, Card(5531)).len(), 1);
}

#[test]
fn delivery_moogle_searches_one_qualified_card_and_reveals_the_public_result() {
    let player = Player::P0;
    let mut game = scenario(player);
    let source = put(&mut game, player, "Delivery Moogle", Zone::Hand);
    let boots = put(&mut game, player, "Swiftfoot Boots", Zone::Graveyard);
    let too_expensive = put(&mut game, player, "Meteor Golem", Zone::Graveyard);
    let before = game.state.players[player.index()].library.clone();
    cast(&mut game, source, None);
    let graveyard = Action::ChooseSearchZones {
        library: false,
        graveyard: true,
    };
    reach(&mut game, |a| *a == graveyard);
    reject(
        &mut game,
        Action::ChooseSearchZones {
            library: false,
            graveyard: false,
        },
    );
    act(&mut game, graveyard);
    reject(&mut game, Action::FinishSearch);
    reject(
        &mut game,
        Action::ChooseSearch {
            card: too_expensive,
        },
    );
    act(&mut game, Action::ChooseSearch { card: boots });
    act(&mut game, Action::FinishSearch);
    settle(&mut game);
    assert!(game.keywords(source).contains(Keywords::FLYING));
    assert_eq!(game.state.players[player.index()].library, before);
    assert_eq!(game.object_view(boots).zone, Zone::Hand);
    assert!(
        game.observe(player.other())
            .known_hand
            .contains(&game.object_ref(boots))
    );
}
