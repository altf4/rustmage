//! Printed attachment costs are independent of the granted template's source costs.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const UNTAP: ActivatedAbility = ActivatedAbility {
    cost: ActivationCost {
        generic: 5,
        mana_component: true,
        ..ActivationCost::ZERO
    },
    targets: TargetPlan::NONE,
    program: &[Instruction::UntapSource],
    guard: ActivationGuard::Always,
    origin: ActivationOrigin::Battlefield,
    exhaust: false,
    cycling: false,
    sorcery: false,
    once_per_turn: false,
};
const NO_TAP: CardRules = CardRules {
    effect: Effect::Aura(TargetPlan::Single(TargetSpec::one(TargetRule::Creature))),
    aura: Some(AuraSpec {
        kinds: 1,
        relation: -1,
        bonus: EquipmentSpec {
            stats: Stats {
                power: 0,
                toughness: 0,
            },
            keywords: Keywords::NONE,
        },
    }),
    static_groups: &[StaticAbilityGroup {
        line: 0,
        first_layer: AbilityLayer::Ability,
        fragments: &[StaticAbilityFragment::AttachmentActivation {
            ability: 0,
            tap_attachment: false,
        }],
    }],
    activated: &[UNTAP],
    ..CardRules::EMPTY
};
const SACRIFICE_FUNDER: CardRules = CardRules {
    activated: &[
        UNTAP,
        ActivatedAbility {
            cost: ActivationCost {
                tap: true,
                sacrifice: true,
                ..ActivationCost::ZERO
            },
            targets: TargetPlan::NONE,
            program: &[Instruction::AddMana(ManaProduction::Fixed([
                0, 0, 0, 0, 0, 5,
            ]))],
            guard: ActivationGuard::Always,
            origin: ActivationOrigin::Battlefield,
            exhaust: false,
            cycling: false,
            sorcery: false,
            once_per_turn: false,
        },
    ],
    ..NO_TAP
};
fn aura_card() -> Card {
    Card::from_name("Pacifism").unwrap()
}
fn setup(recipient_owner: Player, aura_owner: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(aura_owner);
    let recipient = put(&mut g, recipient_owner, "Bear Cub", Zone::Battlefield);
    let aura = put(&mut g, aura_owner, "Pacifism", Zone::Hand);
    cast(&mut g, aura, Some(Target::Card(recipient)));
    settle(&mut g);
    assert_eq!(
        g.object_view(aura).attached_to,
        Some(g.object_ref(recipient))
    );
    if g.actor() != Some(recipient_owner) {
        act(&mut g, Action::Pass);
    }
    g.state.objects[recipient as usize].tapped = true;
    g.state.objects[recipient as usize].summoning_sick = true;
    g.state.players[recipient_owner.index()].mana = [0, 0, 0, 0, 0, 5];
    (g, recipient, aura)
}
fn start(g: &mut Game, aura: CardId, ability: u16) {
    let grantor = g.object_ref(aura);
    act(g, Action::AnnounceAttachedAbility { grantor, ability });
}
fn paid(g: &mut Game, aura: CardId) {
    start(g, aura, 0);
    // A rejected payment after announcement must preserve the entire pending
    // activation, mana and both tap states, before the valid exact payment.
    reject(
        g,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 4],
        },
    );
    reject(
        g,
        Action::PayCosts {
            payment: [0, 0, 0, 0, 0, 6],
        },
    );
    pay(g);
}

