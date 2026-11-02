//! Whole-card proofs for inherited Vehicles, Mounts and their supporting cards.
mod support;
use rustmage as _;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;
fn activation(card: CardId, ability: u16) -> Action {
    Action::AnnounceActivate {
        card,
        ability,
        counter_payment: Counters::default(),
    }
}
fn drain(g: &mut Game, target: Option<Target>) {
    for _ in 0..48 {
        g.prepare_triggers();
        if g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
        {
            return;
        }
        choose(g, |a| {
            matches!(a, Action::Pass | Action::PutTrigger { .. })
                || matches!(a,Action::ChooseTarget{target:t} if Some(*t)==target)
        });
    }
    panic!("Vehicle/Mount resolution bound");
}
fn saddle(g: &mut Game, source: CardId, pilot: CardId) {
    act(g, activation(source, 0));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(g);
    }
    let object = g.object_ref(pilot);
    act(g, Action::ChooseSaddler { object });
    act(g, Action::FinishSaddle);
    drain(g, None);
    assert!(g.object_view(source).saddled);
}
fn attack(g: &mut Game, source: CardId) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.priority = g.state.active;
    act(g, Action::Attack { card: source });
    act(g, Action::FinishAttackers);
}

#[test]
fn burner_rocket_flashes_on_opponents_turn_buffs_only_own_target_and_crews_for_one() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p.other());
        let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let wrong = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let source = put(&mut g, p, "Burner Rocket", Zone::Hand);
        act(&mut g, Action::Pass);
        cast(&mut g, source, None);
        for _ in 0..16 {
            g.prepare_triggers();
            if g.legal_actions().contains(&Action::ChooseTarget {
                target: Target::Card(own),
            }) {
                break;
            }
            choose(&mut g, |a| {
                matches!(a, Action::Pass | Action::PutTrigger { .. })
            });
        }
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(wrong),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        drain(&mut g, None);
        assert_eq!(stats(&g, own), (4, 2));
        assert!(g.keywords(own).contains(Keywords::TRAMPLE));
        assert_eq!(stats(&g, wrong), (2, 2));
        assert!(!g.is_creature(source));
        assert!(g.keywords(source).contains(Keywords::FLASH));
        let crewer = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
        if g.actor() != Some(p) {
            act(&mut g, Action::Pass);
        }
        act(&mut g, activation(source, 0));
        if matches!(g.state.pending, Pending::PayCosts { .. }) {
            pay(&mut g);
        }
        let object = g.object_ref(crewer);
        act(&mut g, Action::ChooseCrewer { object });
        act(&mut g, Action::FinishCrew);
        drain(&mut g, None);
        assert_eq!(stats(&g, source), (3, 1));
        assert!(g.object_view(crewer).tapped);
        assert!(!g.object_view(own).tapped);
        g.finish_cleanup();
        assert!(!g.is_creature(source));
        assert_eq!(stats(&g, own), (2, 2));
        assert!(!g.keywords(own).contains(Keywords::TRAMPLE));
    }
}

#[test]
fn clamorous_ironclad_needs_three_crew_power_and_keeps_menace_until_animation_expires() {
    let p = Player::P0;
    let mut g = scenario(p);
    let source = put(&mut g, p, "Clamorous Ironclad", Zone::Battlefield);
    let bear = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let mako = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
    act(&mut g, activation(source, 0));
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    let object = g.object_ref(bear);
    act(&mut g, Action::ChooseCrewer { object });
    reject(&mut g, Action::FinishCrew);
    let object = g.object_ref(mako);
    act(&mut g, Action::ChooseCrewer { object });
    act(&mut g, Action::FinishCrew);
    drain(&mut g, None);
    assert_eq!(stats(&g, source), (6, 3));
    assert_eq!(g.minimum_blockers(source), 2);
    assert!(g.object_view(bear).tapped && g.object_view(mako).tapped);
    g.finish_cleanup();
    assert!(!g.is_creature(source));
    assert!(g.keywords(source).contains(Keywords::MENACE));
}

