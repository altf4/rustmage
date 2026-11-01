//! Shared rules use representative cards, rather than every card with a keyword.
use super::*;

fn blockers(g: &mut Game) {
    for _ in 0..8 {
        if g.state.pending == Pending::Blockers {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("combat did not reach blockers");
}

fn combat(g: &mut Game, attacker: CardId, blocker: Option<CardId>) {
    attack(g, attacker);
    blockers(g);
    if let Some(blocker) = blocker {
        act(g, Action::Block { blocker, attacker });
    }
    act(g, Action::FinishBlockers);
    for _ in 0..32 {
        if g.state.step == Step::Main2 {
            return;
        }
        if g.state.pending == Pending::Priority {
            act(g, Action::Pass);
        } else {
            choose(g, |a| !matches!(a, Action::Concede));
        }
    }
    panic!("combat exceeded 32 actions");
}

#[test]
fn flying_reach_and_vigilance() {
    let mut g = game();
    let angel = put(&mut g, Player::P0, "Serra Angel", Zone::Battlefield);
    let ground = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let reach = put(&mut g, Player::P1, "Thornweald Archer", Zone::Battlefield);
    attack(&mut g, angel);
    assert!(!g.object_view(angel).tapped);
    blockers(&mut g);
    assert!(!g.legal_actions().contains(&Action::Block {
        blocker: ground,
        attacker: angel
    }));
    act(
        &mut g,
        Action::Block {
            blocker: reach,
            attacker: angel,
        },
    );
}

#[test]
fn deathtouch_and_lifelink() {
    let mut g = game();
    let hawk = put(&mut g, Player::P0, "Vampire Nighthawk", Zone::Battlefield);
    let angel = put(&mut g, Player::P1, "Serra Angel", Zone::Battlefield);
    combat(&mut g, hawk, Some(angel));
    assert_eq!(g.object_view(hawk).zone, Zone::Graveyard);
    assert_eq!(g.object_view(angel).zone, Zone::Graveyard);
    assert_eq!(g.state.players[0].life, 22);
}

#[test]
fn first_strike_kills_before_normal_damage_and_double_strike_deals_twice() {
    let mut g = game();
    let knight = put(&mut g, Player::P0, "Brightblade Stoat", Zone::Battlefield);
    let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    combat(&mut g, knight, Some(bear));
    assert_eq!(g.object_view(knight).zone, Zone::Battlefield);
    assert_eq!(g.object_view(knight).damage, 0);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);

    let mut g = game();
    let striker = put(
        &mut g,
        Player::P0,
        "Swiftblade Vindicator",
        Zone::Battlefield,
    );
    combat(&mut g, striker, None);
    assert_eq!(g.state.players[1].life, 18);
}

#[test]
fn trample_assigns_lethal_damage_then_overflows_to_the_defender() {
    let mut g = game();
    let attacker = put(&mut g, Player::P0, "Tajuru Pathwarden", Zone::Battlefield);
    let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    attack(&mut g, attacker);
    blockers(&mut g);
    act(
        &mut g,
        Action::Block {
            blocker: bear,
            attacker,
        },
    );
    act(&mut g, Action::FinishBlockers);
    for _ in 0..8 {
        if g.state.pending == Pending::Damage {
            break;
        }
        act(&mut g, Action::Pass);
    }
    act(
        &mut g,
        Action::AssignDamage {
            target: Target::Card(bear),
            amount: 2,
        },
    );
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.state.players[1].life, 17);
}

#[test]
fn menace_needs_two_blockers() {
    let mut g = game();
    let attacker = put(&mut g, Player::P0, "Ravine Raider", Zone::Battlefield);
    let a = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let b = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    attack(&mut g, attacker);
    blockers(&mut g);
    act(
        &mut g,
        Action::Block {
            blocker: a,
            attacker,
        },
    );
    assert!(!g.legal_actions().contains(&Action::FinishBlockers));
    act(
        &mut g,
        Action::Block {
            blocker: b,
            attacker,
        },
    );
    act(&mut g, Action::FinishBlockers);
}

#[test]
fn indestructible_survives_destruction() {
    let mut g = game();
    let target = put(
        &mut g,
        Player::P0,
        "Zetalpa, Primal Dawn",
        Zone::Battlefield,
    );
    let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, murder, &[Target::Card(target)]);
    resolve(&mut g);
    assert_eq!(g.object_view(target).zone, Zone::Battlefield);
    assert_eq!(g.object_view(murder).zone, Zone::Graveyard);
}

#[test]
fn legend_rule_keeps_one_permanent_of_the_chosen_name() {
    let mut g = game();
    let first = put(
        &mut g,
        Player::P0,
        "Caelorna, Coral Tyrant",
        Zone::Battlefield,
    );
    let second = put(
        &mut g,
        Player::P0,
        "Caelorna, Coral Tyrant",
        Zone::Battlefield,
    );
    g.state_based_actions();
    let object = g.object_ref(first);
    act(&mut g, Action::KeepLegend { object });
    assert_eq!(g.object_view(first).zone, Zone::Battlefield);
    assert_eq!(g.object_view(second).zone, Zone::Graveyard);
}

