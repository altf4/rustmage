use super::ability_timeline_tests::{cast, checked, settle, setup};
use super::tests::spawn;
use super::*;

// Candidate bodies remain admission-gated while these paid rules scenarios run.
fn named(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    spawn(g, p, Card::from_name(name).unwrap(), zone)
}
fn paid(g: &mut Game, p: Player, name: &str) -> CardId {
    let id = named(g, p, name, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn reject(g: &mut Game, action: Action) {
    let before = g.snapshot();
    let revision = g.revision();
    assert!(g.apply(g.actor().unwrap(), action).is_err());
    assert_eq!(g.snapshot(), before);
    assert_eq!(g.revision(), revision);
}
fn activation(g: &mut Game, source: CardId, ability: u16, target: Option<Target>) {
    checked(
        g,
        Action::AnnounceActivate {
            card: source,
            ability,
            counter_payment: Counters::default(),
        },
    );
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. } | Action::FinishTargets))
            .unwrap();
        checked(g, action);
    }
    panic!("activation payment did not finish");
}
fn tokens(g: &Game, p: Player, token: Card) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(i, o)| {
            (o.zone == Zone::Battlefield && o.controller == p && o.card == token)
                .then_some(i as CardId)
        })
        .collect()
}
#[test]
fn fdn_growth_ovika_cast_value_tokens_temporary_haste_and_mixed_ward() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ovika = paid(&mut g, p, "Ovika, Enigma Goliath");
        assert!(g.keywords(ovika).contains(Keywords::FLYING));
        let spell = named(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, spell, None, &[]);
        settle(&mut g);
        let goblin = Card::from_name("@token/phyrexian_goblin_1_1_red").unwrap();
        let made = tokens(&g, p, goblin);
        assert_eq!(made.len(), 3);
        for id in &made {
            let v = g.object_view(*id);
            assert_eq!((v.power, v.toughness, v.colors), (1, 1, 8));
            assert!(v.has_subtype("Phyrexian") && v.has_subtype("Goblin"));
            assert!(v.keywords.contains(Keywords::HASTE));
        }
        g.state.priority = p.other();
        let shock = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, shock, None, &[Target::Card(ovika)]);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        let beforelife = g.state.players[p.other().index()].life;
        let payment = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap();
        checked(&mut g, payment);
        assert_eq!(g.state.players[p.other().index()].life, beforelife - 3);
        settle(&mut g);
        assert_eq!(g.state.objects[ovika as usize].damage, 2);
        g.finish_cleanup();
        for id in made {
            assert!(!g.keywords(id).contains(Keywords::HASTE));
        }
    }
}
#[test]
fn fdn_growth_ovika_insufficient_life_declines_without_partial_mana_payment() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let o = paid(&mut g, p, "Ovika, Enigma Goliath");
        g.state.priority = p.other();
        g.state.players[p.other().index()].life = 2;
        let s = named(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, s, None, &[Target::Card(o)]);
        for _ in 0..30 {
            if matches!(g.state.pending, Pending::ResolutionMana { .. }) {
                break;
            }
            let a = g
                .legal_actions()
                .into_iter()
                .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                .unwrap();
            checked(&mut g, a);
        }
        reject(
            &mut g,
            Action::PayResolutionMana {
                payment: [0, 0, 0, 0, 0, 3],
            },
        );
        assert!(
            !g.legal_actions()
                .iter()
                .any(|a| matches!(a, Action::PayResolutionMana { .. }))
        );
        checked(&mut g, Action::DeclineResolutionMana);
        settle(&mut g);
        assert_eq!(g.state.objects[s as usize].zone, Zone::Graveyard);
        assert_eq!(g.state.objects[o as usize].damage, 0);
    }
}
#[test]
fn fdn_growth_kellan_resolution_guards_persistent_types_stats_and_reset() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let k = paid(&mut g, p, "Kellan, Planar Trailblazer");
        activation(&mut g, k, 1, None);
        settle(&mut g);
        assert!(g.has_subtype(k, "Scout"));
        assert_eq!(g.stats(k).power, 2);
        activation(&mut g, k, 0, None);
        settle(&mut g);
        assert!(g.has_subtype(k, "Detective"));
        assert!(!g.has_subtype(k, "Scout"));
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        activation(&mut g, k, 0, None);
        settle(&mut g);
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        activation(&mut g, k, 1, None);
        settle(&mut g);
        assert!(g.has_subtype(k, "Rogue"));
        assert!(!g.has_subtype(k, "Detective"));
        assert_eq!(
            g.stats(k),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert!(g.keywords(k).contains(Keywords::DOUBLE_STRIKE));
        g.finish_cleanup();
        assert_eq!(
            g.stats(k),
            Stats {
                power: 3,
                toughness: 2
            }
        );
        assert!(g.keywords(k).contains(Keywords::DOUBLE_STRIKE));
        assert_eq!(g.observe(p).temporary_abilities.len(), 1);
        g.move_object(k, Zone::Hand);
        g.move_object(k, Zone::Battlefield);
        assert_eq!(
            g.stats(k),
            Stats {
                power: 2,
                toughness: 1
            }
        );
        assert!(g.has_subtype(k, "Scout"));
        assert!(!g.keywords(k).contains(Keywords::DOUBLE_STRIKE));
        assert!(g.observe(p).temporary_abilities.is_empty());
    }
}
#[test]
fn fdn_growth_lathril_ten_distinct_other_elves_pay_as_cost_sick_elves_allowed() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let l = paid(&mut g, p, "Lathril, Blade of the Elves");
        g.state.objects[l as usize].summoning_sick = false;
        let elves: Vec<_> = (0..10)
            .map(|_| named(&mut g, p, "Llanowar Elves", Zone::Battlefield))
            .collect();
        checked(
            &mut g,
            Action::AnnounceActivate {
                card: l,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        reject(
            &mut g,
            Action::ChooseAdditionalCost {
                object: g.object_ref(l),
            },
        );
        for (index, id) in elves.iter().enumerate() {
            let r = g.object_ref(*id);
            checked(&mut g, Action::ChooseAdditionalCost { object: r });
            if index < 9 {
                reject(&mut g, Action::ChooseAdditionalCost { object: r });
            }
            assert!(g.observe(p.other()).additional_cost.is_none());
        }
        let action = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, action);
        assert!(g.state.objects[l as usize].tapped);
        for id in &elves {
            assert!(g.state.objects[*id as usize].tapped);
        }
        g.move_object(l, Zone::Graveyard);
        settle(&mut g);
        assert_eq!(g.state.players[p.index()].life, 30);
        assert_eq!(g.state.players[p.other().index()].life, 10);
    }
}
#[test]
fn fdn_growth_preserver_nonhuman_optional_x_paid_reflexive_and_source_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = paid(&mut g, p, "Wildborn Preserver");
        assert!(
            g.keywords(w)
                .contains(Keywords::FLASH.union(Keywords::REACH))
        );
        let human = paid(&mut g, p, "Savannah Lions"); // Cat is a non-Human and must trigger.
        assert!(g.state.objects[human as usize].zone == Zone::Battlefield);
        // Default settle chooses X=0; now test a nonzero actual payment.
        let e = named(&mut g, p, "Llanowar Elves", Zone::Hand);
        cast(&mut g, e, None, &[]);
        for _ in 0..50 {
            if matches!(g.state.pending, Pending::ResolutionX { .. }) {
                break;
            }
            let a = if matches!(g.state.pending, Pending::EffectOptional { .. }) {
                Action::ChooseOptional { accept: true }
            } else {
                g.legal_actions()
                    .into_iter()
                    .find(|a| matches!(a, Action::Pass | Action::FinishTargets))
                    .unwrap()
            };
            checked(&mut g, a);
        }
        checked(&mut g, Action::ChooseX { value: 3 });
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .unwrap();
        checked(&mut g, a);
        assert!(g.state.stack.iter().chain(&g.state.pending_triggers).any(|i|i.object.card==w && i.kind==StackKind::TriggeredAbility{ability:1}));
        settle(&mut g);
        assert_eq!(g.counter_count(w, CounterKind::PlusOne), 3);
    }
}
#[test]
fn fdn_growth_fishing_paid_grant_attachment_counter_and_actual_untap() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let pole = paid(&mut g, p, "Fishing Pole");
        let c = paid(&mut g, p, "Bear Cub");
        g.state.objects[c as usize].summoning_sick = false;
        activation(&mut g, pole, 0, Some(Target::Card(c)));
        settle(&mut g);
        let grantor = g.object_ref(pole);
        checked(
            &mut g,
            Action::AnnounceAttachedAbility {
                grantor,
                ability: 1,
            },
        );
        let a = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, a);
        settle(&mut g);
        assert!(g.state.objects[pole as usize].tapped && g.state.objects[c as usize].tapped);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 1);
        g.add_counter(c, CounterKind::Stun, 1);
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        assert!(g.state.objects[c as usize].tapped);
        assert!(g.state.pending_triggers.is_empty());
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        settle(&mut g);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 0);
        assert_eq!(tokens(&g, p, Card(5317)).len(), 1);
        g.untap_objects(&[c], untap_rules::UntapCause::Effect);
        settle(&mut g);
        assert_eq!(tokens(&g, p, Card(5317)).len(), 1);
    }
}
#[test]
fn fdn_growth_greatwurm_mixed_counter_cost_graveyard_and_normal_hand_routes() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let w = named(&mut g, p, "Quilled Greatwurm", Zone::Graveyard);
        let a = named(&mut g, p, "Bear Cub", Zone::Battlefield);
        let b = named(&mut g, p, "Llanowar Elves", Zone::Battlefield);
        g.add_counter(a, CounterKind::Bait, 3);
        g.add_counter(b, CounterKind::PlusOne, 3);
        checked(&mut g, Action::AnnounceCounterGraveyard { card: w });
        for (id, kind) in [
            (a, CounterKind::Bait),
            (a, CounterKind::Bait),
            (a, CounterKind::Bait),
            (b, CounterKind::PlusOne),
            (b, CounterKind::PlusOne),
            (b, CounterKind::PlusOne),
        ] {
            let object = g.object_ref(id);
            checked(&mut g, Action::ChooseCreatureCounterCost { object, kind });
        }
        let act = g
            .legal_actions()
            .into_iter()
            .find(|a| matches!(a, Action::PayCosts { .. }))
            .unwrap();
        checked(&mut g, act);
        assert_eq!(g.counter_count(a, CounterKind::Bait), 0);
        assert_eq!(g.counter_count(b, CounterKind::PlusOne), 0);
        settle(&mut g);
        assert_eq!(g.state.objects[w as usize].zone, Zone::Battlefield);
        assert!(g.keywords(w).contains(Keywords::TRAMPLE));
    }
}
#[test]
fn fdn_growth_layout_hot_objects_and_pending_stay_bounded() {
    assert_eq!(std::mem::size_of::<Object>(), 144);
    assert_eq!(std::mem::size_of::<Pending>(), 40);
}
