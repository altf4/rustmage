//! Focused printed outcomes; shared engine invariants live in core tests.
//! Key to the Side-Door's separate activated abilities and matching discard cost.
use super::*;

fn announce(g: &mut Game, card: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card,
            ability,
            counter_payment: Counters::default(),
        },
    );
}

#[test]
fn key_discard_requires_a_legendary_card_matching_a_controlled_legendary_permanent() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        let legend = put(&mut g, p, "Óin the Brave", Zone::Battlefield);
        let matching = put(&mut g, p, "Óin the Brave", Zone::Hand);
        let wrong = put(&mut g, p, "Forest", Zone::Hand);
        let enemy = put(&mut g, p.other(), "Óin the Brave", Zone::Hand);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
        announce(&mut g, key, 1);
        let wrong_ref = g.object_ref(wrong);
        let enemy_ref = g.object_ref(enemy);
        reject(&mut g, Action::ChooseAdditionalCost { object: wrong_ref });
        reject(&mut g, Action::ChooseAdditionalCost { object: enemy_ref });
        assert!(
            !g.observe(p.other())
                .objects
                .iter()
                .any(|o| o.id == matching)
        );

        let selected = g.object_ref(matching);
        act(&mut g, Action::ChooseAdditionalCost { object: selected });
        pay(&mut g);
        assert_eq!(g.object_view(matching).zone, Zone::Graveyard);
        assert!(g.object_view(key).tapped);
        assert_eq!(g.object_view(legend).zone, Zone::Battlefield);
        settle(&mut g);
        assert_eq!(g.hand_count(p), 3);
    }
}

#[test]
fn key_rejects_absent_wrong_controller_and_nonlegendary_card() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        put(&mut g, p, "Óin the Brave", Zone::Hand);
        let announce = Action::AnnounceActivate {
            card: key,
            ability: 1,
            counter_payment: Counters::default(),
        };
        reject(&mut g, announce.clone());
        put(&mut g, p.other(), "Óin the Brave", Zone::Battlefield);
        reject(&mut g, announce);
        assert!(!g.object_view(key).tapped);
    }
}

#[test]
fn key_funding_cannot_sacrifice_its_only_matching_legend_to_pay_the_mana_cost() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        let company = put(&mut g, p, "Bolg's Company", Zone::Battlefield);
        let legend = put(&mut g, p, "The Great Goblin", Zone::Battlefield);
        let discard = put(&mut g, p, "The Great Goblin", Zone::Hand);
        g.state.players[p.index()].mana = [0; 6];
        let announce = Action::AnnounceActivate {
            card: key,
            ability: 1,
            counter_payment: Counters::default(),
        };
        reject(&mut g, announce.clone());
        let fodder = put(&mut g, p, "Goblin Piker", Zone::Battlefield);
        act(&mut g, announce);
        let selected = g.object_ref(discard);
        act(&mut g, Action::ChooseAdditionalCost { object: selected });
        act(
            &mut g,
            Action::AnnounceMana {
                card: company,
                ability: 0,
                counter_payment: Counters::default(),
                production: [0, 0, 1, 1, 0, 0],
            },
        );
        let forbidden = g.object_ref(legend);
        reject(&mut g, Action::ChooseAdditionalCost { object: forbidden });
        let allowed = g.object_ref(fodder);
        act(&mut g, Action::ChooseAdditionalCost { object: allowed });
        pay(&mut g);
        pay(&mut g);
        settle(&mut g);
        assert_eq!(g.object_view(legend).zone, Zone::Battlefield);
        assert_eq!(g.object_view(fodder).zone, Zone::Graveyard);
        assert_eq!(g.object_view(discard).zone, Zone::Graveyard);
    }
}

#[test]
fn key_unblockable_effect_prevents_actual_blocking_only_this_turn() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let blocker = put(&mut g, p.other(), "Hill Giant", Zone::Battlefield);
        announce(&mut g, key, 0);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(attacker),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        settle(&mut g);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(&mut g, Action::Attack { card: attacker });
        act(&mut g, Action::FinishAttackers);
        for _ in 0..8 {
            if g.state.pending == Pending::Blockers {
                break;
            }
            act(&mut g, Action::Pass);
        }
        assert_eq!(g.state.pending, Pending::Blockers);
        reject(&mut g, Action::Block { blocker, attacker });
        act(&mut g, Action::FinishBlockers);
        g.finish_cleanup();
        assert!(!g.keywords(attacker).contains(Keywords::CANT_BE_BLOCKED));
    }
}
