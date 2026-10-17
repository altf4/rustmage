//! Paid consumer interactions supplement the already reviewed shared core tests.
use super::ability_timeline_tests::{cast, checked, named, next_turn, settle, setup};
use super::*;

fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn until(g: &mut Game, stop: impl Fn(&Game) -> bool) {
    for _ in 0..80 {
        if stop(g) {
            return;
        }
        checked(g, Action::Pass);
    }
    panic!("missing phase: {:?} {:?}", g.state.step, g.state.pending);
}
fn paid(g: &mut Game, p: Player, name: &str, mana: Mana) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    g.state.players[p.index()].mana = mana;
    cast(g, id, None, &[]);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    assert_eq!(g.object_view(id).zone, Zone::Battlefield);
    id
}
fn activation(id: CardId, ability: u16, target: Option<CardId>, payment: Mana) -> Action {
    Action::Activate {
        card: id,
        ability,
        target: target.map(Target::Card),
        payment,
        counter_payment: Counters::default(),
    }
}
fn own_main_after_untap(g: &mut Game, p: Player) {
    next_turn(g);
    next_turn(g);
    until(g, |g| {
        g.state.active == p && g.state.step == Step::Main1 && g.state.pending == Pending::Priority
    });
}

#[test]
fn fdn_five_bounded_acceptance_axe_double_strike_trample_and_bomber_first_strike_sacrifice() {
    for p in [Player::P0, Player::P1] {
        for grant in [false, true] {
            let mut g = setup(p);
            let bomber = paid(&mut g, p, "Dropkick Bomber", [0, 0, 0, 1, 0, 2]);
            let goblin = paid(&mut g, p, "Goblin Smuggler", [0, 0, 0, 1, 0, 2]);
            let axe = paid(&mut g, p, "Leyline Axe", [0, 0, 0, 0, 0, 4]);
            let hawk = named(&mut g, p.other(), "Healer's Hawk", Zone::Battlefield);
            assert_eq!(
                g.stats(bomber),
                Stats {
                    power: 2,
                    toughness: 3
                }
            );
            assert_eq!(
                g.stats(goblin),
                Stats {
                    power: 3,
                    toughness: 3
                }
            );
            g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
            reject(&mut g, activation(axe, 0, Some(goblin), [0, 0, 0, 0, 0, 2]));
            reject(&mut g, activation(axe, 0, Some(hawk), [0, 0, 0, 0, 0, 3]));
            checked(&mut g, activation(axe, 0, Some(goblin), [0, 0, 0, 0, 0, 3]));
            settle(&mut g);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert_eq!(
                g.stats(goblin),
                Stats {
                    power: 4,
                    toughness: 4
                }
            );
            if grant {
                g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 0];
                checked(
                    &mut g,
                    activation(bomber, 0, Some(goblin), [0, 0, 0, 1, 0, 0]),
                );
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert!(g.keywords(goblin).contains(Keywords::FLYING));
            }
            // Smuggler's printed haste permits this attack on its actual cast turn.
            until(&mut g, |g| g.state.pending == Pending::Attackers);
            checked(&mut g, Action::Attack { card: goblin });
            checked(&mut g, Action::FinishAttackers);
            until(&mut g, |g| g.state.pending == Pending::Blockers);
            checked(
                &mut g,
                Action::Block {
                    blocker: hawk,
                    attacker: goblin,
                },
            );
            checked(&mut g, Action::FinishBlockers);
            until(&mut g, |g| g.state.pending == Pending::Damage);
            assert_eq!(g.state.step, Step::FirstStrikeDamage);
            checked(
                &mut g,
                Action::AssignDamage {
                    target: Target::Card(hawk),
                    amount: 1,
                },
            );
            assert_eq!(g.state.players[p.other().index()].life, 17);
            assert_eq!(g.object_view(hawk).zone, Zone::Graveyard);
            assert_eq!(
                g.state.stack.len(),
                usize::from(grant),
                "one trigger despite damage to two recipients"
            );
            settle(&mut g);
            assert_eq!(
                g.object_view(goblin).zone,
                if grant {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
            assert_eq!(g.object_view(axe).zone, Zone::Battlefield);
            if grant {
                assert!(g.object_view(axe).attached_to.is_none());
            }
            until(&mut g, |g| {
                g.state.step == Step::Main2 && g.state.pending == Pending::Priority
            });
            assert_eq!(
                g.state.players[p.other().index()].life,
                if grant { 17 } else { 13 }
            );
        }
    }
}

#[test]
fn fdn_five_bounded_acceptance_tome_real_untaps_draw_payment_and_fourth_cost_exile() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let tome = paid(&mut g, p, "Mazemind Tome", [0, 0, 0, 0, 0, 2]);
        let starting_life = g.state.players[p.index()].life;
        for number in 1..=4 {
            if number > 1 {
                own_main_after_untap(&mut g, p);
            }
            assert!(!g.object_view(tome).tapped);
            let scry = number == 2;
            let ability = if scry { 0 } else { 1 };
            let payment = if scry { [0; 6] } else { [0, 0, 0, 0, 0, 2] };
            g.state.players[p.index()].mana = payment;
            let hand = g.hand_count(p);
            let library = g.state.players[p.index()].library.len();
            checked(&mut g, activation(tome, ability, None, payment));
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(g.object_view(tome).tapped);
            assert_eq!(g.counter_count(tome, CounterKind::Page), number);
            assert_eq!(
                g.hand_count(p),
                hand,
                "counter is paid before draw resolves"
            );
            reject(&mut g, activation(tome, 0, None, [0; 6]));
            if number == 4 {
                assert_eq!(g.state.stack.len(), 2, "state trigger above paid draw");
            }
            if scry {
                until(&mut g, |g| {
                    matches!(g.state.pending, Pending::EffectLibrary { .. })
                });
                let top = *g.state.players[p.index()].library.last().unwrap();
                assert!(g.observe(p).library_choice.is_some());
                assert!(g.observe(p.other()).library_choice.is_none());
                assert!(g.observe(p.other()).actions.is_empty());
                checked(
                    &mut g,
                    Action::ArrangeLibrary {
                        card: top,
                        destination: LibraryDestination::Bottom,
                    },
                );
                settle(&mut g);
                assert_eq!(g.state.players[p.index()].library[0], top);
            } else {
                settle(&mut g);
            }
            assert_eq!(g.hand_count(p), hand + usize::from(!scry));
            assert_eq!(
                g.state.players[p.index()].library.len(),
                library - usize::from(!scry)
            );
            assert_eq!(
                g.state.players[p.index()].life,
                starting_life + if number == 4 { 4 } else { 0 }
            );
        }
        assert_eq!(g.object_view(tome).zone, Zone::Exile);
    }
}

