//! Actual paid decisions and exact outcomes for two existing-mechanic HOB cards.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("stir/stone choice budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn bind(g: &mut Game, target: Target) {
    act(g, Action::ChooseTarget { target });
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(g, Action::FinishTargets);
    }
}
fn extra(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    act(g, Action::ChooseAdditionalCost { object });
}
fn reject_extra(g: &mut Game, id: CardId) {
    let object = g.object_ref(id);
    reject(g, Action::ChooseAdditionalCost { object });
}
fn stir(g: &mut Game, spell: CardId, target: CardId, route: u16, free: bool) {
    act(
        g,
        if free {
            Action::AnnounceFree {
                card: spell,
                face: CastFace::Normal,
                flash_surcharge: false,
            }
        } else {
            Action::AnnounceCast { card: spell }
        },
    );
    act(g, Action::ChooseSpellCost { cost: route });
    bind(g, Target::Card(target));
}
fn stones(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.card.identity() == "@token/hob-stone-boulder")
                .then_some(id as CardId)
        })
        .collect()
}
fn giant(g: &mut Game, p: Player) -> CardId {
    let source = put(g, p, "Stone-Giant of High Pass", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 2, 0, 5];
    cast(g, source, None);
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(stats(g, source), (7, 7));
    assert!(g.has_subtype(source, "Giant"));
    source
}
fn activate(g: &mut Game, source: CardId, target: Target) {
    act(
        g,
        Action::AnnounceActivate {
            card: source,
            ability: 0,
            counter_payment: Counters::default(),
        },
    );
    bind(g, target);
}
#[test]
fn hob_stir_stone_stir_paid_and_free_sacrifice_creature_artifact_token_or_four_mana() {
    for p in [Player::P0, Player::P1] {
        for free in [false, true] {
            for route in 0..4 {
                let mut g = scenario(p);
                if free {
                    put(&mut g, p, "Omniscience", Zone::Battlefield);
                }
                let fodder = put(
                    &mut g,
                    p,
                    match route {
                        0 => "Bear Cub",
                        1 => "Giant's Boulder",
                        _ => "@token/hob-stone-boulder",
                    },
                    Zone::Battlefield,
                );
                let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
                let spell = put(&mut g, p, "Stir Up Trouble", Zone::Hand);
                g.state.players[p.index()].mana =
                    [0, 0, u32::from(!free), 0, 0, if route == 3 { 4 } else { 0 }];
                stir(&mut g, spell, target, u16::from(route == 3), free);
                if route != 3 {
                    extra(&mut g, fodder);
                }
                pay(&mut g);
                finish(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
                assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(
                    g.object_view(fodder).zone,
                    match route {
                        2 => Zone::Outside,
                        3 => Zone::Battlefield,
                        _ => Zone::Graveyard,
                    }
                );
            }
        }
    }
}
#[test]
fn hob_stir_stone_stir_cost_legality_stale_targets_and_indestructible() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..3 {
            let mut g = scenario(p);
            let forest = put(&mut g, p, "Forest", Zone::Battlefield);
            let enchantment = put(&mut g, p, "Doubling Season", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
            let target = put(
                &mut g,
                p.other(),
                if mode == 2 {
                    "Darksteel Colossus"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            let fodder = put(&mut g, p, "Giant's Boulder", Zone::Battlefield);
            let spell = put(&mut g, p, "Stir Up Trouble", Zone::Hand);
            g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 0];
            act(&mut g, Action::AnnounceCast { card: spell });
            reject(&mut g, Action::ChooseSpellCost { cost: 1 });
            act(&mut g, Action::ChooseSpellCost { cost: 0 });
            reject(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Card(forest),
                },
            );
            bind(&mut g, Target::Card(target));
            for illegal in [forest, enchantment, enemy] {
                reject_extra(&mut g, illegal);
            }
            extra(&mut g, fodder);
            pay(&mut g);
            assert_eq!(
                g.object_view(fodder).zone,
                Zone::Graveyard,
                "cost paid before responses"
            );
            if mode == 1 {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            finish(&mut g);
            assert_eq!(
                g.object_view(target).zone,
                if mode == 0 {
                    Zone::Graveyard
                } else {
                    Zone::Battlefield
                }
            );
        }
        let mut g = scenario(p);
        let spell = put(&mut g, p, "Stir Up Trouble", Zone::Hand);
        put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 3];
        reject(&mut g, Action::AnnounceCast { card: spell });
    }
}
#[test]
fn hob_stir_stone_stir_owned_cost_branches_keep_selected_routes_independent() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let fodder = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Stir Up Trouble", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 1, 0, 0, 4];
        act(&mut g, Action::AnnounceCast { card: spell });
        super::contracts::branch_contract(g.clone());
        let saved = g.snapshot();
        let mut raw = g.clone();
        let mut indexed = g.clone();
        drop(g);
        let action = Action::ChooseSpellCost { cost: 0 };
        let index = indexed
            .legal_actions()
            .iter()
            .position(|a| *a == action)
            .unwrap();
        act(&mut raw, action);
        indexed.apply_index(p, indexed.revision(), index).unwrap();
        for child in [&mut raw, &mut indexed] {
            bind(child, Target::Card(target));
            extra(child, fodder);
            pay(child);
            finish(child);
        }
        assert_eq!(raw.snapshot(), indexed.snapshot());
        assert_eq!(raw.object_view(fodder).zone, Zone::Graveyard);
        indexed.restore(&saved).unwrap();
        act(&mut indexed, Action::ChooseSpellCost { cost: 1 });
        bind(&mut indexed, Target::Card(target));
        pay(&mut indexed);
        finish(&mut indexed);
        assert_eq!(indexed.object_view(fodder).zone, Zone::Battlefield);
        assert_eq!(indexed.state.players[p.index()].mana, [0; 6]);
        assert_eq!(raw.state.players[p.index()].mana, [0, 0, 0, 0, 0, 4]);
    }
}
#[test]
fn hob_stir_stone_giant_paid_entry_attack_exact_recipe_and_defender() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let source = giant(&mut g, p);
        assert_eq!(stones(&g, p).len(), 1);
        assert!(stones(&g, p.other()).is_empty());
        let token = stones(&g, p)[0];
        let v = g.object_view(token);
        assert_eq!((v.power, v.toughness, v.kinds, v.colors), (3, 1, 17, 0));
        assert!(g.has_subtype(token, "Wall"));
        assert!(v.keywords.contains(Keywords::DEFENDER));
        assert!(!v.tapped);
        assert_eq!(v.card.info().deck_limit, None);
        assert_eq!(g.object_name(token), "Stone Boulder");
        g.state.objects[source as usize].summoning_sick = false;
        g.state.objects[token as usize].summoning_sick = false;
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        reject(&mut g, Action::Attack { card: token });
        act(&mut g, Action::Attack { card: source });
        act(&mut g, Action::FinishAttackers);
        until(&mut g, |g| stones(g, p).len() == 2);
        assert!(stones(&g, p).iter().all(|&id| !g.object_view(id).attacking));
    }
}
#[test]
fn hob_stir_stone_giant_activation_cost_damage_source_departure_and_stale_target() {
    for p in [Player::P0, Player::P1] {
        for mode in 0..4 {
            let mut g = scenario(p);
            let source = giant(&mut g, p);
            let fodder = stones(&g, p)[0];
            let own = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let enemy = put(&mut g, p.other(), "Giant's Boulder", Zone::Battlefield);
            let forest = put(&mut g, p, "Forest", Zone::Battlefield);
            let target = put(
                &mut g,
                p.other(),
                if mode == 3 {
                    "Ajani, Caller of the Pride"
                } else {
                    "Bear Cub"
                },
                Zone::Battlefield,
            );
            if mode == 3 {
                g.set_counters(target, Counters::of(CounterKind::Loyalty, 4));
            }
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
            activate(
                &mut g,
                source,
                if mode == 0 {
                    Target::Player(p.other())
                } else {
                    Target::Card(target)
                },
            );
            for illegal in [own, enemy, forest] {
                reject_extra(&mut g, illegal);
            }
            extra(&mut g, fodder);
            pay(&mut g);
            assert_eq!(g.object_view(fodder).zone, Zone::Outside);
            assert_eq!(g.state.players[p.index()].mana, [0; 6]);
            assert!(!g.object_view(source).tapped);
            if mode == 1 {
                g.move_object(source, Zone::Exile);
            }
            if mode == 2 {
                g.move_object(target, Zone::Hand);
                g.move_object(target, Zone::Battlefield);
            }
            finish(&mut g);
            if mode == 0 {
                assert_eq!(g.state.players[p.other().index()].life, 16);
            } else if mode == 2 {
                assert_eq!(g.object_view(target).zone, Zone::Battlefield);
                assert_eq!(g.object_view(target).damage, 0);
            } else {
                assert_eq!(g.object_view(target).zone, Zone::Graveyard);
            }
        }
    }
}
#[test]
fn hob_stir_stone_giant_token_replacement_and_ability_reusability() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        put(&mut g, p, "Doubling Season", Zone::Battlefield);
        let source = giant(&mut g, p);
        assert_eq!(stones(&g, p).len(), 2);
        for life in [16, 12] {
            let fodder = stones(&g, p)[0];
            g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 2];
            activate(&mut g, source, Target::Player(p.other()));
            extra(&mut g, fodder);
            pay(&mut g);
            finish(&mut g);
            assert_eq!(g.state.players[p.other().index()].life, life);
        }
        assert!(stones(&g, p).is_empty());
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: source,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
    }
}
