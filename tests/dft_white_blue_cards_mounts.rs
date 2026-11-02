//! Real mount, crew participant and direct saddle outcomes.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn idle(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn drain(g: &mut Game) {
    for _ in 0..64 {
        if idle(g) {
            return;
        }
        choose(g, |a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::FinishStackTargets
            )
        });
    }
    panic!("mount resolution ceiling");
}
fn ready(g: &mut Game, predicate: impl Fn(&Action) -> bool) {
    for _ in 0..32 {
        if g.legal_actions().iter().any(&predicate) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("mount decision ceiling");
}
fn activate(g: &mut Game, id: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
}
fn mount(g: &mut Game, id: CardId, ability: u16, contributors: &[CardId], crew: bool) {
    activate(g, id, ability);
    for &id in contributors {
        let object = g.object_ref(id);
        act(
            g,
            if crew {
                Action::ChooseCrewer { object }
            } else {
                Action::ChooseSaddler { object }
            },
        );
    }
    act(
        g,
        if crew {
            Action::FinishCrew
        } else {
            Action::FinishSaddle
        },
    );
    drain(g);
}
fn attack(g: &mut Game, id: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    g.state.passes = 0;
    act(g, Action::Attack { card: id });
    act(g, Action::FinishAttackers);
}
fn target(g: &mut Game, id: CardId) {
    ready(g, |a| {
        *a == Action::ChooseTarget {
            target: Target::Card(id),
        }
    });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(id),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn both(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    g.change_object_form(
        object,
        AnimationSpec {
            artifact: true,
            creature_types: AnimationCreatureTypes::Named(&["Mount"]),
            base: None,
            keywords: Keywords::NONE,
            duration: AnimationDuration::Permanent,
        },
        characteristic_layers::AnimationTypeMode::AddCreature,
    );
}

#[test]
fn armory_buffs_only_own_creatures_and_independently_saddles_mounts_or_animates_vehicles() {
    for shape in 0..3 {
        let p = Player::P0;
        let mut g = scenario(p);
        let id = put(
            &mut g,
            p,
            if shape == 0 {
                "Trained Arynx"
            } else {
                "Veloheart Bike"
            },
            Zone::Battlefield,
        );
        if shape == 2 {
            both(&mut g, id);
        }
        let enemy = put(&mut g, p.other(), "Trained Arynx", Zone::Battlefield);
        let source = put(&mut g, p, "Alacrian Armory", Zone::Hand);
        cast(&mut g, source, None);
        drain(&mut g);
        assert_eq!(stats(&g, enemy), (3, 1));
        if shape == 0 {
            assert_eq!(stats(&g, id), (3, 2));
        }
        assert_eq!(g.keywords(id).contains(Keywords::VIGILANCE), shape != 1);
        g.begin_step(Step::BeginCombat);
        g.give_priority();
        ready(&mut g, |a| {
            *a == Action::ChooseTarget {
                target: Target::Card(id),
            }
        });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(enemy),
            },
        );
        target(&mut g, id);
        drain(&mut g);
        assert_eq!(g.is_saddled(g.object_ref(id)), shape != 1);
        assert!(g.is_creature(id));
        assert!(g.keywords(id).contains(Keywords::VIGILANCE));
        assert_eq!(g.current_kinds(id) & 16 != 0, shape != 0);
        if shape == 1 {
            assert_eq!(stats(&g, id), (4, 3));
        }
        g.finish_cleanup();
        assert!(!g.is_saddled(g.object_ref(id)));
        if shape == 1 {
            assert!(!g.is_creature(id));
        }
    }
}

#[test]
fn armory_can_choose_no_target_and_does_not_trigger_in_the_opponents_combat() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Alacrian Armory", Zone::Battlefield);
    let vehicle = put(&mut g, p, "Veloheart Bike", Zone::Battlefield);
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    ready(&mut g, |a| matches!(a, Action::FinishTargets));
    act(&mut g, Action::FinishTargets);
    drain(&mut g);
    assert!(!g.is_creature(vehicle));
    g.state.active = p.other();
    g.begin_step(Step::BeginCombat);
    g.give_priority();
    assert!(g.state.pending_triggers.is_empty());
}

