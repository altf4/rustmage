use super::assurance::big_helpers::spawn;
use super::assurance::big_helpers::{cast, checked, settle, setup};
use super::*;
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
    let snap = g.snapshot();
    assert!(g.apply(g.actor().unwrap(), a).is_err());
    assert_eq!(g.snapshot(), snap);
}
#[test]
fn big_lotus_ring_grants_paid_sacrifice_mana_and_funding_tracks_departure() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ring = paid(&mut g, p, "Lotus Ring");
        let bear = paid(&mut g, p, "Bear Cub");
        equip(&mut g, ring, bear);
        assert_eq!(
            g.stats(bear),
            Stats {
                power: 5,
                toughness: 5
            }
        );
        assert!(g.keywords(bear).contains(Keywords::VIGILANCE));
        assert!(g.keywords(ring).contains(Keywords::INDESTRUCTIBLE));
        let production = [0, 0, 0, 0, 3, 0];
        let action = Action::TapGrantedMana {
            card: bear,
            grantor: g.object_ref(ring),
            production,
        };
        reject(&mut g, action.clone());
        g.state.objects[bear as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        assert!(g.can_fund(FundingGoal {
            actor: p,
            required: production,
            generic: 0,
            reserve: None,
            source: None
        }));
        assert!(!g.can_fund(FundingGoal {
            actor: p,
            required: [0, 0, 0, 0, 4, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        checked(&mut g, action);
        assert_eq!(g.state.players[p.index()].mana, production);
        assert_eq!(g.state.objects[bear as usize].zone, Zone::Graveyard);
        assert_eq!(g.object_view(ring).attached_to, None);
    }
}
#[test]
fn big_lost_jitte_combat_batch_charge_cost_all_three_choices_and_atomicity() {
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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
    for p in [Player::P0, Player::P1] {
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
fn big_esoteric_sacrifice_uses_saved_recipe_and_source_independent_next_end_copy() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let id = paid(&mut g, p, "Esoteric Duplicator");
        named(&mut g, p, "Forest", Zone::Library);
        activate(&mut g, id, 0, None);
        assert_eq!(g.state.objects[id as usize].zone, Zone::Graveyard);
        assert!(g.state.delayed_artifact_copies.is_some());
        g.begin_step(Step::End);
        settle(&mut g);
        let copies: Vec<_> = g
            .state
            .objects
            .iter()
            .filter(|o| {
                o.zone == Zone::Battlefield
                    && o.is_copy_token()
                    && o.card == Card::from_name("Esoteric Duplicator").unwrap()
            })
            .collect();
        assert_eq!(copies.len(), 1);
        assert!(g.state.delayed_artifact_copies.is_none());
    }
}

#[test]
fn big_lotus_vista_funding_recomputes_monocolor_after_virtual_sacrifice() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let ring = paid(&mut g, p, "Lotus Ring");
        let bear = paid(&mut g, p, "Bear Cub");
        equip(&mut g, ring, bear);
        let vista = named(&mut g, p, "Tarnation Vista", Zone::Hand);
        checked(&mut g, Action::PlayLand { card: vista });
        checked(
            &mut g,
            Action::ChooseEntryValue {
                value: EntryChoiceValue::Color(crate::cards::ChoiceColor::Blue),
            },
        );
        settle(&mut g);
        g.state.objects[vista as usize].tapped = false;
        g.state.objects[bear as usize].summoning_sick = false;
        g.state.players[p.index()].mana = [0; 6];
        let saved = g.snapshot();
        assert!(!g.can_fund(FundingGoal {
            actor: p,
            required: [0, 0, 0, 2, 1, 0],
            generic: 0,
            reserve: None,
            source: None
        }));
        assert_eq!(g.snapshot(), saved);
        let grantor = g.object_ref(ring);
        checked(
            &mut g,
            Action::TapGrantedMana {
                card: bear,
                grantor,
                production: [0, 0, 0, 3, 0, 0],
            },
        );
        assert!(!g.legal_actions().iter().any(|a|matches!(a,Action::ActivateMana {card,ability:1,production,..} if *card==vista&&production[4]>0)));
    }
}
#[test]
fn big_esoteric_tracks_simultaneous_artifact_sacrifices_replaced_by_exile() {
    for p in [Player::P0, Player::P1] {
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
