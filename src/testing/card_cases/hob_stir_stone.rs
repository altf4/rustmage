//! Focused printed outcomes; shared engine invariants live in core tests.
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

#[test]
fn hob_stir_stone_stir_paid_and_free_sacrifice_creature_artifact_token_or_four_mana() {
    {
        let p = Player::P0;
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
fn hob_stir_stone_giant_paid_entry_attack_exact_recipe_and_defender() {
    {
        let p = Player::P0;
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
