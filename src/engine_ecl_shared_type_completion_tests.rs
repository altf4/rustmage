//! Whole target groups must be completable before a cast or target prefix commits.
use super::ability_timeline_tests::{checked, named, settle, setup};
use super::*;
fn reject(g: &mut Game, a: Action) {
    let s = g.snapshot();
    assert_eq!(g.apply(g.actor().unwrap(), a), Err(Error::IllegalAction));
    assert_eq!(g.snapshot(), s);
}
#[test]
fn ecl_unbury_unrelated_graveyard_cards_never_offer_the_two_target_mode() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        named(&mut g, p, "Scarblade Scout", Zone::Graveyard);
        let goblin = named(
            &mut g,
            p,
            "Grub, Storied Matriarch // Grub, Notorious Auntie",
            Zone::Graveyard,
        );
        let spell = named(&mut g, p, "Unbury", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        reject(&mut g, Action::ChooseMode { mode: 1 });
        checked(&mut g, Action::ChooseMode { mode: 0 });
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(goblin),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 1],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(goblin).zone, Zone::Hand);
    }
}
#[test]
fn ecl_unbury_first_target_requires_a_distinct_shared_type_completion() {
    for p in [Player::P0, Player::P1] {
        let mut g = setup(p);
        let elf = named(&mut g, p, "Scarblade Scout", Zone::Graveyard);
        let a = named(
            &mut g,
            p,
            "Grub, Storied Matriarch // Grub, Notorious Auntie",
            Zone::Graveyard,
        );
        let b = named(&mut g, p, "Boggart Prankster", Zone::Graveyard);
        let spell = named(&mut g, p, "Unbury", Zone::Hand);
        checked(&mut g, Action::AnnounceCast { card: spell });
        checked(&mut g, Action::ChooseMode { mode: 1 });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(elf),
            },
        );
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(a),
            },
        );
        reject(&mut g, Action::FinishTargets);
        checked(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(b),
            },
        );
        checked(
            &mut g,
            Action::PayCosts {
                payment: [0, 0, 1, 0, 0, 1],
            },
        );
        settle(&mut g);
        assert_eq!(g.object_view(a).zone, Zone::Hand);
        assert_eq!(g.object_view(b).zone, Zone::Hand);
    }
}