#[test]
fn saddled_brightfield_attacks_have_distinct_growth_flying_untap_and_counter_results() {
    for (name, minimum, body) in [
        ("Brightfield Glider", 3, (1, 1)),
        ("Brightfield Mustang", 1, (3, 3)),
    ] {
        let p = Player::P0;
        let mut g = scenario(p);
        let source = put(&mut g, p, name, Zone::Battlefield);
        assert_eq!(stats(&g, source), body);
        let start = Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        };
        reject(&mut g, start.clone());
        if minimum == 3 {
            put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
            reject(&mut g, start);
        }
        let contributor = put(
            &mut g,
            p,
            if minimum == 1 {
                "Llanowar Elves"
            } else {
                "Hill Giant"
            },
            Zone::Battlefield,
        );
        mount(&mut g, source, 0, &[contributor], false);
        assert!(g.object_view(contributor).tapped);
        assert!(g.is_saddled(g.object_ref(source)));
        attack(&mut g, source);
        drain(&mut g);
        if name == "Brightfield Glider" {
            assert_eq!(stats(&g, source), (2, 3));
            assert!(
                g.keywords(source)
                    .contains(Keywords::FLYING.union(Keywords::VIGILANCE))
            );
            assert!(!g.object_view(source).tapped);
        } else {
            assert_eq!(stats(&g, source), (4, 4));
            assert!(!g.object_view(source).tapped);
            assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
        }
        g.finish_cleanup();
        assert_eq!(
            stats(&g, source),
            if name == "Brightfield Glider" {
                (1, 1)
            } else {
                (4, 4)
            }
        );
        assert!(!g.keywords(source).contains(Keywords::FLYING));
    }
}

#[test]
fn unsaddled_attack_does_not_grant_brightfield_growth_or_mustang_counters() {
    for name in ["Brightfield Glider", "Brightfield Mustang"] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P0, name, Zone::Battlefield);
        let before = stats(&g, source);
        attack(&mut g, source);
        drain(&mut g);
        assert_eq!(stats(&g, source), before);
        assert_eq!(g.counter_count(source, CounterKind::PlusOne), 0);
        assert!(!g.keywords(source).contains(Keywords::FLYING));
    }
}

#[test]
fn lynx_saddled_attack_searches_only_a_basic_plains_and_keeps_lifelink() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Gloryheath Lynx", Zone::Battlefield);
    let contributor = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let plains = put(&mut g, p, "Plains", Zone::Library);
    let wrong = put(&mut g, p, "Forest", Zone::Library);
    mount(&mut g, source, 0, &[contributor], false);
    attack(&mut g, source);
    ready(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
    reject(&mut g, Action::ChooseSearch { card: wrong });
    act(&mut g, Action::ChooseSearch { card: plains });
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    assert_eq!(g.object_view(plains).zone, Zone::Hand);
    assert_eq!(g.object_view(wrong).zone, Zone::Library);
    assert!(g.keywords(source).contains(Keywords::LIFELINK));
    assert_eq!(stats(&g, source), (2, 3));
}

#[test]
fn sunmare_searches_a_small_nonland_permanent_onto_the_battlefield() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Guardian Sunmare", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let valid = put(&mut g, p, "Swiftfoot Boots", Zone::Library);
    let land = put(&mut g, p, "Forest", Zone::Library);
    let spell = put(&mut g, p, "Shock", Zone::Library);
    let expensive = put(&mut g, p, "Gilded Lotus", Zone::Library);
    mount(&mut g, source, 0, &[a, b], false);
    attack(&mut g, source);
    ready(&mut g, |a| matches!(a, Action::ChooseSearch { .. }));
    for card in [land, spell, expensive] {
        reject(&mut g, Action::ChooseSearch { card });
    }
    act(&mut g, Action::ChooseSearch { card: valid });
    act(&mut g, Action::FinishSearch);
    drain(&mut g);
    assert_eq!(g.object_view(valid).zone, Zone::Battlefield);
    assert!(!g.object_view(valid).tapped);
    assert_eq!(stats(&g, source), (5, 5));
}

#[test]
fn sunmare_ward_counters_an_opposing_spell_unless_its_controller_pays_two() {
    for pays in [false, true] {
        let mut g = scenario(Player::P0);
        let source = put(&mut g, Player::P1, "Guardian Sunmare", Zone::Battlefield);
        let spell = put(&mut g, Player::P0, "Bounce Off", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(source)));
        g.state.players[0].mana = [0, 0, 0, 0, 0, 2];
        ready(&mut g, |a| matches!(a, Action::DeclineResolutionMana));
        choose(&mut g, |a| {
            if pays {
                matches!(a, Action::PayResolutionMana { .. })
            } else {
                matches!(a, Action::DeclineResolutionMana)
            }
        });
        drain(&mut g);
        assert_eq!(
            g.object_view(source).zone,
            if pays { Zone::Hand } else { Zone::Battlefield }
        );
        assert_eq!(g.state.players[0].mana[5], if pays { 0 } else { 2 });
    }
}

