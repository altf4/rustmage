//! Paid card scenarios for Edge of Eternities collectors 081–094.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BLACK: Mana = [0, 0, 1, 0, 0, 0];

fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if ready(g) {
            return;
        }
        if let Some(range) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: range.target,
                    amount: range.maximum,
                },
            );
        } else {
            choose(g, |a| {
                matches!(
                    a,
                    Action::Pass
                        | Action::PutTrigger { .. }
                        | Action::FinishAttackers
                        | Action::FinishBlockers
                )
            });
        }
    }
    panic!(
        "advance budget exhausted at {:?} {:?}",
        g.state.step, g.state.pending
    );
}

fn quiet(g: &mut Game) {
    g.prepare_triggers();
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn finish_targets(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

fn attack(g: &mut Game, card: CardId) {
    until(g, |g| g.state.pending == Pending::Attackers);
    act(g, Action::Attack { card });
    act(g, Action::FinishAttackers);
}

fn paid_mana(g: &Game) -> u64 {
    g.state.stack.last().unwrap().cast.unwrap().mana_spent()
}

#[test]
fn synthesizer_labship_grants_animation_below_its_own_creature_threshold() {
    let p = Player::P0;
    let mut g = scenario(p);
    let lab = put(&mut g, p, "Synthesizer Labship", Zone::Hand);
    let gunship = put(&mut g, p, "Warmaker Gunship", Zone::Battlefield);
    cast(&mut g, lab, None);
    settle(&mut g);
    let mut below = g.clone();
    below.begin_step(Step::BeginCombat);
    below.prepare_triggers();
    assert!(below.state.stack.is_empty());
    g.add_counter(gunship, CounterKind::Charge, 5);
    g.add_counter(lab, CounterKind::Charge, 2);
    assert!(!g.is_creature(lab));
    until(&mut g, |g| g.legal_actions().contains(&target(gunship)));
    reject(&mut g, target(lab));
    assert!(g.legal_actions().contains(&Action::FinishTargets));
    act(&mut g, target(gunship));
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, gunship), (2, 2));
    assert!(g.keywords(gunship).contains(Keywords::FLYING));
    g.add_counter(gunship, CounterKind::Charge, 1);
    assert_eq!(stats(&g, gunship), (2, 2));
    g.add_counter(lab, CounterKind::Charge, 6);
    assert!(!g.is_creature(lab));
    g.add_counter(lab, CounterKind::Charge, 1);
    assert_eq!(stats(&g, lab), (4, 4));
    assert!(
        g.keywords(lab)
            .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
    );
    let turn = g.state.turn;
    until(&mut g, |g| g.state.turn > turn);
    assert_eq!(stats(&g, gunship), (4, 3));
}

#[test]
fn tractor_beam_controls_and_taps_a_noncreature_spacecraft() {
    let p = Player::P0;
    let mut g = scenario(p);
    let beam = put(&mut g, p, "Tractor Beam", Zone::Hand);
    let ship = put(&mut g, p.other(), "Warmaker Gunship", Zone::Battlefield);
    assert!(!g.is_creature(ship));
    cast(&mut g, beam, Some(Target::Card(ship)));
    settle(&mut g);
    assert_eq!(g.object_view(beam).attached_to, Some(g.object_ref(ship)));
    assert_eq!(g.object_view(ship).controller, p);
    assert!(g.object_view(ship).tapped);
    let turn = g.state.turn;
    until(&mut g, |g| {
        g.state.turn > turn && g.state.active == p && g.state.step == Step::Upkeep
    });
    assert!(g.object_view(ship).tapped);
    g.untap_objects(&[ship], untap_rules::UntapCause::Effect);
    assert!(!g.object_view(ship).tapped);
    g.move_object(beam, Zone::Graveyard);
    assert_eq!(g.object_view(ship).controller, p.other());
}

#[test]
fn unravel_draws_for_the_discounted_warp_but_not_printed_mana_payment() {
    let p = Player::P0;
    for warped in [false, true] {
        let mut g = scenario(p.other());
        let spell = put(&mut g, p, "Unravel", Zone::Hand);
        let victim = put(
            &mut g,
            p.other(),
            if warped {
                "Sinister Cryologist"
            } else {
                "Divination"
            },
            Zone::Hand,
        );
        if warped {
            act(&mut g, Action::AnnounceWarp { card: victim });
            pay(&mut g);
        } else {
            cast(&mut g, victim, None);
        }
        act(&mut g, Action::Pass);
        cast(&mut g, spell, Some(Target::Spell(victim)));
        settle(&mut g);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.hand_count(p), usize::from(warped));
    }
}

