//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn clash_start(g: &mut Game, spell: CardId, target: CardId) {
    act(g, Action::AnnounceCast { card: spell });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(target),
        },
    );
    pay(g);
}
fn clash_choice(g: &mut Game) {
    for _ in 0..48 {
        if matches!(g.state.pending, Pending::TargetLibrary { .. }) {
            return;
        }
        choose(g, |a| matches!(a, Action::Pass | Action::PutTrigger { .. }));
    }
    panic!("owner library choice did not start");
}

#[test]
fn clash_owner_chooses_library_destination_and_top_deals_damage() {
    for caster in [Player::P0, Player::P1] {
        for owner in [caster, caster.other()] {
            for destination in [LibraryDestination::Top, LibraryDestination::Bottom] {
                let mut g = scenario(caster);
                let target = put(&mut g, owner, "Bear Cub", Zone::Battlefield);
                if owner != caster {
                    g.change_controller(g.object_ref(target), caster);
                }
                let source = put(&mut g, caster, "Clash of Elements", Zone::Hand);
                let object = g.object_ref(target);
                let life = g.state.players[owner.index()].life;
                let other_life = g.state.players[owner.other().index()].life;
                clash_start(&mut g, source, target);
                clash_choice(&mut g);
                assert_eq!(g.actor(), Some(owner));

                act(
                    &mut g,
                    Action::ChooseLibraryDestination {
                        object,
                        destination,
                    },
                );
                settle(&mut g);
                assert_eq!(g.object_view(target).zone, Zone::Library);
                let library = &g.state.players[owner.index()].library;
                assert_eq!(
                    if destination == LibraryDestination::Top {
                        library.last()
                    } else {
                        library.first()
                    },
                    Some(&target)
                );
                assert_eq!(
                    g.state.players[owner.index()].life,
                    life - if destination == LibraryDestination::Top {
                        2
                    } else {
                        0
                    }
                );
                assert_eq!(g.state.players[owner.other().index()].life, other_life);
            }
        }
    }
}
