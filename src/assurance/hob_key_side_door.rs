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
    for p in [Player::P0, Player::P1] {
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
        contracts::branch_contract(g.clone());
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
    for p in [Player::P0, Player::P1] {
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
fn key_unblockable_targets_either_players_creature_and_pays_two() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        let creature = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
        announce(&mut g, key, 0);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        assert!(g.object_view(key).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert!(g.keywords(creature).contains(Keywords::CANT_BE_BLOCKED));
    }
}

#[test]
fn key_funding_cannot_sacrifice_its_only_matching_legend_to_pay_the_mana_cost() {
    for p in [Player::P0, Player::P1] {
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
fn key_uses_current_copiable_name_and_legendary_supertype() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        let body = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        put(&mut g, p, "Óin the Brave", Zone::Hand);
        let announce = Action::AnnounceActivate {
            card: key,
            ability: 1,
            counter_payment: Counters::default(),
        };
        reject(&mut g, announce.clone());
        let mut recipe = g.copiable_recipe(g.object_ref(body));
        recipe.overrides = Some(CopyOverrides {
            name: Some("Óin the Brave"),
            legendary: Some(true),
            ..CopyOverrides::NONE
        });
        g.set_copy_recipe(g.object_ref(body), recipe);
        assert!(g.legal_actions().contains(&announce));
        recipe.overrides = Some(CopyOverrides {
            name: Some("Óin the Brave"),
            legendary: Some(false),
            ..CopyOverrides::NONE
        });
        g.set_copy_recipe(g.object_ref(body), recipe);
        reject(&mut g, announce);
    }
}

#[test]
fn key_discard_replacement_and_owned_restore_follow_actual_cost() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        put(&mut g, p, "Óin the Brave", Zone::Battlefield);
        let matching = put(&mut g, p, "Óin the Brave", Zone::Hand);
        put(&mut g, p, "Rest in Peace", Zone::Battlefield);
        announce(&mut g, key, 1);
        let selected = g.object_ref(matching);
        act(&mut g, Action::ChooseAdditionalCost { object: selected });
        contracts::branch_contract(g.clone());
        let before = g.snapshot();
        let mut child = g.clone();
        drop(g);
        pay(&mut child);
        settle(&mut child);
        assert_eq!(child.object_view(matching).zone, Zone::Exile);
        let after = child.snapshot();
        child.restore(&before).unwrap();
        pay(&mut child);
        settle(&mut child);
        assert_eq!(child.snapshot(), after);
    }
}

#[test]
fn key_unblockable_effect_prevents_actual_blocking_only_this_turn() {
    for p in [Player::P0, Player::P1] {
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

#[test]
fn key_unblockable_does_not_apply_to_a_new_target_incarnation() {
    for p in [Player::P0, Player::P1] {
        let mut g = scenario(p);
        let key = put(&mut g, p, "Key to the Side-Door", Zone::Battlefield);
        let creature = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        announce(&mut g, key, 0);
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(creature),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        pay(&mut g);
        g.move_object(creature, Zone::Exile);
        g.move_object(creature, Zone::Battlefield);
        settle(&mut g);
        assert!(!g.keywords(creature).contains(Keywords::CANT_BE_BLOCKED));
    }
}
