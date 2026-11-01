//! Explicitly synthetic composite loyalty and mana abilities on gated Blossombind.
use super::*;

fn owned_step(g: &mut Game, action: Action) {
    let player = g.actor().unwrap();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let parent = g.clone();
    let mut indexed = parent.clone();
    drop(parent);
    let checkpoint = indexed.snapshot();
    indexed.apply_index(player, revision, index).unwrap();
    act(g, action);
    assert_eq!(g.snapshot(), indexed.snapshot());
    let expected = indexed.snapshot();
    indexed.restore(&checkpoint).unwrap();
    indexed
        .apply_index(player, indexed.revision(), index)
        .unwrap();
    assert_eq!(indexed.snapshot(), expected);
}

fn owned_pay(g: &mut Game) {
    let action = g
        .legal_actions()
        .into_iter()
        .find(|a| matches!(a, Action::PayCosts { .. }))
        .unwrap();
    owned_step(g, action);
}

fn board(player: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(player);
    let walker = put(&mut g, player, "Bear Cub", Zone::Battlefield);
    g.state.objects[walker as usize].card = Card::JacePlaneswalkerToken;
    g.state.objects[walker as usize].counters.loyalty = 3;
    let object = g.object_ref(walker);
    let timestamp = g.next_role_timestamp();
    g.state
        .roles
        .get_or_insert_with(Default::default)
        .animations
        .push(AnimationView {
            object,
            timestamp,
            artifact: false,
            land: false,
            preserve_card_types: false,
            food_only: false,
            artifact_noncreature: false,
            enchantment_noncreature: false,
            colors: None,
            creature_types: None,
            all_creature_types: false,
            base: Some((3, 3)),
            keywords: Keywords::NONE,
            expires_turn: None,
        });
    let aura = put(&mut g, player, "Pacifism", Zone::Battlefield);
    g.state.objects[aura as usize].card = Card::from_name("Blossombind").unwrap();
    g.state.objects[aura as usize].attached_to = Some(object);
    assert!(replacements::counters_prohibited(&g, object));
    (g, walker, aura)
}

#[test]
fn mana_window_must_remove_prohibition_before_positive_loyalty_payment() {
    let _mana = crate::cards::sacrifice_mana_fixture::Guard::new();
    for target_sensitive in [false, true] {
        let _loyalty = if target_sensitive {
            crate::cards::composite_loyalty_fixture::Guard::with_mana_positive_target()
        } else {
            crate::cards::composite_loyalty_fixture::Guard::with_mana_positive()
        };
        for player in [Player::P0, Player::P1] {
            let (mut g, walker, aura) = board(player);
            let target = put(&mut g, player.other(), "Bear Cub", Zone::Battlefield);
            let announce = Action::AnnounceActivate {
                card: walker,
                ability: 0,
                counter_payment: Counters::default(),
            };
            assert!(g.legal_actions().contains(&announce));
            assert!(!g.legal_actions().iter().any(|a| matches!(a,
            Action::Activate { card, ability: 0, .. }
            | Action::BeginActivate { card, ability: 0, .. } if *card == walker)));
            owned_step(&mut g, announce);
            if target_sensitive {
                owned_step(
                    &mut g,
                    Action::ChooseTarget {
                        target: Target::Card(target),
                    },
                );
                if g.legal_actions().contains(&Action::FinishTargets) {
                    owned_step(&mut g, Action::FinishTargets);
                }
            }
            assert!(g.mana_payment_domain().is_none());
            reject(
                &mut g,
                Action::PayCosts {
                    payment: if target_sensitive {
                        [0; 6]
                    } else {
                        [0, 0, 0, 0, 1, 0]
                    },
                },
            );
            let mana = g.legal_actions().into_iter().find(|a| matches!(a,
            Action::ActivateMana { card, .. } | Action::AnnounceMana { card, .. } if *card == aura
        )).expect("the Aura can sacrifice itself as a mana ability");
            owned_step(&mut g, mana);
            if matches!(g.state.pending, Pending::PayCosts { .. })
                && g.state
                    .announcing
                    .as_ref()
                    .is_some_and(|a| a.production.is_some())
            {
                owned_pay(&mut g);
            }
            assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
            assert!(g.mana_payment_domain().is_some());
            owned_pay(&mut g);
            assert_eq!(g.counter_count(walker, CounterKind::Loyalty), 4);
            assert!(g.object_view(walker).loyalty_activated_this_turn);
        }
    }
}

#[test]
fn mana_in_pool_does_not_pay_a_prohibited_loyalty_cost() {
    let _loyalty = crate::cards::composite_loyalty_fixture::Guard::with_mana_positive();
    for player in [Player::P0, Player::P1] {
        let (mut g, walker, _) = board(player);
        let announce = Action::AnnounceActivate {
            card: walker,
            ability: 0,
            counter_payment: Counters::default(),
        };
        assert!(!g.legal_actions().contains(&announce));
        reject(&mut g, announce);
    }
}