#[test]
fn chitin_and_ironclad_cycling_pay_the_printed_cost_and_ignore_spell_cost_reductions() {
    for p in [Player::P0, Player::P1] {
        for (name, ability, mana) in [
            ("Clamorous Ironclad", 1, [0, 0, 0, 1, 0, 0]),
            ("Chitin Gravestalker", 0, [0, 0, 0, 0, 0, 2]),
        ] {
            let mut g = scenario(p);
            for _ in 0..6 {
                put(&mut g, p, "Pactdoll Terror", Zone::Graveyard);
            }
            let card = put(&mut g, p, name, Zone::Hand);
            g.state.players[p.index()].mana = if name == "Chitin Gravestalker" {
                [0, 0, 0, 0, 0, 1]
            } else {
                [0; 6]
            };
            reject(&mut g, activation(card, ability));
            g.state.players[p.index()].mana = mana;
            act(&mut g, activation(card, ability));
            pay(&mut g);
            assert_eq!(g.object_view(card).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 0);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            drain(&mut g, None);
            assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        }
    }
}

#[test]
fn chitin_counts_owned_artifact_or_creature_cards_once_and_not_lands_or_foreign_cards() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Pactdoll Terror", Zone::Graveyard);
        put(&mut g, p, "Grim Bauble", Zone::Graveyard);
        put(&mut g, p, "Aegis Turtle", Zone::Graveyard);
        put(&mut g, p, "Mountain", Zone::Graveyard);
        put(&mut g, p.other(), "Pactdoll Terror", Zone::Graveyard);
        let card = put(&mut g, p, "Chitin Gravestalker", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 1];
        reject(&mut g, Action::AnnounceCast { card });
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 2];
        cast(&mut g, card, None);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        drain(&mut g, None);
        assert_eq!(stats(&g, card), (5, 4));
        assert_eq!(g.object_view(card).zone, Zone::Battlefield);
    }
}

#[test]
fn chitin_generic_reduction_floors_at_zero_but_never_removes_the_black_symbol() {
    let p = Player::P0;
    let mut g = scenario(p);
    for _ in 0..6 {
        put(&mut g, p, "Aegis Turtle", Zone::Graveyard);
    }
    let card = put(&mut g, p, "Chitin Gravestalker", Zone::Hand);
    g.state.players[0].mana = [0, 0, 0, 0, 0, 1];
    reject(&mut g, Action::AnnounceCast { card });
    g.state.players[0].mana = [0, 0, 1, 0, 0, 0];
    cast(&mut g, card, None);
    drain(&mut g, None);
    assert_eq!(g.state.players[0].mana, [0; 6]);
    assert_eq!(stats(&g, card), (5, 4));
}

#[test]
fn gilded_ghoda_must_attack_while_saddled_and_creates_a_real_spendable_treasure() {
    for p in [Player::P0, Player::P1] {
        for saddled in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Gilded Ghoda", Zone::Battlefield);
            let pilot = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
            assert_eq!(stats(&g, source), (2, 2));
            if saddled {
                g.state.step = Step::Upkeep;
                reject(&mut g, activation(source, 0));
                g.state.step = Step::Main1;
                saddle(&mut g, source, pilot);
            }
            attack(&mut g, source);
            if saddled {
                g.move_object(source, Zone::Hand);
            }
            drain(&mut g, None);
            let treasures: Vec<_> = g
                .state
                .objects
                .iter()
                .enumerate()
                .filter(|(id, o)| {
                    o.zone == Zone::Battlefield
                        && o.controller == p
                        && o.is_token()
                        && g.has_subtype(*id as CardId, "Treasure")
                })
                .map(|(id, _)| id as CardId)
                .collect();
            assert_eq!(treasures.len(), usize::from(saddled));
            for token in treasures {
                assert_eq!(g.object_view(token).colors, 0);
                assert!(!g.object_view(token).tapped);
                g.state.players[p.index()].mana = [0; 6];
                if g.actor() != Some(p) {
                    act(&mut g, Action::Pass);
                }
                choose(
                    &mut g,
                    |a| matches!(a,Action::ActivateMana{card,production,..} if *card==token&&*production==[0,0,0,1,0,0]),
                );
                assert_eq!(g.state.objects[token as usize].zone, Zone::Outside);
                assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 1, 0, 0]);
            }
        }
    }
}