#[test]
fn uthros_psionicist_reduces_exactly_the_second_spell_even_on_the_other_turn() {
    let p = Player::P0;
    for active in [p, p.other()] {
        let mut g = scenario(p);
        g.state.active = active;
        put(&mut g, p, "Uthros Psionicist", Zone::Battlefield);
        for expected in [3, 1, 3] {
            until(&mut g, |g| {
                g.actor() == Some(p) && g.state.pending == Pending::Priority
            });
            let spell = put(&mut g, p, "Quick Study", Zone::Hand);
            cast(&mut g, spell, None);
            assert_eq!(paid_mana(&g), expected);
            settle(&mut g);
        }
        assert_eq!(g.hand_count(p), 6);
    }
}

#[test]
fn uthros_scanship_draws_before_a_private_discard_and_stations_at_eight() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Uthros Scanship", Zone::Hand);
    let drawn = put(&mut g, p, "Island", Zone::Library);
    let other = put(&mut g, p, "Mountain", Zone::Library);
    cast(&mut g, ship, None);
    until(&mut g, |g| {
        matches!(g.state.pending, Pending::EffectDiscard { .. })
    });
    assert_eq!(g.hand_count(p), 2);
    assert!(
        g.observe(p.other())
            .objects
            .iter()
            .all(|o| o.id != drawn && o.id != other)
    );
    act(&mut g, Action::Discard { card: drawn });
    quiet(&mut g);
    assert_eq!(g.object_view(drawn).zone, Zone::Graveyard);
    assert_eq!(g.object_view(other).zone, Zone::Hand);
    let donor = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    g.add_counter(ship, CounterKind::Charge, 7);
    assert!(!g.is_creature(ship));
    act(&mut g, activate(ship));
    let object = g.object_ref(donor);
    act(&mut g, Action::ChooseAdditionalCost { object });
    pay(&mut g);
    settle(&mut g);
    assert_eq!(stats(&g, ship), (4, 4));
    assert!(g.keywords(ship).contains(Keywords::FLYING));
}

#[test]
fn weftwalking_refills_only_on_cast_entry_and_allows_the_first_offhand_spell() {
    let p = Player::P0;
    let mut g = scenario(p);
    let weft = put(&mut g, p, "Weftwalking", Zone::Hand);
    let old = put(&mut g, p, "Island", Zone::Graveyard);
    put(&mut g, p, "Mountain", Zone::Hand);
    let foe_grave = put(&mut g, p.other(), "Plains", Zone::Graveyard);
    let exile = put(&mut g, p.other(), "Hill Giant", Zone::Exile);
    cast(&mut g, weft, None);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 7);
    assert_ne!(g.object_view(old).zone, Zone::Graveyard);
    assert_eq!(g.object_view(foe_grave).zone, Zone::Graveyard);
    let free = |card| Action::AnnounceFree {
        card,
        face: CastFace::Normal,
        flash_surcharge: false,
    };
    let mut same_turn = g.clone();
    let later = put(&mut same_turn, p, "Hill Giant", Zone::Hand);
    reject(&mut same_turn, free(later));
    let mut blinked = g.clone();
    let saved = put(&mut blinked, p, "Island", Zone::Graveyard);
    let hand = blinked.hand_count(p);
    blinked.move_object(weft, Zone::Exile);
    blinked.move_object(weft, Zone::Battlefield);
    quiet(&mut blinked);
    assert_eq!(blinked.hand_count(p), hand);
    assert_eq!(blinked.object_view(saved).zone, Zone::Graveyard);

    until(&mut g, |g| {
        g.state.active == p.other() && g.state.step == Step::Main1 && g.actor() == Some(p.other())
    });
    g.grant_exile_play_permission(p.other(), g.object_ref(exile), false);
    act(&mut g, free(exile));
    pay(&mut g);
    assert_eq!(paid_mana(&g), 0);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().origin,
        Zone::Exile
    );
    settle(&mut g);
    assert_eq!(g.object_view(exile).zone, Zone::Battlefield);
    let second = put(&mut g, p.other(), "Hill Giant", Zone::Hand);
    reject(&mut g, free(second));
}

#[test]
fn alpharael_has_random_discard_ward_and_void_halves_odd_life() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Alpharael, Stonechosen", Zone::Hand);
    let witness = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    cast(&mut g, source, None);
    settle(&mut g);
    g.state.objects[source as usize].summoning_sick = false;
    let mut ward = g.clone();
    let bounce = put(&mut ward, p.other(), "Unsummon", Zone::Hand);
    let a = put(&mut ward, p.other(), "Island", Zone::Hand);
    let b = put(&mut ward, p.other(), "Mountain", Zone::Hand);
    act(&mut ward, Action::Pass);
    cast(&mut ward, bounce, Some(Target::Card(source)));
    until(&mut ward, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    assert_eq!(ward.actor(), Some(p.other()));
    assert!(
        ward.legal_actions()
            .iter()
            .all(|a| matches!(a, Action::ChooseOptional { .. } | Action::Concede))
    );
    act(&mut ward, Action::ChooseOptional { accept: true });
    assert_eq!(
        [a, b]
            .into_iter()
            .filter(|&id| ward.object_view(id).zone == Zone::Graveyard)
            .count(),
        1
    );
    settle(&mut ward);
    assert_eq!(ward.object_view(source).zone, Zone::Hand);

    g.state.players[p.other().index()].life = 19;
    let mut normal = g.clone();
    attack(&mut normal, source);
    quiet(&mut normal);
    assert_eq!(normal.state.players[p.other().index()].life, 19);
    g.move_object(witness, Zone::Exile);
    attack(&mut g, source);
    quiet(&mut g);
    assert_eq!(g.state.players[p.other().index()].life, 9);
}