#[test]
fn fdn_five_bounded_acceptance_demolition_real_land_drop_colorless_mana_and_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let field = named(&mut g, p, "Demolition Field", Zone::Hand);
        let another = named(&mut g, p, "Forest", Zone::Hand);
        let victim = named(&mut g, p.other(), "Rogue's Passage", Zone::Battlefield);
        g.state.players[p.index()].mana = [0; 6];
        checked(&mut g, Action::PlayLand { card: field });
        reject(&mut g, Action::PlayLand { card: another });
        let mana = Action::ActivateMana {
            card: field,
            ability: 0,
            payment: [0; 6],
            counter_payment: Counters::default(),
            production: [0, 0, 0, 0, 0, 1],
        };
        checked(&mut g, mana.clone());
        assert_eq!(g.state.players[p.index()].mana, [0, 0, 0, 0, 0, 1]);
        reject(&mut g, mana);
        own_main_after_untap(&mut g, p);
        assert!(!g.object_view(field).tapped);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        checked(
            &mut g,
            activation(field, 1, Some(victim), [0, 0, 0, 0, 0, 2]),
        );
        assert_eq!(g.object_view(field).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        until(&mut g, |g| {
            matches!(g.state.pending, Pending::OptionalSearch(_))
        });
        assert_eq!(g.actor(), Some(p.other()));
        assert_eq!(g.object_view(victim).zone, Zone::Graveyard);
        let rng = g.state.rng;
        checked(&mut g, Action::ChooseOptional { accept: false });
        assert_eq!(g.actor(), Some(p));
        checked(&mut g, Action::ChooseOptional { accept: false });
        settle(&mut g);
        assert_eq!(g.state.rng, rng);
    }
}

#[test]
fn fdn_five_bounded_acceptance_ooze_paid_cast_attack_and_real_lethal_combat_history() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ooze = paid(&mut g, p, "Predator Ooze", [0, 0, 0, 0, 3, 0]);
        let bear = named(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        own_main_after_untap(&mut g, p);
        until(&mut g, |g| g.state.pending == Pending::Attackers);
        checked(&mut g, Action::Attack { card: ooze });
        checked(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert_eq!(g.counter_count(ooze, CounterKind::PlusOne), 1);
        until(&mut g, |g| g.state.pending == Pending::Blockers);
        checked(
            &mut g,
            Action::Block {
                blocker: bear,
                attacker: ooze,
            },
        );
        checked(&mut g, Action::FinishBlockers);
        until(&mut g, |g| {
            g.state.step == Step::CombatDamage && g.state.pending == Pending::Priority
        });
        settle(&mut g);
        assert_eq!(g.object_view(bear).zone, Zone::Graveyard);
        assert_eq!(g.object_view(ooze).zone, Zone::Battlefield);
        assert_eq!(g.object_view(ooze).damage, 2);
        assert_eq!(g.counter_count(ooze, CounterKind::PlusOne), 2);
        assert_eq!(
            g.stats(ooze),
            Stats {
                power: 3,
                toughness: 3
            }
        );
        let history = g.observe(p).creature_damage_history;
        assert!(
            history
                .iter()
                .any(|row| row.source.card == ooze && row.recipient.card == bear)
        );
    }
}
