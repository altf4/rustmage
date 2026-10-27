use super::*;
fn armies(g: &Game, p: Player) -> Vec<CardId> {
    g.state
        .objects
        .iter()
        .enumerate()
        .filter(|(id, o)| {
            o.zone == Zone::Battlefield && o.controller == p && g.has_subtype(*id as CardId, "Army")
        })
        .map(|(id, _)| id as CardId)
        .collect()
}
#[test]
fn hob_amass_spell_and_entry_outcomes() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
        cast(&mut g, rage, None);
        settle(&mut g);
        let army = armies(&g, p)[0];
        assert_eq!(stats(&g, army), (2, 2));
        assert!(g.has_subtype(army, "Goblin"));
        assert_eq!(g.state.players[p.index()].life, 19);
        assert_eq!(g.state.players[p.index()].cards_drawn_this_turn, 1);
        let flunkies = put(&mut g, p, "Goblin-town Flunkies", Zone::Hand);
        cast(&mut g, flunkies, None);
        settle(&mut g);
        assert_eq!(armies(&g, p), vec![army]);
        assert_eq!(stats(&g, army), (3, 3));
        assert!(g.keywords(flunkies).contains(Keywords::HASTE));
        let gathering = put(&mut g, p, "Gathering of Darkness", Zone::Hand);
        let dead = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
        cast(&mut g, gathering, Some(Target::Card(dead)));
        settle(&mut g);
        assert_eq!(g.state.objects[dead as usize].zone, Zone::Hand);
        assert_eq!(stats(&g, army), (6, 6));
    }
}
#[test]
fn hob_amass_trigger_and_death_outcomes() {
    let p = Player::P0;
    let mut g = scenario(p);
    let noise = put(&mut g, p, "Bothersome Noisemaker", Zone::Battlefield);
    let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
    cast(&mut g, rage, None);
    settle(&mut g);
    let army = armies(&g, p)[0];
    assert_eq!(stats(&g, army), (3, 3));
    assert_eq!(stats(&g, noise), (2, 2));
    let pair = put(&mut g, p, "Fearsome Goblin Pair", Zone::Hand);
    cast(&mut g, pair, None);
    settle(&mut g);
    assert_eq!(stats(&g, army), (3, 3));
    let murder = put(&mut g, p, "Murder", Zone::Hand);
    cast(&mut g, murder, Some(Target::Card(pair)));
    settle(&mut g);
    assert_eq!(g.state.objects[pair as usize].zone, Zone::Graveyard);
    assert_eq!(stats(&g, army), (8, 8));
}
#[test]
fn hob_amass_attack_is_once_per_declaration() {
    let p = Player::P1;
    let mut g = scenario(p);
    let raider = put(&mut g, p, "Misty Mountains Raider", Zone::Battlefield);
    let a = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    let b = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    g.state.step = Step::Attackers;
    g.state.pending = Pending::Attackers;
    act(&mut g, Action::Attack { card: a });
    act(&mut g, Action::Attack { card: b });
    act(&mut g, Action::FinishAttackers);
    settle(&mut g);
    assert_eq!(stats(&g, armies(&g, p)[0]), (2, 2));
    assert_eq!(stats(&g, raider), (4, 4));
}
#[test]
fn hob_amass_multiple_armies_choose_without_targeting_and_restore() {
    let p = Player::P0;
    let mut g = scenario(p);
    let a = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
    let b = put(&mut g, p, "@token/goblin_army", Zone::Battlefield);
    for id in [a, b] {
        g.state.objects[id as usize].counters.plus_one = 1;
    }
    let enemy = put(&mut g, p.other(), "@token/goblin_army", Zone::Battlefield);
    g.state.objects[enemy as usize].counters.plus_one = 1;
    let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
    cast(&mut g, rage, None);
    for _ in 0..16 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert!(matches!(g.state.pending, Pending::EffectChoice { .. }));
    let enemy_ref = g.object_ref(enemy);
    reject(&mut g, Action::ChooseArmy { object: enemy_ref });
    let saved = g.snapshot();
    let mut child = g.clone();
    let choice = Action::ChooseArmy {
        object: g.object_ref(a),
    };
    act(&mut g, choice.clone());
    settle(&mut g);
    assert_eq!(stats(&g, a), (3, 3));
    assert_eq!(stats(&g, b), (1, 1));
    let idx = child
        .legal_actions()
        .iter()
        .position(|x| *x == choice)
        .unwrap();
    child.apply_index(p, child.revision(), idx).unwrap();
    settle(&mut child);
    assert_eq!(child.snapshot(), g.snapshot());
    g.restore(&saved).unwrap();
    let other = Action::ChooseArmy {
        object: g.object_ref(b),
    };
    act(&mut g, other);
    settle(&mut g);
    assert_eq!(stats(&g, b), (3, 3));
    assert_eq!(stats(&g, a), (1, 1));
}
fn army_type_fixture(g: &mut Game, id: CardId) {
    let recipient = g.object_ref(id);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .creature_types
        .push(subtypes::CreatureTypesEffectView {
            recipient,
            source: recipient,
            types: &["Army"],
            additional: false,
            timestamp,
        });
}
#[test]
fn hob_amass_noncreature_army_does_not_suppress_creation() {
    let p = Player::P0;
    let mut g = scenario(p);
    let rock = put(&mut g, p, "Heraldic Banner", Zone::Battlefield);
    army_type_fixture(&mut g, rock);
    assert!(g.has_subtype(rock, "Army"));
    assert!(!g.is_creature(rock));
    let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
    cast(&mut g, rage, None);
    settle(&mut g);
    let creatures: Vec<_> = armies(&g, p)
        .into_iter()
        .filter(|&id| g.is_creature(id))
        .collect();
    assert_eq!(creatures.len(), 1);
    assert_eq!(stats(&g, creatures[0]), (2, 2));
    assert_eq!(g.counter_count(rock, CounterKind::PlusOne), 0);
}
#[test]
fn hob_amass_subtype_waits_for_counters_and_expires_on_departure() {
    let p = Player::P0;
    let mut g = scenario(p);
    let army = put(&mut g, p, "Llanowar Elves", Zone::Battlefield);
    army_type_fixture(&mut g, army);
    put(&mut g, p, "Doubling Season", Zone::Battlefield);
    put(&mut g, p, "Doubling Season", Zone::Battlefield);
    let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
    cast(&mut g, rage, None);
    for _ in 0..16 {
        if matches!(g.state.pending, Pending::Replacement { .. }) {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert!(matches!(g.state.pending, Pending::Replacement { .. }));
    assert!(!g.has_subtype(army, "Goblin"));
    let saved = g.snapshot();
    let mut indexed = g.clone();
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::ChooseReplacement { .. }))
        .unwrap();
    let index = indexed
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    act(&mut g, action.clone());
    settle(&mut g);
    indexed.apply_index(p, indexed.revision(), index).unwrap();
    settle(&mut indexed);
    assert_eq!(indexed.snapshot(), g.snapshot());
    g.restore(&saved).unwrap();
    act(&mut g, action);
    settle(&mut g);
    assert_eq!(indexed.snapshot(), g.snapshot());
    assert!(g.has_subtype(army, "Goblin"));
    assert!(g.has_subtype(army, "Army"));
    assert_eq!(g.counter_count(army, CounterKind::PlusOne), 8);
    let bounce = put(&mut g, p, "Unsummon", Zone::Hand);
    cast(&mut g, bounce, Some(Target::Card(army)));
    settle(&mut g);
    assert!(!g.has_subtype(army, "Goblin"));
    assert!(!g.has_subtype(army, "Army"));
}
#[test]
fn hob_amass_token_replacement_creates_two_then_chooses_one() {
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Doubling Season", Zone::Battlefield);
    let rage = put(&mut g, p, "Rage into the Valley", Zone::Hand);
    cast(&mut g, rage, None);
    for _ in 0..16 {
        if matches!(g.state.pending, Pending::EffectChoice { .. }) {
            break;
        }
        act(&mut g, Action::Pass);
    }
    assert_eq!(armies(&g, p).len(), 2);
    choose(&mut g, |a| matches!(a, Action::ChooseArmy { .. }));
    settle(&mut g);
    let survivors = armies(&g, p);
    assert_eq!(survivors.len(), 1);
    assert_eq!(stats(&g, survivors[0]), (4, 4));
}
#[test]
fn hob_amass_optional_target_zero_and_all_invalid() {
    let p = Player::P0;
    let mut g = scenario(p);
    let gather = put(&mut g, p, "Gathering of Darkness", Zone::Hand);
    cast(&mut g, gather, None);
    settle(&mut g);
    assert_eq!(stats(&g, armies(&g, p)[0]), (3, 3));
    let dead = put(&mut g, p, "Llanowar Elves", Zone::Graveyard);
    let gather = put(&mut g, p, "Gathering of Darkness", Zone::Hand);
    cast(&mut g, gather, Some(Target::Card(dead)));
    // Fixture simulates an opposing response removing the sole target before resolution.
    g.move_object(dead, Zone::Exile);
    settle(&mut g);
    assert_eq!(stats(&g, armies(&g, p)[0]), (3, 3));
}
