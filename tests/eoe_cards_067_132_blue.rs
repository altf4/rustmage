//! Paid card scenarios for Edge of Eternities collectors 067–080.
mod support;

use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const BLUE: Mana = [0, 1, 0, 0, 0, 0];
const TWO_BLUE: Mana = [0, 2, 0, 0, 0, 0];

fn passes(g: &mut Game) {
    act(g, Action::Pass);
    act(g, Action::Pass);
}

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

fn target(card: CardId) -> Action {
    Action::ChooseTarget {
        target: Target::Card(card),
    }
}

fn select_target(g: &mut Game, card: CardId) {
    act(g, target(card));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}

fn activate(card: CardId) -> Action {
    Action::AnnounceActivate {
        card,
        ability: 0,
        counter_payment: Counters::default(),
    }
}

fn station(g: &mut Game, ship: CardId, creature: CardId) {
    act(g, activate(ship));
    let object = g.object_ref(creature);
    act(g, Action::ChooseAdditionalCost { object });
    pay(g);
}

fn warp(g: &mut Game, card: CardId) {
    act(g, Action::AnnounceWarp { card });
    pay(g);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Warp
    );
}

#[test]
fn mental_modulation_locks_own_turn_cost_and_draws_for_a_tapped_target() {
    let p = Player::P0;
    for (own, mana, expected) in [(true, BLUE, 1), (false, TWO_BLUE, 2)] {
        let mut g = scenario(p);
        if !own {
            g.state.active = p.other();
        }
        let spell = put(&mut g, p, "Mental Modulation", Zone::Hand);
        let artifact = put(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
        g.state.objects[artifact as usize].tapped = true;
        if !own {
            g.state.players[p.index()].mana = BLUE;
            reject(&mut g, Action::AnnounceCast { card: spell });
        }
        g.state.players[p.index()].mana = mana;
        cast(&mut g, spell, Some(Target::Card(artifact)));
        assert_eq!(
            g.state.stack.last().unwrap().cast.unwrap().mana_spent(),
            expected
        );
        let before = g.hand_count(p);
        let mut stale = g.clone();
        stale.move_object(artifact, Zone::Exile);
        settle(&mut stale);
        assert_eq!(stale.hand_count(p), before);
        settle(&mut g);
        assert!(g.object_view(artifact).tapped);
        assert_eq!(g.hand_count(p), before + 1);
    }
}

#[test]
fn mm_menon_looks_privately_and_two_artifacts_fund_an_offhand_spell() {
    let p = Player::P0;
    let mut g = scenario(p);
    let provider = put(&mut g, p, "Mm'menon, the Right Hand", Zone::Hand);
    let a = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let b = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    let opposing = put(&mut g, p.other(), "Candy Trail", Zone::Battlefield);
    let hand = put(&mut g, p, "Swiftfoot Boots", Zone::Hand);
    let artifact = put(&mut g, p, "Swiftfoot Boots", Zone::Library);
    let cover = put(&mut g, p, "Hill Giant", Zone::Library);
    cast(&mut g, provider, None);
    settle(&mut g);
    assert!(g.keywords(provider).contains(Keywords::FLYING));
    assert_eq!(
        g.observe(p).known_library_top[p.index()].unwrap().object,
        g.object_ref(cover)
    );
    assert!(g.observe(p.other()).known_library_top[p.index()].is_none());
    reject(&mut g, Action::AnnounceCast { card: cover });
    g.move_object(cover, Zone::Exile);
    g.state.players[p.index()].mana = [0; 6];
    let grantor = g.object_ref(provider);
    let tap = |card| Action::TapGrantedMana {
        card,
        grantor,
        production: BLUE,
    };
    reject(&mut g, tap(provider));
    reject(&mut g, tap(opposing));

    let mut restricted = g.clone();
    act(&mut restricted, tap(a));
    act(&mut restricted, tap(b));
    assert_eq!(
        restricted.observe(p).mana_lots[0].restriction,
        ManaRestrictionView::CastFromNonhand
    );
    reject(&mut restricted, Action::AnnounceCast { card: hand });

    act(&mut g, Action::AnnounceCast { card: artifact });
    act(&mut g, tap(a));
    act(&mut g, tap(b));
    pay(&mut g);
    assert_eq!(
        g.state.stack.last().unwrap().cast.unwrap().origin,
        Zone::Library
    );
    settle(&mut g);
    assert_eq!(g.object_view(artifact).zone, Zone::Battlefield);
}

#[test]
fn moonlit_meditation_copies_a_noncreature_artifact_and_declining_spends_the_event() {
    let p = Player::P0;
    let mut parent = scenario(p);
    let boots = put(&mut parent, p, "Swiftfoot Boots", Zone::Battlefield);
    let aura = put(&mut parent, p, "Moonlit Meditation", Zone::Hand);
    let first = put(&mut parent, p, "Gravpack Monoist", Zone::Battlefield);
    let second = put(&mut parent, p, "Gravpack Monoist", Zone::Battlefield);
    cast(&mut parent, aura, Some(Target::Card(boots)));
    settle(&mut parent);
    assert_eq!(
        parent.object_view(aura).attached_to,
        Some(parent.object_ref(boots))
    );
    parent.add_counter(boots, CounterKind::Charge, 4);
    parent.move_object(first, Zone::Graveyard);
    parent.prepare_triggers();
    until(&mut parent, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    let boots_card = parent.object_view(boots).card;
    for accept in [false, true] {
        let mut g = parent.clone();
        act(&mut g, Action::ChooseOptional { accept });
        quiet(&mut g);
        g.move_object(second, Zone::Graveyard);
        quiet(&mut g);
        let tokens = g
            .observe(p)
            .objects
            .into_iter()
            .filter(|o| o.zone == Zone::Battlefield && o.is_token())
            .collect::<Vec<_>>();
        assert_eq!(tokens.len(), 2);
        assert_eq!(
            tokens.iter().filter(|o| o.card == boots_card).count(),
            usize::from(accept)
        );
        assert!(
            tokens
                .iter()
                .all(|o| o.tapped && g.counter_count(o.id, CounterKind::Charge) == 0)
        );
    }
}

#[test]
fn mouth_of_the_storm_snapshots_the_affected_group_and_ward_can_counter() {
    let p = Player::P0;
    let mut g = scenario(p);
    let mouth = put(&mut g, p, "Mouth of the Storm", Zone::Hand);
    let opposing = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let late = put(&mut g, p.other(), "Bear Cub", Zone::Hand);
    cast(&mut g, mouth, None);
    settle(&mut g);
    assert_eq!(stats(&g, opposing), (0, 3));
    assert!(g.keywords(mouth).contains(Keywords::FLYING));
    g.move_object(late, Zone::Battlefield);
    assert_eq!(stats(&g, late), (2, 2));

    let mut ward = g.clone();
    let bounce = put(&mut ward, p.other(), "Unsummon", Zone::Hand);
    act(&mut ward, Action::Pass);
    cast(&mut ward, bounce, Some(Target::Card(mouth)));
    until(&mut ward, |g| {
        g.legal_actions().contains(&Action::DeclineResolutionMana)
    });
    let mut paid = ward.clone();
    act(&mut paid, Action::PayResolutionMana { payment: TWO_BLUE });
    settle(&mut paid);
    assert_eq!(paid.object_view(mouth).zone, Zone::Hand);
    act(&mut ward, Action::DeclineResolutionMana);
    settle(&mut ward);
    assert_eq!(ward.object_view(mouth).zone, Zone::Battlefield);
    assert_eq!(ward.object_view(bounce).zone, Zone::Graveyard);

    g.change_controller(g.object_ref(opposing), p);
    assert_eq!(stats(&g, opposing), (0, 3));
    let turn = g.state.turn;
    until(&mut g, |g| g.state.turn > turn && g.state.active == p);
    assert_eq!(stats(&g, opposing), (3, 3));
}

#[test]
fn nanoform_sentinel_observes_station_payment_once_each_turn() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Specimen Freighter", Zone::Battlefield);
    let sentinel = put(&mut g, p, "Nanoform Sentinel", Zone::Hand);
    let forest = put(&mut g, p, "Forest", Zone::Battlefield);
    cast(&mut g, sentinel, None);
    settle(&mut g);
    assert!(g.object_view(sentinel).summoning_sick);
    g.state.objects[forest as usize].tapped = true;
    station(&mut g, ship, sentinel);
    until(&mut g, |g| g.legal_actions().contains(&target(forest)));
    reject(&mut g, target(sentinel));
    select_target(&mut g, forest);
    quiet(&mut g);
    assert!(!g.object_view(forest).tapped);
    assert_eq!(g.counter_count(ship, CounterKind::Charge), 3);

    g.state.objects[sentinel as usize].tapped = false;
    g.state.objects[forest as usize].tapped = true;
    station(&mut g, ship, sentinel);
    quiet(&mut g);
    assert!(g.object_view(forest).tapped);
    assert_eq!(g.counter_count(ship, CounterKind::Charge), 6);
}

#[test]
fn quantum_riddler_warp_and_whole_draw_event_use_the_current_hand_size() {
    let p = Player::P0;
    let mut g = scenario(p);
    let riddler = put(&mut g, p, "Quantum Riddler", Zone::Hand);
    let land = put(&mut g, p, "Island", Zone::Hand);
    warp(&mut g, riddler);
    assert_eq!(g.state.stack.last().unwrap().cast.unwrap().mana_spent(), 2);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 3);
    assert!(g.keywords(riddler).contains(Keywords::FLYING));
    let divination = put(&mut g, p, "Divination", Zone::Hand);
    let mut large = g.clone();
    cast(&mut large, divination, None);
    settle(&mut large);
    assert_eq!(large.hand_count(p), 5);
    let draw_cards = g
        .observe(p)
        .objects
        .into_iter()
        .filter(|o| o.owner == p && o.zone == Zone::Hand && o.id != land && o.id != divination)
        .map(|o| o.id)
        .collect::<Vec<_>>();
    for id in draw_cards {
        g.move_object(id, Zone::Graveyard);
    }
    cast(&mut g, divination, None);
    settle(&mut g);
    assert_eq!(g.hand_count(p), 4);
    until(&mut g, |g| g.object_view(riddler).zone == Zone::Exile);
    reject(&mut g, Action::AnnounceExile { card: riddler });
}

