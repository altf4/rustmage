//! Focused printed outcomes; shared engine invariants live in core tests.
//! Combat action restrictions and live split second providers.
use super::*;

#[test]
fn samut_split_second_blocks_spells_and_nonmana_activations_while_spell_remains() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let samut = put(&mut g, p, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let creature = put(&mut g, p, "Surveillance Phantasm", Zone::Battlefield);
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        let shock = put(&mut g, p, "Shock", Zone::Hand);
        let followup = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, shock, Some(Target::Player(p.other())));
        assert!(g.nonmana_actions_prohibited());
        reject(&mut g, Action::AnnounceCast { card: followup });
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: creature,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert!(
            g.legal_actions()
                .contains(&Action::TapMana { card: forest })
        );

        g.move_object(samut, Zone::Graveyard);
        assert!(!g.nonmana_actions_prohibited());
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: followup })
        );
    }
}

#[test]
fn yuriko_forbids_nonspecific_casting_and_nonmana_activations_only_in_combat() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        let yuriko = put(&mut g, p, "Yuriko, Blade of the Mighty", Zone::Battlefield);
        let creature = put(&mut g, p, "Surveillance Phantasm", Zone::Battlefield);
        let forest = put(&mut g, p, "Forest", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        assert!(!g.nonmana_actions_prohibited());
        g.state.step = Step::BeginCombat;
        assert!(g.nonmana_actions_prohibited());
        reject(&mut g, Action::AnnounceCast { card: spell });
        reject(
            &mut g,
            Action::AnnounceActivate {
                card: creature,
                ability: 0,
                counter_payment: Counters::default(),
            },
        );
        assert!(
            g.legal_actions()
                .contains(&Action::TapMana { card: forest })
        );

        g.move_object(yuriko, Zone::Graveyard);
        assert!(!g.nonmana_actions_prohibited());
        assert!(
            g.legal_actions()
                .contains(&Action::AnnounceCast { card: spell })
        );
    }
}

#[test]
fn samut_lock_uses_live_provider_and_only_its_controllers_spells() {
    {
        let p = Player::P0;
        let mut g = scenario(p.other());
        let samut = put(&mut g, p, "Samut, Tyrant of Naktamun", Zone::Battlefield);
        let opponent_spell = put(&mut g, p.other(), "Shock", Zone::Hand);
        cast(&mut g, opponent_spell, Some(Target::Player(p)));
        assert!(!g.nonmana_actions_prohibited());
        settle(&mut g);
        g.state.priority = p;
        let own_spell = put(&mut g, p, "Shock", Zone::Hand);
        cast(&mut g, own_spell, Some(Target::Player(p.other())));
        assert!(g.nonmana_actions_prohibited());
        let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
        g.state.objects[aura as usize].attached_to = Some(g.object_ref(samut));
        assert!(!g.nonmana_actions_prohibited());
        g.move_object(aura, Zone::Graveyard);
        assert!(g.nonmana_actions_prohibited());
    }
}

#[test]
fn yuriko_does_not_trigger_for_a_planeswalker_attacked_alone() {
    {
        let p = Player::P0;
        let mut g = scenario(p);
        put(&mut g, p, "Yuriko, Blade of the Mighty", Zone::Battlefield);
        let attacker = put(&mut g, p, "Bear Cub", Zone::Battlefield);
        let walker = put(
            &mut g,
            p.other(),
            "Ajani, Caller of the Pride",
            Zone::Battlefield,
        );
        g.add_counter(walker, CounterKind::Loyalty, 3);
        let defender = g.object_ref(walker);
        g.state.step = Step::Attackers;
        g.state.pending = Pending::Attackers;
        act(
            &mut g,
            Action::AttackPlaneswalker {
                card: attacker,
                defender,
            },
        );
        act(&mut g, Action::FinishAttackers);
        settle(&mut g);
        assert!(!g.keywords(attacker).contains(Keywords::DOUBLE_STRIKE));
    }
}
