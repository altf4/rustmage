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
                super::contracts::branch_contract(g.clone());
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

#[test]
fn clash_rejects_lands_and_fizzles_if_target_changes_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let target = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let land = put(&mut g, p.other(), "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Clash of Elements", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(land),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(target),
            },
        );
        pay(&mut g);
        g.move_object(target, Zone::Exile);
        g.move_object(target, Zone::Battlefield);
        let life = g.state.players[p.other().index()].life;
        settle(&mut g);
        assert_eq!(g.object_view(target).zone, Zone::Battlefield);
        assert_eq!(g.state.players[p.other().index()].life, life);
    }
}