#[test]
fn scour_for_scrap_chooses_each_mode_and_searches_before_returning() {
    let p = Player::P0;
    for mode in 0..3 {
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Scour for Scrap", Zone::Hand);
        let library = put(&mut g, p, "Candy Trail", Zone::Library);
        let grave = put(&mut g, p, "Swiftfoot Boots", Zone::Graveyard);
        act(&mut g, Action::AnnounceCast { card: spell });
        act(&mut g, Action::ChooseMode { mode });
        if mode != 0 {
            select_target(&mut g, grave);
        }
        pay(&mut g);
        if mode == 2 {
            let mut stale = g.clone();
            stale.move_object(grave, Zone::Exile);
            settle(&mut stale);
            assert_eq!(stale.object_view(library).zone, Zone::Library);
        }
        passes(&mut g);
        if mode != 1 {
            assert_eq!(g.object_view(grave).zone, Zone::Graveyard);
            assert!(g.observe(p.other()).search_choice.is_none());
            act(&mut g, Action::ChooseSearch { card: library });
            if g.legal_actions().contains(&Action::FinishSearch) {
                act(&mut g, Action::FinishSearch);
            }
        }
        settle(&mut g);
        assert_eq!(g.object_view(library).zone == Zone::Hand, mode != 1);
        assert_eq!(g.object_view(grave).zone == Zone::Hand, mode != 0);
    }
}

