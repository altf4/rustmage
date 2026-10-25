//! Paid interactions across copied spells, cast card copies, Sneak and harness.
use super::ability_world::CharacteristicWorld;
use super::assurance::big_helpers::{cast, checked, named, next_turn, setup};
use super::msh_thirteenth_combined_tests::owned_checked_step;
use super::*;

const KING: &str = "The Kingpin of Crime";
const ZEMO: &str = "Baron Helmut Zemo";
const MIND: &str = "The Mind Stone";

fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.announcing.is_none()
        && g.state.resolving.is_none()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
}
fn priority(g: &mut Game, p: Player) {
    if g.actor() != Some(p) {
        checked(g, Action::Pass);
    }
    assert_eq!(g.actor(), Some(p));
}
fn step(g: &mut Game) {
    let a = g
        .legal_actions()
        .into_iter()
        .find(|a| {
            matches!(
                a,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::FinishTargets
                    | Action::PayCosts { .. }
                    | Action::ChooseCastCost { .. }
                    | Action::FinishAttackers
                    | Action::FinishBlockers
                    | Action::FinishStackTargets
                    | Action::ChooseOptional { accept: false }
                    | Action::DeclineResolutionMana
            )
        })
        .unwrap_or_else(|| panic!("unhandled {:?}", g.state.pending));
    checked(g, a);
}
fn until(g: &mut Game, done: impl Fn(&Game) -> bool) {
    g.prepare_triggers();
    for _ in 0..400 {
        if done(g) {
            return;
        }
        step(g);
    }
    panic!("unfinished {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    while g.state.announcing.is_some() {
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| {
                matches!(
                    a,
                    Action::FinishTargets | Action::PayCosts { .. } | Action::ChooseCastCost { .. }
                )
            })
            .unwrap();
        checked(g, a);
    }
}
fn packet(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    priority(g, p);
    let id = named(g, p, name, Zone::Hand);
    let (mut mana, generic) = Card::from_name(name).unwrap().cost();
    mana[5] += generic;
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, targets);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn paid(g: &mut Game, p: Player, name: &str, targets: &[Target]) -> CardId {
    let id = packet(g, p, name, targets);
    // Setup spells decline Extort; test spells below explicitly pay every event.
    until(g, quiet);
    id
}
fn fresh(p: Player) -> Game {
    let mut g = setup(p);
    for state in &mut g.state.players {
        state.mana = [0; 6];
    }
    g
}
fn orphan(g: Game, a: Action) -> Game {
    let actor = g.actor().unwrap();
    let index = g.legal_actions().iter().position(|x| *x == a).unwrap();
    owned_checked_step(g, actor, index, a)
}
fn extort_all(mut g: Game, p: Player) -> (Game, usize) {
    g.prepare_triggers();
    let mut events = 0;
    for _ in 0..300 {
        if quiet(&g) {
            return (g, events);
        }
        if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
            assert_eq!(g.actor(), Some(p));
            events += 1;
            g = orphan(
                g,
                Action::PayResolutionMana {
                    payment: [1, 0, 0, 0, 0, 0],
                },
            );
        } else {
            step(&mut g);
        }
    }
    panic!("unfinished extort/copy continuation");
}
fn trigger_count(g: &Game, source: CardId, index: u16) -> usize {
    g.state
        .pending_triggers
        .iter()
        .chain(&g.state.ordering_triggers)
        .chain(&g.state.stack)
        .filter(|s| {
            s.card == source
                && matches!(s.kind, StackKind::TriggeredAbility { ability } if ability == index)
        })
        .count()
}

#[test]
fn msh_final_copy_photon_three_spell_copies_produce_only_one_actual_cast_extort() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        paid(&mut g, p, KING, &[]);
        let target = paid(&mut g, p, "Three Tree Mascot", &[]);
        g.add_counter(target, CounterKind::PlusOne, 5);
        let before = g.state.players[p.index()].spells_cast_this_turn;
        let barrage = named(&mut g, p, "Photon Blast Barrage", Zone::Hand);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 3];
        checked(&mut g, Action::AnnounceCast { card: barrage });
        checked(&mut g, Action::ChooseX { value: 3 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        finish(&mut g);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 0];
        let (g, extorts) = extort_all(g, p);
        assert_eq!(extorts, 1);
        assert_eq!(g.state.players[p.index()].spells_cast_this_turn, before + 1);
        assert_eq!(g.state.objects[target as usize].damage, 4);
        assert_eq!((g.life(p), g.life(p.other())), (21, 19));
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        assert!(g.on_battlefield(target));
    }
}