#[test]
fn archenemys_charm_exiles_returns_and_grants_lifelink_in_its_distinct_modes() {
    let p = Player::P0;
    for mode in 0..3 {
        let mut g = scenario(p);
        let charm = put(&mut g, p, "Archenemy's Charm", Zone::Hand);
        let our = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let enemy = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        let grave = put(&mut g, p, "Grizzly Bears", Zone::Graveyard);
        let walker = put(&mut g, p, "Ajani, Caller of the Pride", Zone::Graveyard);
        act(&mut g, Action::AnnounceCast { card: charm });
        act(&mut g, Action::ChooseMode { mode });
        match mode {
            0 => act(&mut g, target(enemy)),
            1 => {
                act(&mut g, target(grave));
                act(&mut g, target(walker));
            }
            _ => {
                reject(&mut g, target(enemy));
                act(&mut g, target(our));
            }
        }
        finish_targets(&mut g);
        pay(&mut g);
        if mode == 1 {
            g.move_object(grave, Zone::Exile);
        }
        settle(&mut g);
        match mode {
            0 => assert_eq!(g.object_view(enemy).zone, Zone::Exile),
            1 => {
                assert_eq!(g.object_view(walker).zone, Zone::Hand);
                assert_eq!(g.object_view(grave).zone, Zone::Exile);
            }
            _ => {
                assert_eq!(g.object_view(our).counters.plus_one, 2);
                assert!(g.keywords(our).contains(Keywords::LIFELINK));
            }
        }
    }
}

#[test]
fn beamsaw_prospector_death_makes_a_lander_that_searches_for_a_tapped_basic() {
    let p = Player::P0;
    let mut g = scenario(p);
    let prospector = put(&mut g, p, "Beamsaw Prospector", Zone::Hand);
    let basic = put(&mut g, p, "Swamp", Zone::Library);
    cast(&mut g, prospector, None);
    settle(&mut g);
    let mut exiled = g.clone();
    exiled.move_object(prospector, Zone::Exile);
    quiet(&mut exiled);
    assert!(!exiled.observe(p).objects.iter().any(|o| o.is_token()));
    g.move_object(prospector, Zone::Graveyard);
    quiet(&mut g);
    let lander = g
        .observe(p)
        .objects
        .into_iter()
        .find(|o| o.is_token())
        .unwrap();
    assert!(lander.has_subtype("Lander"));
    assert_ne!(lander.kinds & 16, 0);
    act(&mut g, activate(lander.id));
    pay(&mut g);
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseSearch { card: basic })
    });
    assert!(g.observe(p.other()).search_choice.is_none());
    act(&mut g, Action::ChooseSearch { card: basic });
    if g.legal_actions().contains(&Action::FinishSearch) {
        act(&mut g, Action::FinishSearch);
    }
    settle(&mut g);
    assert!(!g.on_battlefield(lander.id));
    assert_eq!(g.object_view(basic).zone, Zone::Battlefield);
    assert!(g.object_view(basic).tapped);
}

#[test]
fn blade_of_the_swarm_chooses_counters_or_bottoms_an_exiled_printed_warp_card() {
    let p = Player::P0;
    let mut g = scenario(p);
    let blade = put(&mut g, p, "Blade of the Swarm", Zone::Hand);
    let warped = put(&mut g, p.other(), "Sinister Cryologist", Zone::Exile);
    let unrelated = put(&mut g, p.other(), "Hill Giant", Zone::Exile);
    cast(&mut g, blade, None);
    until(&mut g, |g| {
        g.legal_actions().contains(&Action::ChooseMode { mode: 0 })
    });
    let mut counters = g.clone();
    act(&mut counters, Action::ChooseMode { mode: 0 });
    settle(&mut counters);
    assert_eq!(stats(&counters, blade), (5, 3));
    assert_eq!(counters.object_view(warped).zone, Zone::Exile);
    assert!(g.observe(p).warped_cards.is_empty());
    act(&mut g, Action::ChooseMode { mode: 1 });
    reject(&mut g, target(unrelated));
    act(&mut g, target(warped));
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(warped).zone, Zone::Library);
    assert_eq!(
        g.state.players[p.other().index()].library.first().copied(),
        Some(warped)
    );
}