#[test]
fn selfcraft_mechan_can_sacrifice_itself_before_its_reflexive_target_is_chosen() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Selfcraft Mechan", Zone::Hand);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    cast(&mut g, source, None);
    until(&mut g, |g| {
        g.legal_actions()
            .iter()
            .any(|action| matches!(action, Action::Sacrifice { object } if object.card == source))
    });
    let object = g.object_ref(source);
    let sacrifice = Action::Sacrifice { object };
    let nonartifact = g.object_ref(bear);
    reject(
        &mut g,
        Action::Sacrifice {
            object: nonartifact,
        },
    );
    let mut declined = g.clone();
    act(&mut declined, Action::DeclineEffectPayment);
    quiet(&mut declined);
    assert_eq!(declined.object_view(source).zone, Zone::Battlefield);
    assert_eq!(declined.hand_count(p), 0);
    act(&mut g, sacrifice);
    assert_eq!(g.object_view(source).zone, Zone::Graveyard);
    until(&mut g, |g| g.legal_actions().contains(&target(bear)));
    select_target(&mut g, bear);
    let mut stale = g.clone();
    stale.move_object(bear, Zone::Exile);
    settle(&mut stale);
    assert_eq!(stale.hand_count(p), 0);
    settle(&mut g);
    assert_eq!(g.object_view(bear).counters.plus_one, 1);
    assert_eq!(g.hand_count(p), 1);
}

