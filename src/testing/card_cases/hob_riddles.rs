//! Focused printed outcomes; shared engine invariants live in core tests.
//! Private piles, opponent choice, and actual hidden/public zone movement.
use super::*;
fn until(g: &mut Game, ready: impl Fn(&Game) -> bool) {
    for _ in 0..64 {
        if ready(g) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("Riddles decision budget {:?}", g.state.pending);
}
fn finish(g: &mut Game) {
    until(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.resolving.is_none()
            && g.state.pending_triggers.is_empty()
    });
}
fn cast_riddles(g: &mut Game, p: Player) -> CardId {
    let id = put(g, p, "Riddles in the Dark", Zone::Hand);
    g.state.active = p.other();
    g.state.step = Step::End;
    g.state.priority = p;
    g.state.players[p.index()].mana = [0, 1, 0, 0, 0, 2];
    cast(g, id, None);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
    id
}
fn wait_choice(g: &mut Game) {
    until(g, |g| {
        matches!(g.state.pending, Pending::LinkedChoice { .. })
    });
}
fn assign(g: &mut Game, id: CardId, pile: LibraryPile) {
    let object = g.object_ref(id);
    act(g, Action::AssignLibraryPile { object, pile });
}

#[test]
fn riddles_short_and_empty_libraries_both_empty_pile_choices_and_replacements() {
    for size in [0, 2] {
        for up in [false, true] {
            for replacement in [false, true] {
                let p = Player::P0;
                let mut g = scenario(p);
                while g.state.players[p.index()].library.len() > size {
                    let id = *g.state.players[p.index()].library.last().unwrap();
                    g.move_object(id, Zone::Exile);
                }
                if replacement {
                    put(&mut g, p, "Rest in Peace", Zone::Battlefield);
                }
                let top = g.state.players[p.index()].library.to_vec();
                cast_riddles(&mut g, p);
                wait_choice(&mut g);
                for id in top.iter().copied() {
                    assign(&mut g, id, LibraryPile::FaceDown);
                }
                assert_eq!(g.actor(), Some(p.other()));
                assert_eq!(
                    g.observe(p.other()).library_piles.unwrap().face_down_count,
                    size
                );
                act(
                    &mut g,
                    Action::ChooseLibraryPile {
                        pile: if up {
                            LibraryPile::FaceUp
                        } else {
                            LibraryPile::FaceDown
                        },
                    },
                );
                finish(&mut g);
                for id in top {
                    assert_eq!(
                        g.object_view(id).zone,
                        if !up {
                            Zone::Hand
                        } else if replacement {
                            Zone::Exile
                        } else {
                            Zone::Graveyard
                        }
                    );
                }
                assert_eq!(g.hand_count(p), if up { 0 } else { size });
                assert!(g.state.players[p.index()].library.is_empty());
                assert!(g.outcome().is_none());
            }
        }
    }
}
#[test]
fn riddles_countered_before_resolution_never_exposes_piles() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let spell = cast_riddles(&mut g, p);
        if g.actor() == Some(p) {
            act(&mut g, Action::Pass);
        }
        let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
        g.state.players[p.other().index()].mana = [0, 1, 0, 0, 0, 1];
        cast(&mut g, negate, Some(Target::Spell(spell)));
        finish(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.players[p.index()].library.len(), 8);
        assert!(g.observe(p).library_piles.is_none());
        assert!(g.observe(p.other()).library_piles.is_none());
    }
}
