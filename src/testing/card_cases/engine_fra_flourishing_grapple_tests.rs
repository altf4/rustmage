//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn grapple_cast(g: &mut Game, spell: CardId, victim: CardId, dealer: CardId) {
    act(g, Action::AnnounceCast { card: spell });
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(victim),
        },
    );
    act(
        g,
        Action::ChooseTarget {
            target: Target::Card(dealer),
        },
    );
    pay(g);
}

#[test]
fn grapple_removes_abilities_before_power_damage_and_until_cleanup() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
        assert!(g.keywords(victim).contains(Keywords::FLYING));
        grapple_cast(&mut g, spell, victim, dealer);

        settle(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(g.state.objects[victim as usize].damage, 2);
        assert!(!g.keywords(victim).contains(Keywords::FLYING));
        assert!(!g.keywords(victim).contains(Keywords::VIGILANCE));
        g.finish_cleanup();
        assert!(g.keywords(victim).contains(Keywords::FLYING));
        assert!(g.keywords(victim).contains(Keywords::VIGILANCE));
    }
}

#[test]
fn grapple_requires_opponents_red_or_white_creature_or_planeswalker() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let victim = put(&mut g, p.other(), "Serra Angel", Zone::Battlefield);
        let green = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let own = put(&mut g, p, "Serra Angel", Zone::Battlefield);
        let spell = put(&mut g, p, "Flourishing Grapple", Zone::Hand);
        act(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(green),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        reject(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(green),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(own),
            },
        );
        pay(&mut g);
    }
}