#[test]
fn sinister_cryologist_warp_targets_only_an_opposing_creature() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Sinister Cryologist", Zone::Hand);
    let theirs = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let ours = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    g.state.players[p.index()].mana = BLUE;
    warp(&mut g, source);
    until(&mut g, |g| g.legal_actions().contains(&target(theirs)));
    reject(&mut g, target(ours));
    select_target(&mut g, theirs);
    settle(&mut g);
    assert_eq!(stats(&g, theirs), (0, 3));
    until(&mut g, |g| g.object_view(source).zone == Zone::Exile);
    assert_eq!(g.observe(p).warped_cards[0].object, g.object_ref(source));
}

#[test]
fn specimen_freighter_returns_two_creatures_and_mills_after_losing_station_counters() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ship = put(&mut g, p, "Specimen Freighter", Zone::Hand);
    let first = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    let second = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
    let excluded = put(&mut g, p.other(), "Warmaker Gunship", Zone::Battlefield);
    g.add_counter(excluded, CounterKind::Charge, 6);
    cast(&mut g, ship, None);
    until(&mut g, |g| g.legal_actions().contains(&target(first)));
    reject(&mut g, target(excluded));
    assert!(g.legal_actions().contains(&Action::FinishTargets));
    act(&mut g, target(first));
    act(&mut g, target(second));
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    settle(&mut g);
    assert_eq!(g.object_view(first).zone, Zone::Hand);
    assert_eq!(g.object_view(second).zone, Zone::Hand);
    assert!(!g.is_creature(ship));
    g.add_counter(ship, CounterKind::Charge, 8);
    let donor = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    assert!(!g.is_creature(ship));
    station(&mut g, ship, donor);
    settle(&mut g);
    assert_eq!(stats(&g, ship), (4, 7));
    assert!(g.keywords(ship).contains(Keywords::FLYING));
    g.state.objects[ship as usize].summoning_sick = false;
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    act(&mut g, Action::Attack { card: ship });
    act(&mut g, Action::FinishAttackers);
    until(&mut g, |g| {
        g.state
            .stack
            .iter()
            .any(|s| s.source.is_some_and(|x| x.id == ship))
    });
    g.pay_counters(ship, &Counters::of(CounterKind::Charge, 1));
    let before = g.state.players[p.other().index()].library.len();
    settle(&mut g);
    assert_eq!(g.state.players[p.other().index()].library.len(), before - 4);
    assert!(!g.is_creature(ship));
}

#[test]
fn starbreach_whale_warp_and_later_paid_cast_each_surveil_privately() {
    let p = Player::P0;
    let mut g = scenario(p);
    let whale = put(&mut g, p, "Starbreach Whale", Zone::Hand);
    let grave = put(&mut g, p, "Island", Zone::Library);
    let top = put(&mut g, p, "Mountain", Zone::Library);
    warp(&mut g, whale);
    until(&mut g, |g| g.observe(p).library_choice.is_some());
    let choice = g.observe(p).library_choice.unwrap();
    assert!(choice.surveil);
    assert_eq!(choice.remaining.len(), 2);
    assert!(g.observe(p.other()).library_choice.is_none());
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: grave,
            destination: LibraryDestination::Graveyard,
        },
    );
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Top,
        },
    );
    settle(&mut g);
    assert_eq!(g.object_view(grave).zone, Zone::Graveyard);
    assert_eq!(
        g.state.players[p.index()].library.last().copied(),
        Some(top)
    );
    assert!(g.keywords(whale).contains(Keywords::FLYING));
    until(&mut g, |g| g.object_view(whale).zone == Zone::Exile);
    let exiled_turn = g.state.turn;
    until(&mut g, |g| {
        g.state.turn > exiled_turn && g.state.active == p && g.state.step == Step::Main1
    });
    g.state.players[p.index()].mana = [0, 5, 0, 0, 0, 0];
    act(&mut g, Action::AnnounceExile { card: whale });
    pay(&mut g);
    assert_eq!(g.state.stack.last().unwrap().cast.unwrap().mana_spent(), 5);
    until(&mut g, |g| g.observe(p).library_choice.is_some());
    assert_eq!(g.observe(p).library_choice.unwrap().remaining.len(), 2);
}

