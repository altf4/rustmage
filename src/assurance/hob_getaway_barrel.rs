//! Public reveal, deterministic random creature and random remainder.
use super::*;
fn library(g: &mut Game, p: Player, names: &[&str]) -> Vec<CardId> {
    for id in std::mem::take(&mut g.state.players[p.index()].library) {
        g.state.objects[id as usize].zone = Zone::Outside;
    }
    let mut ids: Vec<_> = names
        .iter()
        .rev()
        .map(|n| put(g, p, n, Zone::Library))
        .collect();
    ids.reverse();
    ids
}
fn wait_reveal(g: &mut Game) {
    for _ in 0..40 {
        if matches!(g.state.pending, Pending::EffectLibrarySelection { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("barrel reveal not reached: {:?}", g.state.pending);
}
fn paid_and_destroyed(g: &mut Game, p: Player) -> CardId {
    let barrel = put(g, p, "Getaway Barrel", Zone::Hand);
    g.state.players[p.index()].mana = [0, 0, 0, 1, 0, 3];
    cast(g, barrel, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    settle(g);
    let spell = put(g, p, "Disenchant", Zone::Hand);
    g.state.players[p.index()].mana = [1, 0, 0, 0, 0, 1];
    cast(g, spell, Some(Target::Card(barrel)));
    barrel
}
#[test]
fn getaway_barrel_paid_destroy_public_reveal_and_rng_follow_exact_top_thirteen() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let mut names = vec!["Forest"; 17];
        names[0] = "Bear Cub";
        names[3] = "Hill Giant";
        names[12] = "Llanowar Elves";
        names[13] = "Grizzly Bears";
        let ids = library(&mut g, p, &names);
        let barrel = paid_and_destroyed(&mut g, p);
        wait_reveal(&mut g);
        assert_eq!(g.object_view(barrel).zone, Zone::Graveyard);
        let view = g.observe(p.other()).library_selection.unwrap();
        assert!(view.public && view.random && view.reveal);
        assert_eq!(
            view.remaining.iter().map(|v| v.id).collect::<Vec<_>>(),
            ids[..13]
        );
        assert!(!g.observe(p.other()).objects.iter().any(|o| o.id == ids[13]));
        assert_eq!(
            g.legal_actions()
                .into_iter()
                .filter(|a| *a != Action::Concede)
                .collect::<Vec<_>>(),
            vec![Action::FinishLibrarySelection]
        );
        reject(&mut g, Action::SelectLibrary { card: ids[0] });
        contracts::branch_contract(g.clone());
        let mut rng = g.state.rng;
        let eligible = [ids[0], ids[3], ids[12]];
        let chosen = eligible[rng.below(eligible.len() as u64) as usize];
        let mut bottom = ids[..13]
            .iter()
            .copied()
            .filter(|id| *id != chosen)
            .collect::<Vec<_>>();
        rng.shuffle(&mut bottom);
        let expected = bottom
            .into_iter()
            .rev()
            .chain(ids[13..].iter().rev().copied())
            .collect::<Vec<_>>();
        let snap = g.snapshot();
        let mut branch = g.clone();
        drop(g);
        act(&mut branch, Action::FinishLibrarySelection);
        settle(&mut branch);
        assert_eq!(branch.object_view(chosen).zone, Zone::Battlefield);
        assert_eq!(branch.state.players[p.index()].library.to_vec(), expected);
        assert_eq!(branch.state.rng, rng);
        let result = branch.snapshot();
        branch.restore(&snap).unwrap();
        act(&mut branch, Action::FinishLibrarySelection);
        settle(&mut branch);
        assert_eq!(branch.snapshot(), result);
    }
}
#[test]
fn getaway_barrel_no_creature_or_empty_library_and_replaced_death() {
    for n in [0, 1, 5] {
        let p = Player::P0;
        let mut g = scenario(p);
        let ids = library(&mut g, p, &vec!["Forest"; n]);
        let barrel = paid_and_destroyed(&mut g, p);
        if n > 0 {
            wait_reveal(&mut g);
            assert_eq!(
                g.observe(p.other())
                    .library_selection
                    .unwrap()
                    .remaining
                    .len(),
                n
            );
            act(&mut g, Action::FinishLibrarySelection);
        }
        settle(&mut g);
        assert_eq!(g.object_view(barrel).zone, Zone::Graveyard);
        let mut remaining = g.state.players[p.index()].library.to_vec();
        remaining.sort_unstable();
        let mut expected = ids;
        expected.sort_unstable();
        assert_eq!(remaining, expected);
    }
    let p = Player::P0;
    let mut g = scenario(p);
    let ids = library(&mut g, p, &["Bear Cub", "Forest"]);
    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
    let barrel = paid_and_destroyed(&mut g, p);
    settle(&mut g);
    assert_eq!(g.object_view(barrel).zone, Zone::Exile);
    assert_eq!(
        g.state.players[p.index()].library.to_vec(),
        ids.into_iter().rev().collect::<Vec<_>>()
    );
}
#[test]
fn getaway_barrel_random_creature_entry_choice_suspends_before_bottoming() {
    let p = Player::P0;
    let mut g = scenario(p);
    let ids = library(&mut g, p, &["Adaptive Automaton", "Forest", "Island"]);
    paid_and_destroyed(&mut g, p);
    wait_reveal(&mut g);
    act(&mut g, Action::FinishLibrarySelection);
    assert!(g.observe(p).entry_value.is_some());
    assert!(g.observe(p.other()).library_selection.is_some());
    assert_eq!(g.object_view(ids[1]).zone, Zone::Library);
    contracts::branch_contract(g.clone());
    act(
        &mut g,
        Action::ChooseEntryValue {
            value: EntryChoiceValue::CreatureType(
                crate::cards::CreatureType::from_name("Elf").unwrap(),
            ),
        },
    );
    settle(&mut g);
    assert_eq!(g.object_view(ids[0]).zone, Zone::Battlefield);
    assert_eq!(g.state.players[p.index()].library.len(), 2);
}

#[test]
fn getaway_barrel_death_uses_last_controller_library_not_physical_owner() {
    for controller in [Player::P0, Player::P1] {
        let mut g = scenario(controller);
        let owner = controller.other();
        let own_top = library(&mut g, owner, &["Hill Giant", "Forest"]);
        let controlled_top = library(&mut g, controller, &["Bear Cub", "Island"]);
        let barrel = put(&mut g, owner, "Getaway Barrel", Zone::Battlefield);
        let reference = g.object_ref(barrel);
        g.change_controller(reference, controller);
        let disenchant = put(&mut g, controller, "Disenchant", Zone::Hand);
        g.state.players[controller.index()].mana = [1, 0, 0, 0, 0, 1];
        cast(&mut g, disenchant, Some(Target::Card(barrel)));
        wait_reveal(&mut g);
        assert_eq!(g.actor(), Some(controller));
        assert_eq!(g.object_view(barrel).zone, Zone::Graveyard);
        let view = g.observe(owner).library_selection.unwrap();
        assert_eq!(
            view.remaining.iter().map(|v| v.id).collect::<Vec<_>>(),
            controlled_top
        );
        act(&mut g, Action::FinishLibrarySelection);
        settle(&mut g);
        assert_eq!(g.object_view(controlled_top[0]).zone, Zone::Battlefield);
        assert_eq!(
            g.state.players[owner.index()].library.to_vec(),
            own_top.into_iter().rev().collect::<Vec<_>>()
        );
    }
}