#[test]
fn ordinary_grant_only_pays_five_even_with_opposing_tapped_aura_and_sick_tapped_recipient() {
    let _rules = fixture_rules::activate(&[(aura_card(), &NO_TAP)]);
    for p in [Player::P0, Player::P1] {
        for owner in [p, p.other()] {
            for tapped in [false, true] {
                let (mut g, recipient, aura) = setup(p, owner);
                g.state.objects[aura as usize].tapped = tapped;
                paid(&mut g, aura);
                assert_eq!(g.state.players[p.index()].mana, [0; 6]);
                assert_eq!(g.object_view(aura).tapped, tapped);
                let packet = g.state.stack.last().unwrap();
                assert_eq!(packet.controller, p);
                assert_eq!(packet.source().unwrap().object, g.object_ref(recipient));
                settle(&mut g);
                assert!(!g.object_view(recipient).tapped);
                assert_eq!(g.object_view(aura).tapped, tapped);
            }
        }
    }
}
#[test]
fn aura_controller_cannot_activate_the_other_players_creatures_ability_and_cost_failure_rolls_back()
{
    let _rules = fixture_rules::activate(&[(aura_card(), &NO_TAP)]);
    let p = Player::P1;
    let (mut g, _, aura) = setup(p, p.other());
    let action = Action::AnnounceAttachedAbility {
        grantor: g.object_ref(aura),
        ability: 0,
    };
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 4];
    reject(&mut g, action.clone());
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 5];
    act(&mut g, Action::Pass);
    assert_eq!(g.actor(), Some(p.other()));
    reject(&mut g, action);
}
#[test]
fn ordinary_grant_packet_survives_aura_departure_and_keeps_its_original_recipient() {
    let _rules = fixture_rules::activate(&[(aura_card(), &NO_TAP)]);
    let p = Player::P0;
    let (mut g, recipient, aura) = setup(p, p.other());
    paid(&mut g, aura);
    g.move_object(aura, Zone::Graveyard);
    settle(&mut g);
    assert!(!g.object_view(recipient).tapped);
}
#[test]
fn departed_recipient_packet_never_untaps_its_new_incarnation() {
    let _rules = fixture_rules::activate(&[(aura_card(), &NO_TAP)]);
    let p = Player::P0;
    let (mut g, recipient, aura) = setup(p, p.other());
    paid(&mut g, aura);
    let old = g.object_ref(recipient);
    g.move_object(recipient, Zone::Hand);
    g.move_object(recipient, Zone::Battlefield);
    assert_ne!(g.object_ref(recipient), old);
    g.state.objects[recipient as usize].tapped = true;
    settle(&mut g);
    assert!(g.object_view(recipient).tapped);
}
#[test]
fn ordinary_grant_can_fund_itself_by_sacrificing_its_own_attachment_for_mana() {
    let _rules = fixture_rules::activate(&[(aura_card(), &SACRIFICE_FUNDER)]);
    let p = Player::P0;
    let (mut g, recipient, aura) = setup(p, p);
    g.state.players[p.index()].mana = [0; 6];
    start(&mut g, aura, 0);
    choose(&mut g, |a| {
        matches!(a, Action::ActivateMana { card, ability: 1, production, .. }
        if *card == aura && *production == [0, 0, 0, 0, 0, 5])
    });
    assert_eq!(g.object_view(aura).zone, Zone::Graveyard);
    pay(&mut g);
    settle(&mut g);
    assert!(!g.object_view(recipient).tapped);
    assert_eq!(g.state.players[p.index()].mana, [0; 6]);
}
#[test]
fn ordinary_grant_payment_is_owned_raw_index_restorable_and_survives_parent_disposal() {
    let _rules = fixture_rules::activate(&[(aura_card(), &NO_TAP)]);
    let p = Player::P1;
    let (mut parent, recipient, aura) = setup(p, p.other());
    start(&mut parent, aura, 0);
    let action = Action::PayCosts {
        payment: [0, 0, 0, 0, 0, 5],
    };
    let saved = parent.snapshot();
    let revision = parent.revision();
    let index = parent
        .legal_actions()
        .iter()
        .position(|a| *a == action)
        .unwrap();
    let mut raw = parent.clone();
    let mut indexed = parent.clone();
    drop(parent);
    assert_eq!(raw.apply(p.other(), action.clone()), Err(Error::WrongActor));
    assert_eq!(raw.snapshot(), saved);
    raw.apply(p, action.clone()).unwrap();
    indexed.apply_index(p, revision, index).unwrap();
    settle(&mut raw);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
    assert!(!raw.object_view(recipient).tapped);
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut indexed, action);
    settle(&mut indexed);
    assert_eq!(raw.snapshot(), indexed.snapshot());
}
fn pole_setup(p: Player) -> (Game, CardId, CardId) {
    let mut g = scenario(p);
    let recipient = put(&mut g, p, "Bear Cub", Zone::Battlefield);
    let pole = put(&mut g, p, "Fishing Pole", Zone::Battlefield);
    choose(&mut g, |a| {
        matches!(a, Action::Activate {card, ability: 0, target: Some(Target::Card(t)), .. }
        if *card == pole && *t == recipient)
    });
    settle(&mut g);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 1];
    (g, recipient, pole)
}
#[test]
fn actual_fishing_pole_retains_both_distinct_tap_costs_and_one_mana_payment() {
    for p in [Player::P0, Player::P1] {
        let (mut g, recipient, pole) = pole_setup(p);
        start(&mut g, pole, 1);
        for _ in 0..8 {
            if !g.state.stack.is_empty() {
                break;
            }
            match g.state.pending {
                Pending::PayCosts { .. } => pay(&mut g),
                Pending::AdditionalCost { .. } => {
                    let object = g.object_ref(pole);
                    act(&mut g, Action::ChooseAdditionalCost { object });
                }
                Pending::CastCosts { .. } => {
                    choose(&mut g, |a| matches!(a, Action::ChooseCastCost { .. }))
                }
                _ => panic!("unexpected pole costs {:?}", g.state.pending),
            }
        }
        assert_eq!(g.state.stack.len(), 1);
        assert!(g.object_view(recipient).tapped);
        assert!(g.object_view(pole).tapped);
        assert_eq!(g.state.players[p.index()].mana, [0; 6]);
        settle(&mut g);
        assert_eq!(g.counter_count(pole, CounterKind::Bait), 1);
    }
}
#[test]
fn actual_fishing_pole_still_rejects_tapped_or_foreign_attachment_and_sick_recipient() {
    for which in 0..4 {
        let p = Player::P0;
        let (mut g, recipient, pole) = pole_setup(p);
        match which {
            0 => g.state.objects[pole as usize].tapped = true,
            1 => g.change_controller(g.object_ref(pole), p.other()),
            2 => g.state.objects[recipient as usize].tapped = true,
            _ => g.state.objects[recipient as usize].summoning_sick = true,
        }
        let grantor = g.object_ref(pole);
        reject(
            &mut g,
            Action::AnnounceAttachedAbility {
                grantor,
                ability: 1,
            },
        );
    }
}

#[test]
fn explicit_attachment_tap_cost_does_not_invent_a_recipient_tap_cost() {
    const TAP_GIVER: CardRules = CardRules {
        static_groups: &[StaticAbilityGroup {
            line: 0,
            first_layer: AbilityLayer::Ability,
            fragments: &[StaticAbilityFragment::AttachmentActivation {
                ability: 0,
                tap_attachment: true,
            }],
        }],
        ..NO_TAP
    };
    let _rules = fixture_rules::activate(&[(aura_card(), &TAP_GIVER)]);
    let p = Player::P0;
    let (mut g, recipient, aura) = setup(p, p);
    // The template has no tap symbol; its tapped, sick recipient can pay the
    // separately printed attachment cost and becomes untapped on resolution.
    paid(&mut g, aura);
    assert!(g.object_view(aura).tapped);
    assert!(g.object_view(recipient).tapped);
    settle(&mut g);
    assert!(!g.object_view(recipient).tapped);
}