#[test]
fn msh_final_copy_zemo_three_cast_card_copies_each_extort_without_hand_connive() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        paid(&mut g, p, KING, &[]);
        let zemo = paid(&mut g, p, ZEMO, &[]);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1 && quiet(g)
        });
        let cards = (0..8)
            .map(|_| named(&mut g, p, "Murder", Zone::Graveyard))
            .collect::<Vec<_>>();
        let victims = (0..3)
            .map(|_| named(&mut g, p.other(), "Hill Giant", Zone::Battlefield))
            .collect::<Vec<_>>();
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: zemo });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, quiet);
        priority(&mut g, p);
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: zemo,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::AdditionalCost { .. })
        });
        for &card in &cards {
            let object = g.object_ref(card);
            checked(&mut g, Action::ChooseAdditionalCost { object });
        }
        checked(&mut g, Action::FinishGraveyardSymbolCost);
        finish(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionCasts { .. })
        });
        let casts_before = g.state.players[p.index()].spells_cast_this_turn;
        g.state.players[p.index()].mana = [3, 0, 0, 0, 0, 0];
        for &victim in &victims {
            let object = g.state.resolution_free_casts.as_ref().unwrap().remaining[0];
            checked(
                &mut g,
                Action::CastExiledDuringResolution {
                    object,
                    face: CastFace::Normal,
                },
            );
            checked(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(victim),
                },
            );
            finish(&mut g);
        }
        assert_eq!(
            g.state
                .resolution_free_casts
                .as_ref()
                .unwrap()
                .casts_remaining,
            Some(0)
        );
        checked(&mut g, Action::FinishResolutionCasts);
        let (g, extorts) = extort_all(g, p);
        assert_eq!(extorts, 3);
        assert_eq!(
            g.state.players[p.index()].spells_cast_this_turn,
            casts_before + 3
        );
        assert_eq!((g.life(p), g.life(p.other())), (23, 17));
        assert_eq!(g.counter_count(zemo, CounterKind::PlusOne), 0);
        assert!(
            cards
                .iter()
                .all(|&card| g.object_view(card).zone == Zone::Exile)
        );
        assert!(
            victims
                .iter()
                .all(|&card| g.object_view(card).zone == Zone::Graveyard)
        );
    }
}

#[test]
fn msh_final_copy_sneak_cast_extorts_but_putting_attacking_does_not_retrigger_kingpin() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let king = paid(&mut g, p, KING, &[]);
        let attacker = paid(&mut g, p, "Vampire Nighthawk", &[]);
        let victim = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1 && quiet(g)
        });
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: attacker });
        checked(&mut g, Action::FinishAttackers);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::EffectOptional { .. })
        });
        checked(&mut g, Action::ChooseOptional { accept: true });
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        assert_eq!(g.life(p), 18);
        checked(&mut g, Action::FinishBlockers);
        priority(&mut g, p);
        let elektra = named(&mut g, p, "Elektra, Daughter of the Hand", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 2, 0, 0, 1];
        checked(
            &mut g,
            Action::AnnounceSneak {
                card: elektra,
                permission: None,
            },
        );
        let object = g.object_ref(attacker);
        g = orphan(g, Action::ChooseAdditionalCost { object });
        finish(&mut g);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::ResolutionMana { .. })
        });
        assert_eq!(trigger_count(&g, king, 1), 0);
        checked(&mut g, Action::DeclineResolutionMana);
        until(&mut g, |g| {
            g.state
                .announcing
                .as_ref()
                .is_some_and(|a| a.item.card == elektra)
        });
        assert_eq!(g.object_view(attacker).zone, Zone::Hand);
        assert!(g.state.attackers.iter().any(|a| a.card == elektra));
        assert!(!g.state.attacked_this_turn.contains(&g.object_ref(elektra)));
        assert_eq!(trigger_count(&g, king, 1), 0);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        finish(&mut g);
        until(&mut g, quiet);
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        assert_eq!(g.life(p), 18);
        assert_eq!(trigger_count(&g, king, 1), 0);
    }
}

#[test]
fn msh_final_copy_secret_layer_hides_mind_trigger_but_expiry_preserves_harness_designation() {
    for p in [Player::P0, Player::P1] {
        let mut g = fresh(p);
        let mind = paid(&mut g, p, MIND, &[]);
        let bear = paid(&mut g, p, "Bear Cub", &[]);
        let mind_ref = g.object_ref(mind);
        priority(&mut g, p);
        g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 5];
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: mind,
                ability: 1,
                counter_payment: Counters::default(),
            },
        );
        finish(&mut g);
        until(&mut g, quiet);
        assert_eq!(g.object_view(mind).harnessed, Some(true));
        paid(&mut g, p, "I Am Iron Man", &[Target::Card(mind)]);
        assert!(g.object_view(mind).is_creature());
        let aura = packet(&mut g, p, "Secret Invasion", &[Target::Card(mind)]);
        until(&mut g, |g| {
            g.state.announcing.as_ref().is_some_and(|a| {
                a.item.card == aura && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
            })
        });
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        until(&mut g, quiet);
        assert_eq!(g.object_view(bear).zone, Zone::Exile);
        assert_eq!(g.object_view(mind).name(), "Bear Cub");
        assert_eq!(g.object_view(mind).harnessed, Some(true));
        assert_eq!(g.object_ref(mind), mind_ref);
        until(&mut g, |g| g.state.step == Step::End && quiet(g));
        assert_eq!(trigger_count(&g, mind, 0), 0);
        paid(&mut g, p, "Disenchant", &[Target::Card(aura)]);
        assert_eq!(g.object_view(bear).zone, Zone::Battlefield);
        assert_eq!(g.object_view(mind).card, Card::from_name(MIND).unwrap());
        assert_eq!(g.object_view(mind).harnessed, Some(true));
        assert_eq!(g.object_ref(mind), mind_ref);
        next_turn(&mut g);
        next_turn(&mut g);
        until(&mut g, |g| {
            g.state.active == p && g.state.step == Step::Main1 && quiet(g)
        });
        let bear_ref = g.object_ref(bear);
        until(&mut g, |g| {
            g.state.announcing.as_ref().is_some_and(|a| {
                a.item.card == mind && matches!(a.item.kind, StackKind::TriggeredAbility { .. })
            })
        });
        g = orphan(
            g,
            Action::ChooseTarget {
                target: Target::Card(bear),
            },
        );
        finish(&mut g);
        until(&mut g, quiet);
        assert_ne!(g.object_ref(bear), bear_ref);
        assert_eq!(g.object_ref(mind), mind_ref);
        assert_eq!(g.object_view(mind).harnessed, Some(true));
    }
}
