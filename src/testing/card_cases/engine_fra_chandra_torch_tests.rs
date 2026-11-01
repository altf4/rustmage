//! Focused printed outcomes; shared engine invariants live in core tests.
use super::*;

fn torch_activate(g: &mut Game, id: CardId, ability: u16) {
    act(
        g,
        Action::AnnounceActivate {
            card: id,
            ability,
            counter_payment: Counters::default(),
        },
    );
    pay(g);
}

#[test]
fn torch_mana_and_damage_modes_obey_loyalty_and_targets() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        g.state.players[player.index()].mana = [0; 6];
        torch_activate(&mut g, chandra, 1);
        g.resolve_top();
        assert_eq!(g.state.players[player.index()].mana[3], 2);
        assert_eq!(g.counter_count(chandra, CounterKind::Loyalty), 5);
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        let victim = put(&mut g, player.other(), "Hill Giant", Zone::Battlefield);
        g.state.objects[victim as usize].growth.toughness = 4;
        g.state.objects[chandra as usize].counters.loyalty = 4;
        act(
            &mut g,
            Action::AnnounceActivate {
                card: chandra,
                ability: 2,
                counter_payment: Counters::default(),
            },
        );
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Card(victim),
            },
        );
        pay(&mut g);
        g.resolve_top();
        assert_eq!(g.object_view(victim).damage, 4);
    }
}

#[test]
fn torch_emblem_survives_source_and_triggers_for_its_controllers_casts() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 8;
        torch_activate(&mut g, chandra, 3);
        g.resolve_top();
        g.move_object(chandra, Zone::Graveyard);
        assert!(g.state.objects.iter().any(|o| o.zone == Zone::Command
            && o.card == Card::from_name("@emblem/fra-chandra-torch").unwrap()));
        let spell = put(&mut g, player, "Bear Cub", Zone::Hand);
        g.state.players[player.index()].mana = [0, 0, 0, 0, 1, 1];
        cast(&mut g, spell, None);
        g.prepare_triggers();
        act(
            &mut g,
            Action::ChooseTarget {
                target: Target::Player(player.other()),
            },
        );
        if g.legal_actions().contains(&Action::FinishTargets) {
            act(&mut g, Action::FinishTargets);
        }
        let trigger = g
            .state
            .stack
            .iter()
            .find(|s| {
                s.source.is_some_and(|source| {
                    source.card == Card::from_name("@emblem/fra-chandra-torch").unwrap()
                })
            })
            .expect("emblem cast trigger");
        assert_eq!(trigger.controller, player);
        let life = g.state.players[player.other().index()].life;
        g.resolve_top();
        assert_eq!(g.state.players[player.other().index()].life, life - 5);
    }
}

#[test]
fn torch_paid_offer_preserves_chosen_x_and_normal_mana_cost() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        let spell = put(&mut g, player, "Ajani's Anguish", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 1, 0, 2];
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let object = g.object_ref(spell);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        act(&mut g, Action::ChooseX { value: 2 });
        pay(&mut g);
        assert_eq!(g.state.stack.last().unwrap().x, Some(2));
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        act(&mut g, Action::FinishResolutionCasts);
    }
}

#[test]
fn torch_paid_offer_keeps_modal_spell_choices_and_opponent_tax() {
    {
        let player = Player::P0;
        let mut g = scenario(player);
        let chandra = put(
            &mut g,
            player,
            "Chandra, Torch of Defiance",
            Zone::Battlefield,
        );
        g.state.objects[chandra as usize].counters.loyalty = 4;
        put(
            &mut g,
            player.other(),
            "Thalia, the Survivor",
            Zone::Battlefield,
        );
        let charm = put(&mut g, player, "Konstrari Charm", Zone::Library);
        g.state.players[player.index()].mana = [0, 0, 0, 1, 1, 1];
        torch_activate(&mut g, chandra, 0);
        g.resolve_top();
        let object = g.object_ref(charm);
        act(
            &mut g,
            Action::CastExiledDuringResolution {
                object,
                face: CastFace::Normal,
            },
        );
        act(&mut g, Action::ChooseMode { mode: 2 });
        pay(&mut g);
        assert_eq!(g.state.players[player.index()].mana, [0; 6]);
        assert_eq!(g.state.stack.last().unwrap().mode, Some(2));
        act(&mut g, Action::FinishResolutionCasts);
    }
}