#[test]
fn sloth_saddled_attack_untaps_own_creatures_and_grants_temporary_indestructible() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Unswerving Sloth", Zone::Battlefield);
    let a = put(&mut g, p, "Hill Giant", Zone::Battlefield);
    let b = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    g.state.objects[other as usize].tapped = true;
    mount(&mut g, source, 0, &[a, b], false);
    attack(&mut g, source);
    drain(&mut g);
    for id in [source, a, b] {
        assert!(!g.object_view(id).tapped);
    }
    assert!(g.object_view(other).tapped);
    assert!(g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
    g.finish_cleanup();
    assert!(!g.keywords(source).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn canyon_vaulter_grants_flying_for_main_phase_crew_and_saddle_only() {
    for (crew, main) in [(true, true), (true, false), (false, true)] {
        let p = Player::P0;
        let mut g = scenario(p);
        let pilot = put(&mut g, p, "Canyon Vaulter", Zone::Battlefield);
        let source = put(
            &mut g,
            p,
            if crew {
                "Broadcast Rambler"
            } else {
                "Trained Arynx"
            },
            Zone::Battlefield,
        );
        if !main {
            g.state.step = Step::BeginCombat;
        }
        mount(&mut g, source, 0, &[pilot], crew);
        assert_eq!(g.keywords(source).contains(Keywords::FLYING), main);
        assert!(g.object_view(pilot).tapped);
        assert_eq!(stats(&g, pilot), (3, 1));
        g.finish_cleanup();
        assert!(!g.keywords(source).contains(Keywords::FLYING));
    }
}

#[test]
fn captain_boosts_the_union_once_and_its_two_extra_contribution_pays_saddle_four() {
    let p = Player::P0;
    let mut g = scenario(p);
    let captain = put(&mut g, p, "Cloudspire Captain", Zone::Battlefield);
    let own = put(&mut g, p, "Unswerving Sloth", Zone::Battlefield);
    let enemy = put(&mut g, p.other(), "Unswerving Sloth", Zone::Battlefield);
    let vehicle = put(&mut g, p, "Broadcast Rambler", Zone::Battlefield);
    assert_eq!(stats(&g, captain), (2, 3));
    assert_eq!(stats(&g, own), (6, 6));
    assert_eq!(stats(&g, enemy), (5, 5));
    mount(&mut g, own, 0, &[captain], false);
    assert!(g.object_view(captain).tapped);
    both(&mut g, vehicle);
    assert_eq!(stats(&g, vehicle), (6, 5));
    g.move_object(captain, Zone::Graveyard);
    assert_eq!(stats(&g, vehicle), (5, 4));
}

#[test]
fn interface_ace_uses_toughness_and_untaps_only_for_the_first_own_turn_tap() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ace = put(&mut g, p, "Interface Ace", Zone::Battlefield);
    let vehicle = put(&mut g, p, "Hulldrifter", Zone::Battlefield);
    let mount_id = put(&mut g, p, "Guardian Sunmare", Zone::Battlefield);
    assert_eq!(stats(&g, ace), (0, 4));
    mount(&mut g, vehicle, 0, &[ace], true);
    assert!(!g.object_view(ace).tapped);
    mount(&mut g, mount_id, 0, &[ace], false);
    assert!(g.object_view(ace).tapped);
    g.state.objects[ace as usize].tapped = false;
    g.state.active = p.other();
    mount(&mut g, vehicle, 0, &[ace], true);
    assert!(g.object_view(ace).tapped);
}

#[test]
fn bulwark_attack_targets_any_creature_and_sacrifice_protects_only_own_counter_bearers() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ox = put(&mut g, p, "Bulwark Ox", Zone::Battlefield);
    let pilot = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let own = put(&mut g, p, "Aegis Turtle", Zone::Battlefield);
    let other = put(&mut g, p.other(), "Grizzly Bears", Zone::Battlefield);
    g.add_counter(own, CounterKind::Stun, 1);
    mount(&mut g, ox, 1, &[pilot], false);
    attack(&mut g, ox);
    target(&mut g, other);
    drain(&mut g);
    assert_eq!(g.counter_count(other, CounterKind::PlusOne), 1);
    activate(&mut g, ox, 0);
    assert_eq!(g.object_view(ox).zone, Zone::Graveyard);
    drain(&mut g);
    assert!(
        g.keywords(own)
            .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
    );
    assert!(!g.keywords(pilot).contains(Keywords::HEXPROOF));
    assert!(!g.keywords(other).contains(Keywords::INDESTRUCTIBLE));
    g.add_counter(pilot, CounterKind::PlusOne, 1);
    assert!(!g.keywords(pilot).contains(Keywords::HEXPROOF));
    g.finish_cleanup();
    assert!(!g.keywords(own).contains(Keywords::INDESTRUCTIBLE));
}

