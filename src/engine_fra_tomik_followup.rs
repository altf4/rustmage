//! Independent Tomik damage-replacement interactions.
use super::*;

fn event(g: &Game, source: CardId, target: Target, preventable: bool) -> DamageEvent {
    DamageEvent {
        source: g.damage_source(source),
        target,
        amount: 1,
        combat: false,
        preventable,
    }
}

#[test]
fn tomik_and_damage_doubler_respect_affected_players_replacement_order() {
    for p in [Player::P0, Player::P1] {
        for tomik_first in [false, true] {
            let mut g = scenario(p);
            let tomik = put(&mut g, p, "Tomik, Izzet Sparkmage", Zone::Battlefield);
            let doubler = put(&mut g, p, "Twinflame Tyrant", Zone::Battlefield);
            let source = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let damage = event(&g, source, Target::Player(p.other()), true);
            g.apply_damage_batch(&[damage]);
            g.advance_replacements(false);
            assert_eq!(g.actor(), Some(p.other()));
            let chosen = g.object_ref(if tomik_first { tomik } else { doubler });
            let action = Action::ChooseReplacement {
                source: chosen,
                ability: 0,
            };
            assert!(g.legal_actions().contains(&action));
            let saved = g.snapshot();
            super::super::contracts::branch_contract(g.clone());
            for replay in 0..2 {
                if replay == 1 {
                    g.restore(&saved).unwrap();
                }
                act(&mut g, action.clone());
                assert_eq!(
                    g.state.players[p.other().index()].life,
                    if tomik_first { 16 } else { 17 }
                );
            }
        }
    }
}

#[test]
fn copied_tomik_contributes_independently_and_loses_effect_when_blanked() {
    for p in [Player::P0, Player::P1] {
        for blanked in [false, true] {
            let mut g = scenario(p);
            let original = put(&mut g, p, "Tomik, Izzet Sparkmage", Zone::Battlefield);
            let copy = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let mut recipe = CopyRecipe::new(Card::from_name("Tomik, Izzet Sparkmage").unwrap());
            recipe.overrides = Some(CopyOverrides {
                legendary: Some(false),
                ..CopyOverrides::NONE
            });
            g.apply_temporary_copy(g.object_ref(copy), recipe, CopyDeadline::EndOfTurn);
            if blanked {
                let aura = put(&mut g, p.other(), "Fresh Start", Zone::Battlefield);
                g.state.objects[aura as usize].attached_to = Some(g.object_ref(copy));
            }
            let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let damage = event(&g, dealer, Target::Player(p.other()), true);
            g.apply_damage_batch(&[damage]);
            g.advance_replacements(false);
            if !blanked {
                let first = g.object_ref(original);
                act(
                    &mut g,
                    Action::ChooseReplacement {
                        source: first,
                        ability: 0,
                    },
                );
            }
            assert_eq!(
                g.state.players[p.other().index()].life,
                if blanked { 18 } else { 17 }
            );
        }
    }
}

#[test]
fn tomik_and_prevention_obey_recipient_choice_and_unpreventable_damage() {
    for p in [Player::P0, Player::P1] {
        for preventable in [false, true] {
            let mut g = scenario(p);
            let tomik = put(&mut g, p, "Tomik, Izzet Sparkmage", Zone::Battlefield);
            let dealer = put(&mut g, p, "Bear Cub", Zone::Battlefield);
            let prevention = put(&mut g, p.other(), "Forest", Zone::Battlefield);
            let source = g.object_ref(prevention);
            let recipient = g.object_ref(dealer);
            let timestamp = g.next_role_timestamp();
            g.state
                .roles
                .get_or_insert_default()
                .source_linked_effects
                .push(SourceLinkedEffectView {
                    source,
                    recipient,
                    controller: p.other(),
                    kind: SourceLinkedEffectKind::PreventDamageBy,
                    timestamp,
                });
            let damage = event(&g, dealer, Target::Player(p.other()), preventable);
            g.apply_damage_batch(&[damage]);
            g.advance_replacements(false);
            if preventable {
                let tomik_ref = g.object_ref(tomik);
                act(
                    &mut g,
                    Action::ChooseReplacement {
                        source: tomik_ref,
                        ability: 0,
                    },
                );
            }
            assert_eq!(
                g.state.players[p.other().index()].life,
                if preventable { 20 } else { 18 }
            );
        }
    }
}