#[test]
fn dracosaur_auxiliary_has_flying_haste_and_deals_two_only_when_attacking_saddled() {
    for saddled in [false, true] {
        for walker in [false, true] {
            let p = Player::P0;
            let mut g = scenario(p);
            let source = put(&mut g, p, "Dracosaur Auxiliary", Zone::Hand);
            cast(&mut g, source, None);
            drain(&mut g, None);
            assert_eq!(stats(&g, source), (4, 4));
            assert!(
                g.keywords(source)
                    .contains(Keywords::FLYING.union(Keywords::HASTE))
            );
            let pilot = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let extra = put(&mut g, p, "Marauding Mako", Zone::Battlefield);
            let target = if walker {
                let id = put(
                    &mut g,
                    p.other(),
                    "Chandra, Spark Hunter",
                    Zone::Battlefield,
                );
                g.state.objects[id as usize].counters.loyalty = 7;
                Target::Card(id)
            } else {
                Target::Player(p.other())
            };
            if saddled {
                act(&mut g, activation(source, 0));
                if matches!(g.state.pending, Pending::PayCosts { .. }) {
                    pay(&mut g);
                }
                let object = g.object_ref(pilot);
                act(&mut g, Action::ChooseSaddler { object });
                reject(&mut g, Action::FinishSaddle);
                let object = g.object_ref(extra);
                act(&mut g, Action::ChooseSaddler { object });
                act(&mut g, Action::FinishSaddle);
                drain(&mut g, None);
            }
            attack(&mut g, source);
            drain(&mut g, Some(target));
            match target {
                Target::Player(victim) => assert_eq!(
                    g.state.players[victim.index()].life,
                    if saddled { 18 } else { 20 }
                ),
                Target::Card(id) => assert_eq!(
                    g.object_view(id).counters.loyalty,
                    if saddled { 5 } else { 7 }
                ),
                _ => unreachable!(),
            }
            assert!(g.object_view(source).tapped);
            if saddled {
                assert!(g.object_view(pilot).tapped);
            }
        }
    }
}

#[test]
fn count_on_luck_triggers_only_on_own_upkeep_and_permits_a_paid_play_after_source_leaves() {
    for p in [Player::P0, Player::P1] {
        for own in [false, true] {
            let mut g = scenario(p);
            let source = put(&mut g, p, "Count on Luck", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 0, 3, 0, 0];
            cast(&mut g, source, None);
            drain(&mut g, None);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            let top = put(&mut g, p, "Shock", Zone::Library);
            g.state.active = if own { p } else { p.other() };
            g.begin_step(Step::Upkeep);
            drain(&mut g, None);
            assert_eq!(
                g.object_view(top).zone,
                if own { Zone::Exile } else { Zone::Library }
            );
            if own {
                let mut expired = g.clone();
                expired.finish_cleanup();
                expired.state.active = p;
                expired.state.priority = p;
                expired.state.step = Step::Main1;
                expired.state.pending = Pending::Priority;
                expired.state.players[p.index()].mana = [8; 6];
                reject(&mut expired, Action::AnnounceExile { card: top });
                g.move_object(source, Zone::Hand);
                g.state.active = p;
                g.state.priority = p;
                g.state.step = Step::Main1;
                g.state.pending = Pending::Priority;
                reject(&mut g, Action::AnnounceExile { card: top });
                g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
                act(&mut g, Action::AnnounceExile { card: top });
                act(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Player(p.other()),
                    },
                );
                pay(&mut g);
                drain(&mut g, None);
                assert_eq!(g.state.players[p.other().index()].life, 18);
            }
        }
    }
}