#[test]
fn defender_prevents_attacking() {
    let mut g = game();
    let wall = put(&mut g, Player::P0, "Monoist Sentry", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    assert!(!g.legal_actions().contains(&Action::Attack { card: wall }));
}

#[test]
fn flash_allows_casting_during_the_opponents_turn() {
    let mut g = game();
    g.state.active = Player::P1;
    let normal = put(&mut g, Player::P0, "Bear Cub", Zone::Hand);
    assert!(
        !g.legal_actions()
            .iter()
            .any(|a| matches!(a, Action::AnnounceCast {card} if *card==normal))
    );
    let flash = put(&mut g, Player::P0, "Ambush Wolf", Zone::Hand);
    cast(&mut g, flash, &[]);
    // Entry has a card-specific target choice; casting permission is the shared rule.
    for _ in 0..8 {
        if g.on_battlefield(flash) {
            return;
        }
        act(&mut g, Action::Pass);
    }
    panic!("flash creature did not enter");
}

#[test]
fn hexproof_prevents_an_opponents_targeting_but_allows_its_controller() {
    let mut g = game();
    let sphinx = put(
        &mut g,
        Player::P1,
        "Sphinx of the Final Word",
        Zone::Battlefield,
    );
    put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
    act(&mut g, Action::AnnounceCast { card: murder });
    assert!(!g.legal_actions().contains(&Action::ChooseTarget {
        target: Target::Card(sphinx)
    }));
    let mut g = game();
    let sphinx = put(
        &mut g,
        Player::P0,
        "Sphinx of the Final Word",
        Zone::Battlefield,
    );
    let murder = put(&mut g, Player::P0, "Murder", Zone::Hand);
    cast(&mut g, murder, &[Target::Card(sphinx)]);
    resolve(&mut g);
    assert_eq!(g.object_view(sphinx).zone, Zone::Graveyard);
}

#[test]
fn ward_accepts_payment_or_counters_the_targeting_spell() {
    for paid in [false, true] {
        let mut g = game();
        let terror = put(&mut g, Player::P1, "Tolarian Terror", Zone::Battlefield);
        let shock = put(&mut g, Player::P0, "Shock", Zone::Hand);
        cast(&mut g, shock, &[Target::Card(terror)]);
        reach(&mut g, &Action::DeclineResolutionMana);
        if paid {
            let before: u32 = g.state.players[0].mana.iter().sum();
            choose(&mut g, |a| matches!(a, Action::PayResolutionMana { .. }));
            assert_eq!(before - g.state.players[0].mana.iter().sum::<u32>(), 2);
        } else {
            act(&mut g, Action::DeclineResolutionMana);
        }
        resolve(&mut g);
        assert_eq!(g.object_view(shock).zone, Zone::Graveyard);
        assert_eq!(g.object_view(terror).damage, if paid { 2 } else { 0 });
    }
}

#[test]
fn changeling_grants_creature_types() {
    let mut g = game();
    let card = put(&mut g, Player::P0, "Barkform Harvester", Zone::Battlefield);
    assert!(g.has_subtype(card, "Elf"));
    assert!(g.has_subtype(card, "Dragon"));
    assert!(!g.has_subtype(card, "Equipment"));
}

#[test]
fn haste_and_summoning_sickness() {
    let mut g = game();
    let fast = put(&mut g, Player::P0, "Brazen Scourge", Zone::Battlefield);
    let slow = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    g.state.objects[fast as usize].summoning_sick = true;
    g.state.objects[slow as usize].summoning_sick = true;
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    assert!(!g.legal_actions().contains(&Action::Attack { card: slow }));
    act(&mut g, Action::Attack { card: fast });
}

#[test]
fn stack_response_resolves_before_the_original_spell() {
    let mut g = game();
    let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Battlefield);
    let shock = put(&mut g, Player::P0, "Shock", Zone::Hand);
    cast(&mut g, shock, &[Target::Card(bear)]);
    act(&mut g, Action::Pass);
    let growth = put(&mut g, Player::P1, "Giant Growth", Zone::Hand);
    cast(&mut g, growth, &[Target::Card(bear)]);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
    assert_eq!(stats(&g, bear), (5, 5));
    g.finish_cleanup();
    assert_eq!(stats(&g, bear), (2, 2));
}

#[test]
fn cant_block_prevents_block_declaration() {
    let mut g = game();
    let bear = put(&mut g, Player::P0, "Bear Cub", Zone::Battlefield);
    let blocker = put(&mut g, Player::P1, "Bloodghast", Zone::Battlefield);
    attack(&mut g, bear);
    reach(&mut g, &Action::FinishBlockers);
    assert!(!g.legal_actions().contains(&Action::Block {
        blocker,
        attacker: bear
    }));
}

#[test]
fn counterspell_counters_a_spell_on_the_stack() {
    let mut g = game();
    g.state.active = Player::P1;
    g.state.priority = Player::P1;
    let bear = put(&mut g, Player::P1, "Bear Cub", Zone::Hand);
    cast(&mut g, bear, &[]);
    act(&mut g, Action::Pass);
    let counter = put(&mut g, Player::P0, "Counterspell", Zone::Hand);
    g.state.players[0].mana = [0, 2, 0, 0, 0, 0];
    cast(&mut g, counter, &[Target::Spell(bear)]);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    resolve(&mut g);
    assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
    assert_eq!(g.object_view(counter).zone, Zone::Graveyard);
}
