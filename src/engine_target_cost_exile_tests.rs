//! Latest-main integration witnesses: target discounts compose with owned exile rights.
use super::tests::{act, scenario, spawn};
use super::*;

fn named(g: &mut Game, owner: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, owner, Card::from_name(name).unwrap(), zone)
}
fn replay(g: &mut Game, action: Action) {
    let actor = g.actor().unwrap();
    assert!(g.legal_actions().contains(&action), "{action:?}");
    let saved = g.snapshot();
    let old_revision = g.revision();
    let mut fork = g.clone();
    act(&mut fork, action.clone());
    act(g, action.clone());
    let expected = g.snapshot();
    assert_eq!(fork.snapshot(), expected);
    g.restore(&saved).unwrap();
    assert_eq!(
        g.apply_index(actor, old_revision, 0),
        Err(Error::StaleRevision)
    );
    act(g, action);
    assert_eq!(g.snapshot(), expected);
}
fn reject(g: &mut Game, action: Action) {
    let saved = g.snapshot();
    let revision = g.revision();
    assert_eq!(
        g.apply(g.actor().unwrap(), action),
        Err(Error::IllegalAction)
    );
    assert_eq!(g.snapshot(), saved);
    assert_eq!(g.revision(), revision);
}
fn resolve(g: &mut Game) {
    for _ in 0..20 {
        if g.state.stack.is_empty() && g.state.resolving.is_none() {
            return;
        }
        replay(g, Action::Pass);
    }
    panic!("resolution did not finish");
}

#[test]
fn real_exile_rights_use_only_staged_discounted_casts_and_preserve_normal_origins() {
    for owner in [Player::P0, Player::P1] {
        // Independent expected payments from printed text, not the generated reduction rule.
        for (case, name, payment) in [
            (0, "Ajani's Response", [1, 0, 0, 0, 0, 1]),
            (1, "Brush Off", [0, 1, 0, 0, 0, 1]),
            (2, "Run Over", [0, 0, 0, 0, 1, 0]),
            (3, "Swampsnare Trap", [0, 0, 1, 0, 0, 1]),
        ] {
            let mut g = scenario();
            g.state.active = if case == 1 { owner.other() } else { owner };
            g.state.priority = owner;
            let target = match case {
                1 => {
                    let id = named(&mut g, owner.other(), "Shock", Zone::Hand);
                    let item = g.cast_item(owner.other(), id, [0; 6]);
                    g.state.stack.push(item);
                    Target::Spell(id)
                }
                2 => Target::Card(named(&mut g, owner, "Three Tree Mascot", Zone::Battlefield)),
                3 => Target::Card(named(
                    &mut g,
                    owner.other(),
                    "Malevolent Chandelier",
                    Zone::Battlefield,
                )),
                _ => {
                    let id = named(&mut g, owner.other(), "Bear Cub", Zone::Battlefield);
                    g.state.objects[id as usize].tapped = true;
                    Target::Card(id)
                }
            };
            let victim = if case == 2 {
                Some(named(&mut g, owner.other(), "Bear Cub", Zone::Battlefield))
            } else {
                None
            };
            let spell = named(&mut g, owner, name, Zone::Library);
            g.exile_top_play(owner, 1, true);
            assert!(g.can_play_exiled(owner, spell));
            let exiled = g.object_ref(spell);
            g.state.players[owner.index()].mana = payment;
            assert!(
                g.legal_actions()
                    .contains(&Action::AnnounceExile { card: spell })
            );
            assert!(!g.legal_actions().iter().any(|a| matches!(a,
                Action::Cast { card, .. } | Action::BeginCast { card, .. } if *card == spell)));
            reject(
                &mut g,
                Action::Cast {
                    card: spell,
                    target: Some(target),
                    payment,
                },
            );
            reject(
                &mut g,
                Action::BeginCast {
                    card: spell,
                    payment,
                },
            );
            reject(&mut g, Action::AnnounceCast { card: spell });
            replay(&mut g, Action::AnnounceExile { card: spell });
            assert!(!g.can_play_exiled(owner, spell));
            assert_ne!(g.object_ref(spell), exiled);
            replay(&mut g, Action::ChooseTarget { target });
            if let Some(victim) = victim {
                replay(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(victim),
                    },
                );
            }
            if g.legal_actions().contains(&Action::FinishTargets) {
                replay(&mut g, Action::FinishTargets);
            }
            reject(&mut g, Action::PayCosts { payment: [0; 6] });
            replay(&mut g, Action::PayCosts { payment });
            assert_eq!(g.state.players[owner.index()].mana, [0; 6]);
            assert_eq!(g.state.players[owner.index()].spells_cast_this_turn, 1);
            let item = g.state.stack.iter().find(|s| s.card == spell).unwrap();
            assert_eq!(item.cast.unwrap().origin, Zone::Exile);
            assert_eq!(item.cast.unwrap().method, CastMethod::Normal);
            assert!(!g.state.objects[spell as usize].flashback);
            resolve(&mut g);
            assert_eq!(
                g.state.objects[spell as usize].zone,
                if case == 3 {
                    Zone::Battlefield
                } else {
                    Zone::Graveyard
                }
            );
            if let Some(victim) = victim {
                assert_eq!(g.state.objects[victim as usize].zone, Zone::Graveyard);
            }
            if case == 3 {
                let Target::Card(target) = target else {
                    unreachable!()
                };
                assert_eq!(
                    g.state.objects[spell as usize].attached_to,
                    Some(g.object_ref(target))
                );
                assert_eq!(
                    g.stats(target),
                    Stats {
                        power: -1,
                        toughness: 1
                    }
                );
            }
        }
    }
}

#[test]
fn discounted_exile_payment_capacity_failure_restores_locked_announcement_and_mana() {
    for owner in [Player::P0, Player::P1] {
        let mut g = scenario();
        g.state.active = owner;
        g.state.priority = owner;
        let target = named(&mut g, owner.other(), "Bear Cub", Zone::Battlefield);
        g.state.objects[target as usize].tapped = true;
        let spell = named(&mut g, owner, "Ajani's Response", Zone::Library);
        g.exile_top_play(owner, 1, true);
        let payment = [1, 0, 0, 0, 0, 1];
        g.state.players[owner.index()].mana = payment;
        replay(&mut g, Action::AnnounceExile { card: spell });
        replay(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            replay(&mut g, Action::FinishTargets);
        }
        let locked = g.state.announcing.as_ref().unwrap().total_cost;
        assert_eq!(
            locked,
            Some(ManaCost {
                colored: [1, 0, 0, 0, 0, 0],
                generic: 1
            })
        );
        g.state.players[owner.index()].spells_cast_this_turn = u64::MAX;
        let saved = g.snapshot();
        let revision = g.revision();
        assert!(matches!(
            g.apply(owner, Action::PayCosts { payment }),
            Err(Error::CapacityExceeded(_))
        ));
        assert_eq!(g.snapshot(), saved);
        assert_eq!(g.revision(), revision);
        assert_eq!(g.state.players[owner.index()].mana, payment);
        assert_eq!(g.state.announcing.as_ref().unwrap().total_cost, locked);
        g.state.players[owner.index()].spells_cast_this_turn = 0;
        replay(&mut g, Action::PayCosts { payment });
        assert_eq!(g.state.players[owner.index()].mana, [0; 6]);
        resolve(&mut g);
        assert_eq!(g.state.objects[target as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[spell as usize].zone, Zone::Graveyard);
    }
}
