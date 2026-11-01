//! Focused printed outcomes; shared engine invariants live in core tests.
//! Public subtype choice, live creature count and ordinary Equipment combat.
use super::*;
const FIXTURES: &[&str] = &[
    "Orcrist, Goblin-cleaver",
    "Bear Cub",
    "Barkform Harvester",
    "Morcant's Eyes",
    "Goldvein Pick",
    "Doubling Season",
    "Iron Hills Blacksmith",
    "Forest",
    "@token/hob-wolf",
    "@token/b60f6cd3b44b22bd289fe4e2",
];
fn fixture(g: &mut Game, p: Player, name: &str, zone: Zone) -> CardId {
    assert!(FIXTURES.contains(&name), "unlisted {name}");
    put(g, p, name, zone)
}
fn quiet(g: &Game) -> bool {
    g.state.pending == Pending::Priority
        && g.state.stack.is_empty()
        && g.state.pending_triggers.is_empty()
        && g.state.ordering_triggers.is_empty()
        && g.state.resolving.is_none()
}
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        if let Some(r) = g.damage_assignment_range() {
            act(
                g,
                Action::AssignDamage {
                    target: r.target,
                    amount: r.minimum,
                },
            );
        } else {
            choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
        }
    }
    panic!("Orcrist budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, quiet);
}
fn type_choice(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::CreatureTypeChoice { .. })
    });
}
fn typ(name: &str) -> crate::cards::CreatureType {
    crate::cards::CreatureType::from_name(name).unwrap()
}
fn choose_type(g: &mut Game, name: &str) {
    act(
        g,
        Action::ChooseCreatureType {
            creature_type: typ(name),
        },
    );
}
fn treasures(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter_map(|(id, o)| {
            (o.zone == Zone::Battlefield
                && o.controller == p
                && o.is_token()
                && g.has_subtype(id as CardId, "Treasure"))
            .then_some(id as CardId)
        })
        .collect()
}
fn paid(g: &mut Game, p: Player) -> CardId {
    let id = fixture(g, p, "Orcrist, Goblin-cleaver", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    cast(g, id, None);
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert!(g.object_view(id).is_legendary());
    id
}
fn equip(g: &mut Game, p: Player, e: CardId, b: CardId) {
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
    act(
        g,
        Action::Activate {
            card: e,
            ability: 0,
            target: Some(Target::Card(b)),
            payment: [0, 0, 0, 0, 0, 3],
            counter_payment: Counters::default(),
        },
    );
    finish(g);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    assert_eq!(g.object_view(e).attached_to, Some(g.object_ref(b)));
}
fn combat(g: &mut Game, b: CardId, blocker: Option<CardId>) {
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    g.state.objects[b as usize].summoning_sick = false;
    act(g, Action::Attack { card: b });
    act(g, Action::FinishAttackers);
    until(g, |g| g.state.pending == Pending::Blockers);
    if let Some(blocker) = blocker {
        act(
            g,
            Action::Block {
                blocker,
                attacker: b,
            },
        );
    }
    act(g, Action::FinishBlockers);
}
fn root(p: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let bear = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
    let e = paid(&mut g, p);
    equip(&mut g, p, e, bear);
    combat(&mut g, bear, None);
    type_choice(&mut g);
    (g, e, bear)
}
#[test]
fn orcrist_paid_equip_legality_boost_trample_actual_combat_treasure_recipe_and_mana() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let b = fixture(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let e = paid(&mut g, p);
        g.state.players[p.index()].mana = [0; 6];
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 3];
        g.state.active = p.other();
        g.state.priority = p;
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: e,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        g.state.active = p;
        reject(
            &mut g,
            Action::Activate {
                card: e,
                ability: 0,
                target: Some(Target::Card(blocker)),
                payment: [0, 0, 0, 0, 0, 3],
                counter_payment: Counters::default(),
            },
        );
        equip(&mut g, p, e, b);
        assert_eq!(stats(&g, b), (4, 4));
        assert!(g.keywords(b).contains(Keywords::TRAMPLE));
        combat(&mut g, b, Some(blocker));
        type_choice(&mut g);
        assert_eq!(g.object_view(blocker).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.other().index()].life, 18);
        choose_type(&mut g, "Bear");
        finish(&mut g);
        let ts = treasures(&g, p);
        assert_eq!(ts.len(), 1);
        let t = ts[0];
        let view = g.object_view(t);
        assert!(!view.is_creature());
        assert_eq!(view.kinds, 16);
        assert!(!view.tapped);
        assert_eq!(view.colors, 0);
        g.state.players[p.index()].mana = [0; 6];
        choose(
            &mut g,
            |a| matches!(a,Action::ActivateMana{card,production:[1,0,0,0,0,0],..}if *card==t),
        );
        assert_eq!(g.state.players[p.index()].mana, [1, 0, 0, 0, 0, 0]);
        assert!(treasures(&g, p).is_empty());
    }
}
#[test]
fn orcrist_every_valid_type_absent_choice_changeling_tokens_and_noncreature_exclusion() {
    let p = Player::P0;
    let (mut g, _, _) = root(p);
    fixture(&mut g, p, "Barkform Harvester", Zone::Battlefield);
    fixture(&mut g, p, "@token/hob-wolf", Zone::Battlefield);
    fixture(&mut g, p, "Morcant's Eyes", Zone::Battlefield);
    fixture(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
    fixture(&mut g, p, "Bear Cub", Zone::Graveyard);
    let offered = g
        .legal_actions()
        .into_iter()
        .filter_map(|a| {
            if let Action::ChooseCreatureType { creature_type } = a {
                Some(creature_type.name())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(offered, crate::CREATURE_TYPES);
    for name in crate::CREATURE_TYPES {
        let mut child = g.clone();
        choose_type(&mut child, name);
        finish(&mut child);
        assert_eq!(
            treasures(&child, p).len(),
            1 + usize::from(*name == "Bear") + usize::from(*name == "Wolf"),
            "{name}"
        );
    }
    let (mut empty, _, _) = root(p);
    choose_type(&mut empty, "Aetherborn");
    finish(&mut empty);
    assert!(treasures(&empty, p).is_empty());
    for invalid in ["Equipment", "Treasure", "Land", "", "NotAType"] {
        assert!(crate::cards::CreatureType::from_name(invalid).is_none());
        let bad = serde_json::json!({"ChooseCreatureType":{"creature_type":invalid}});
        assert!(serde_json::from_value::<Action>(bad).is_err());
    }
}