#[test]
fn chorale_of_the_void_returns_the_defenders_card_and_can_survive_its_end_trigger() {
    let p = Player::P0;
    let mut g = scenario(p);
    let aura = put(&mut g, p, "Chorale of the Void", Zone::Hand);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let own = put(&mut g, p, "Bear Cub", Zone::Graveyard);
    let foe = put(&mut g, p.other(), "Hill Giant", Zone::Graveyard);
    let witness = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    cast(&mut g, aura, Some(Target::Card(bear)));
    settle(&mut g);
    let mut ending = g.clone();
    ending.begin_step(Step::End);
    ending.prepare_triggers();
    let mut saved = ending.clone();
    saved.move_object(witness, Zone::Graveyard);
    settle(&mut saved);
    assert_eq!(saved.object_view(aura).zone, Zone::Battlefield);
    settle(&mut ending);
    assert_eq!(ending.object_view(aura).zone, Zone::Graveyard);

    attack(&mut g, bear);
    until(&mut g, |g| g.legal_actions().contains(&target(foe)));
    reject(&mut g, target(own));
    act(&mut g, target(foe));
    finish_targets(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(foe).controller, p);
    assert!(g.object_view(foe).tapped);
    assert!(g.state.attackers.iter().any(|a| a.card == foe));
}

#[test]
fn comet_crawler_sacrifices_another_artifact_before_lifelink_combat_damage() {
    let p = Player::P0;
    let mut g = scenario(p);
    let crawler = put(&mut g, p, "Comet Crawler", Zone::Battlefield);
    let artifact = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    attack(&mut g, crawler);
    let sacrifice = Action::Sacrifice {
        object: g.object_ref(artifact),
    };
    until(&mut g, |g| g.legal_actions().contains(&sacrifice));
    let own = g.object_ref(crawler);
    reject(&mut g, Action::Sacrifice { object: own });
    for pay in [false, true] {
        let mut branch = g.clone();
        act(
            &mut branch,
            if pay {
                sacrifice.clone()
            } else {
                Action::DeclineEffectPayment
            },
        );
        quiet(&mut branch);
        let amount = if pay { 4 } else { 2 };
        assert_eq!(stats(&branch, crawler), (amount, 3));
        assert_eq!(
            branch.object_view(artifact).zone,
            if pay {
                Zone::Graveyard
            } else {
                Zone::Battlefield
            }
        );
        until(&mut branch, |g| {
            g.state.players[p.other().index()].life < 20
        });
        assert_eq!(branch.state.players[p.other().index()].life, 20 - amount);
        assert_eq!(branch.state.players[p.index()].life, 20 + amount);
    }
}

#[test]
fn dark_endurance_funds_its_single_black_route_only_for_a_blocking_target() {
    let p = Player::P0;
    let mut g = scenario(p);
    let giant = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let blocker = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let spell = put(&mut g, p, "Dark Endurance", Zone::Hand);
    g.state.players[p.index()].mana = BLACK;
    reject(&mut g, Action::AnnounceCast { card: spell });
    attack(&mut g, giant);
    until(&mut g, |g| g.state.pending == Pending::Blockers);
    act(
        &mut g,
        Action::Block {
            blocker,
            attacker: giant,
        },
    );
    act(&mut g, Action::FinishBlockers);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    g.state.players[p.index()].mana = BLACK;
    act(&mut g, Action::AnnounceCast { card: spell });
    reject(&mut g, target(giant));
    act(&mut g, target(blocker));
    finish_targets(&mut g);
    pay(&mut g);
    assert_eq!(paid_mana(&g), 1);
    settle(&mut g);
    assert_eq!(stats(&g, blocker), (4, 2));
    assert!(g.keywords(blocker).contains(Keywords::INDESTRUCTIBLE));
    until(&mut g, |g| g.state.step == Step::EndCombat);
    assert_eq!(g.object_view(blocker).zone, Zone::Battlefield);
    assert_eq!(g.object_view(giant).zone, Zone::Graveyard);
}

#[test]
fn decode_transmissions_draws_two_and_uses_only_the_matching_life_branch() {
    let p = Player::P0;
    for enabled in [false, true] {
        let mut g = scenario(p);
        let witness = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        if enabled {
            g.move_object(witness, Zone::Exile);
        }
        let spell = put(&mut g, p, "Decode Transmissions", Zone::Hand);
        cast(&mut g, spell, None);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 2);
        assert_eq!(
            g.state.players[p.index()].life,
            if enabled { 20 } else { 18 }
        );
        assert_eq!(
            g.state.players[p.other().index()].life,
            if enabled { 18 } else { 20 }
        );
    }
}