#[test]
fn starfield_vocalist_doubles_controlled_permanent_triggers_for_either_entrant() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Dazzling Angel", Zone::Battlefield);
    put(&mut g, p, "Authority of the Consuls", Zone::Battlefield);
    let vocalist = put(&mut g, p, "Starfield Vocalist", Zone::Hand);
    warp(&mut g, vocalist);
    quiet(&mut g);
    assert_eq!(g.state.players[p.index()].life, 22);
    let theirs = put(&mut g, p.other(), "Grizzly Bears", Zone::Hand);
    g.move_object(theirs, Zone::Battlefield);
    quiet(&mut g);
    assert_eq!(g.state.players[p.index()].life, 24);
    until(&mut g, |g| g.object_view(vocalist).zone == Zone::Exile);
    let later = put(&mut g, p, "Bear Cub", Zone::Hand);
    g.move_object(later, Zone::Battlefield);
    quiet(&mut g);
    assert_eq!(g.state.players[p.index()].life, 25);
}

#[test]
fn starwinder_has_an_independently_optional_draw_for_each_combat_damage_dealer() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Starwinder", Zone::Hand);
    let bear = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let giant = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    warp(&mut g, source);
    settle(&mut g);
    assert_eq!(stats(&g, source), (7, 7));
    until(&mut g, |g| g.state.pending == Pending::Attackers);
    act(&mut g, Action::Attack { card: bear });
    act(&mut g, Action::Attack { card: giant });
    act(&mut g, Action::FinishAttackers);
    until(&mut g, |g| g.state.stack.len() == 2);
    let mut sizes = g
        .state
        .stack
        .iter()
        .map(|s| s.damage_event.unwrap().amount)
        .collect::<Vec<_>>();
    sizes.sort_unstable();
    assert_eq!(sizes, [2, 3]);
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: true })
    });
    let amount = g
        .state
        .resolving
        .as_ref()
        .unwrap()
        .item
        .damage_event
        .unwrap()
        .amount as usize;
    act(&mut g, Action::ChooseOptional { accept: true });
    until(&mut g, |g| {
        g.legal_actions()
            .contains(&Action::ChooseOptional { accept: false })
    });
    act(&mut g, Action::ChooseOptional { accept: false });
    quiet(&mut g);
    assert_eq!(g.hand_count(p), amount);
    assert_eq!(g.state.players[p.other().index()].life, 15);
}

#[test]
fn steelswarm_operator_uses_distinct_tap_routes_for_artifact_spells_and_abilities() {
    let p = Player::P0;
    let mut g = scenario(p);
    let operator = put(&mut g, p, "Steelswarm Operator", Zone::Battlefield);
    let boots = put(&mut g, p, "Swiftfoot Boots", Zone::Hand);
    let trail = put(&mut g, p, "Candy Trail", Zone::Battlefield);
    assert!(g.keywords(operator).contains(Keywords::FLYING));
    let mana = |ability, production| Action::ActivateMana {
        card: operator,
        ability,
        payment: [0; 6],
        counter_payment: Counters::default(),
        production,
    };

    let mut activate_only = g.clone();
    activate_only.state.players[p.index()].mana = [0; 6];
    act(&mut activate_only, mana(1, TWO_BLUE));
    reject(&mut activate_only, mana(0, BLUE));
    reject(&mut activate_only, Action::AnnounceCast { card: boots });
    act(&mut activate_only, activate(trail));
    pay(&mut activate_only);
    settle(&mut activate_only);
    assert_eq!(activate_only.object_view(trail).zone, Zone::Graveyard);
    assert_eq!(activate_only.state.players[p.index()].life, 23);

    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    act(&mut g, mana(0, BLUE));
    reject(&mut g, activate(trail));
    cast(&mut g, boots, None);
    settle(&mut g);
    assert_eq!(g.object_view(boots).zone, Zone::Battlefield);
}