#[test]
fn basri_pays_white_tap_and_exert_for_one_lifelink_cat_and_cycles_for_own_cat_protection() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Basri, Tomorrow's Champion", Zone::Battlefield);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 0];
    activate(&mut g, source, 0);
    assert!(g.object_view(source).tapped);
    drain(&mut g);
    let cats: Vec<_> = g
        .state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(i as CardId, "Cat"))
            .then_some(i as CardId)
        })
        .collect();
    assert_eq!(cats.len(), 1);
    let cat = cats[0];
    assert_eq!(stats(&g, cat), (1, 1));
    assert_eq!(g.object_view(cat).colors, 1);
    assert!(g.keywords(cat).contains(Keywords::LIFELINK));
    g.state.turn += 2;
    g.untap_active_player();
    assert!(g.object_view(source).tapped);
    g.state.turn += 2;
    g.untap_active_player();
    assert!(!g.object_view(source).tapped);
    let enemy = put(&mut g, p.other(), "Savannah Lions", Zone::Battlefield);
    let other = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let cycling = put(&mut g, p, "Basri, Tomorrow's Champion", Zone::Hand);
    g.state.players[0].mana = [1, 0, 0, 0, 0, 2];
    activate(&mut g, cycling, 1);
    assert_eq!(g.object_view(cycling).zone, Zone::Graveyard);
    drain(&mut g);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(g.state.players[0].cards_drawn_this_turn, 1);
    assert!(
        g.keywords(cat)
            .contains(Keywords::HEXPROOF.union(Keywords::INDESTRUCTIBLE))
    );
    assert!(!g.keywords(enemy).contains(Keywords::HEXPROOF));
    assert!(!g.keywords(other).contains(Keywords::INDESTRUCTIBLE));
    g.finish_cleanup();
    assert!(!g.keywords(cat).contains(Keywords::HEXPROOF));
    assert!(g.keywords(cat).contains(Keywords::LIFELINK));
}

#[test]
fn voyager_glidecar_scries_one_then_taps_three_other_creatures_for_flying_and_a_counter() {
    let p = Player::P0;
    let mut g = scenario(p);
    let top = put(&mut g, p, "Shock", Zone::Library);
    let source = put(&mut g, p, "Voyager Glidecar", Zone::Hand);
    cast(&mut g, source, None);
    ready(&mut g, |a| matches!(a, Action::ArrangeLibrary { .. }));
    assert_eq!(
        g.observe(p)
            .library_choice
            .as_ref()
            .unwrap()
            .remaining
            .len(),
        1
    );
    act(
        &mut g,
        Action::ArrangeLibrary {
            card: top,
            destination: LibraryDestination::Top,
        },
    );
    drain(&mut g);
    assert_eq!(g.top_library_card(p), Some(top));
    let first = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    let second = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    reject(
        &mut g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    let third = put(&mut g, p, "Grizzly Bears", Zone::Battlefield);
    activate(&mut g, source, 0);
    let snapshot = g.snapshot();
    let mut branch = g.clone();
    for id in [first, second, third] {
        let object = g.object_ref(id);
        act(&mut g, Action::ChooseAdditionalCost { object });
    }
    pay(&mut g);
    for id in [first, second, third] {
        assert!(g.object_view(id).tapped);
    }
    drain(&mut g);
    assert!(g.is_creature(source));
    assert_eq!(stats(&g, source), (3, 4));
    assert!(g.keywords(source).contains(Keywords::FLYING));
    branch.restore(&snapshot).unwrap();
    assert!(!branch.object_view(first).tapped);
    assert!(!branch.is_creature(source));
    g.finish_cleanup();
    assert!(!g.is_creature(source));
    assert_eq!(g.counter_count(source, CounterKind::PlusOne), 1);
    g.state.active = p;
    g.state.priority = p;
    g.state.pending = Pending::Priority;
    g.state.step = Step::Main1;
    g.state.objects[first as usize].tapped = false;
    mount(&mut g, source, 1, &[first], true);
    assert_eq!(stats(&g, source), (3, 4));
    assert!(!g.keywords(source).contains(Keywords::FLYING));
}
