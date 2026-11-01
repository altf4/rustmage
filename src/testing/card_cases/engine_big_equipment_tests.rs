//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;
use crate::engine::testing::card_cases::big_helpers::spawn;
use crate::engine::testing::card_cases::big_helpers::{cast, checked, settle, setup};
fn named(g: &mut Game, p: Player, n: &str, z: Zone) -> CardId {
    spawn(g, p, Card::from_name(n).unwrap(), z)
}
fn paid(g: &mut Game, p: Player, n: &str) -> CardId {
    let id = named(g, p, n, Zone::Hand);
    cast(g, id, None, &[]);
    settle(g);
    id
}
fn equip(g: &mut Game, id: CardId, target: CardId) {
    let a=g.legal_actions().into_iter().find(|a|matches!(a,Action::Activate {card,ability:0,target:Some(Target::Card(t)),..} if *card==id && *t==target)).expect("paid equip");
    checked(g, a);
    settle(g);
}
fn activate(g: &mut Game, id: CardId, index: u16, target: Option<Target>) {
    let jitte = g.state.objects[id as usize].card.name() == "Lost Jitte";
    let counters = if jitte {
        Counters::of(CounterKind::Charge, 1)
    } else {
        Counters::default()
    };
    checked(
        g,
        Action::AnnounceActivate {
            card: id,
            ability: if jitte { 1 } else { index },
            counter_payment: counters,
        },
    );
    if jitte {
        checked(g, Action::ChooseMode { mode: index - 1 });
    }
    if let Some(target) = target {
        checked(g, Action::ChooseTarget { target });
    }
    settle_paid(g);
}
fn settle_paid(g: &mut Game) {
    g.prepare_triggers();
    for _ in 0..100 {
        if g.state.stack.is_empty()
            && g.state.pending == Pending::Priority
            && g.state.announcing.is_none()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
        {
            return;
        }
        let acts = g.legal_actions();
        let a = acts
            .iter()
            .find(|a| matches!(a, Action::PayResolutionMana { .. }))
            .or_else(|| {
                acts.iter().find(|a| {
                    matches!(
                        a,
                        Action::Pass
                            | Action::PutTrigger { .. }
                            | Action::PayCosts { .. }
                            | Action::FinishTargets
                            | Action::FinishStackTargets
                    )
                })
            })
            .or_else(|| acts.iter().find(|a| !matches!(a, Action::Concede)))
            .unwrap()
            .clone();
        checked(g, a);
    }
    panic!("unsettled");
}
fn damage(g: &mut Game, source: CardId, target: Target, combat: bool) {
    let source = g.damage_source(source);
    g.apply_damage_batch(&[DamageEvent {
        source,
        target,
        amount: 1,
        combat,
        preventable: true,
    }]);
    g.prepare_triggers();
    settle(g);
}
fn reject(g: &mut Game, a: Action) {
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
}

#[test]
fn big_lost_jitte_combat_batch_charge_cost_all_three_choices_and_atomicity() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let j = paid(&mut g, p, "Lost Jitte");
        let bear = paid(&mut g, p, "Bear Cub");
        equip(&mut g, j, bear);
        let land = named(&mut g, p, "Forest", Zone::Battlefield);
        g.state.objects[land as usize].tapped = true;
        damage(&mut g, bear, Target::Player(p.other()), false);
        assert_eq!(g.counter_count(j, CounterKind::Charge), 0);
        damage(&mut g, bear, Target::Player(p.other()), true);
        assert_eq!(g.counter_count(j, CounterKind::Charge), 1);
        activate(&mut g, j, 1, Some(Target::Card(land)));
        assert!(!g.object_view(land).tapped);
        assert_eq!(g.counter_count(j, CounterKind::Charge), 0);
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: j,
                ability: 1,
                counter_payment: Counters::of(CounterKind::Charge, 1),
            },
        );
        damage(&mut g, bear, Target::Player(p.other()), true);
        activate(&mut g, j, 2, Some(Target::Card(bear)));
        assert!(g.keywords(bear).contains(Keywords::CANT_BLOCK));
        damage(&mut g, bear, Target::Player(p.other()), true);
        activate(&mut g, j, 3, None);
        assert_eq!(g.counter_count(bear, CounterKind::PlusOne), 1);
    }
}
#[test]
fn big_sword_protection_prevents_targeting_and_damage_and_delays_one_spell_copy() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let sword = paid(&mut g, p, "Sword of Wealth and Power");
        let bear = paid(&mut g, p, "Bear Cub");
        equip(&mut g, sword, bear);
        assert_eq!(g.stats(bear).power, 4);
        let shock = named(&mut g, p, "Shock", Zone::Hand);
        reject(
            &mut g,
            Action::Cast {
                card: shock,
                target: Some(Target::Card(bear)),
                payment: [0, 0, 0, 1, 0, 0],
            },
        );
        let source = g.damage_source(shock);
        g.apply_damage_batch(&[DamageEvent {
            source,
            target: Target::Card(bear),
            amount: 3,
            combat: false,
            preventable: true,
        }]);
        assert_eq!(g.object_view(bear).damage, 0);
        damage(&mut g, bear, Target::Player(p.other()), true);
        assert!(g.state.objects.iter().any(|o| {
            o.zone == Zone::Battlefield
                && o.controller == p
                && o.card
                    == Card::from_name("@token/treasure")
                        .unwrap_or(Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap())
        }));
        cast(&mut g, shock, None, &[Target::Player(p.other())]);
        settle_paid(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 15);
    }
}
#[test]
fn big_generous_plunderer_optional_reflexive_tapped_treasure_and_attack_count() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let id = paid(&mut g, p, "Generous Plunderer");
        g.begin_step(Step::Upkeep);
        settle(&mut g);
        let own: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p
                    && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()
            })
            .collect();
        let theirs: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.controller == p.other()
                    && o.card == Card::from_name("@token/b60f6cd3b44b22bd289fe4e2").unwrap()
            })
            .collect();
        assert_eq!(own.len(), 1);
        assert_eq!(theirs.len(), 1);
        assert!(!own[0].tapped);
        assert!(theirs[0].tapped);
        let source = g.object_view(id);
        assert!(g.queue_trigger(&source, 2));
        settle(&mut g);
        assert_eq!(g.state.players[p.other().index()].life, 19);
    }
}

#[test]
fn big_esoteric_tracks_simultaneous_artifact_sacrifices_replaced_by_exile() {
    {
        let p = Player::P0;
        let mut g = setup(p);
        let dup = paid(&mut g, p, "Esoteric Duplicator");
        let ring = paid(&mut g, p, "Lotus Ring");
        paid(&mut g, p, "Rest in Peace");
        g.sacrifice_batch(&[(p, dup), (p, ring)]);
        assert_eq!(g.state.objects[dup as usize].zone, Zone::Exile);
        assert_eq!(g.state.objects[ring as usize].zone, Zone::Exile);
        settle_paid(&mut g);
        assert_eq!(g.state.delayed_artifact_copies.as_ref().unwrap().len(), 2);
        g.begin_step(Step::End);
        settle_paid(&mut g);
        for name in ["Esoteric Duplicator", "Lotus Ring"] {
            assert!(g.state.objects.iter().any(|o| o.zone == Zone::Battlefield
                && o.is_copy_token()
                && o.card.name() == name));
        }
    }
}
