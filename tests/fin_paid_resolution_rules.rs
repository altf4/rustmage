//! Paid casts from exact public objects during another ability's resolution.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::cards::*;
use rustmage_core::engine::*;
use support::*;

const GRAVE_TARGET: TargetPlan = TargetPlan::Single(TargetSpec::one(TargetRule::Object {
    zone: TargetZone::Graveyard,
    kinds: 4 | 8,
    relation: -1,
    other: false,
    nonland: false,
    tapped: false,
    min_mana: 0,
    max_mana: u32::MAX,
}));
const GRAVE_OFFER: CardRules = CardRules {
    effect: Effect::Creature(2, 2),
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::SelfEnters,
        targets: GRAVE_TARGET,
        program: &[
            Instruction::CastTargetedGraveyardPaid {
                spend_any_mana: true,
                exile_on_graveyard: true,
            },
            Instruction::GainLife(PlayerSet::Controller, 1),
        ],
    }],
    ..CardRules::EMPTY
};
const TOP_OFFER: CardRules = CardRules {
    effect: Effect::Spell(
        TargetPlan::NONE,
        &[
            Instruction::ExileTopMayCastPaidFrom(PlayerSet::Opponent),
            Instruction::IfOfferedCastCommitted { skip: 1 },
            Instruction::CreateTokens {
                token: Card::from_catalogue_id(5302),
                amount: 1,
                tapped: false,
            },
        ],
    ),
    ..CardRules::EMPTY
};

fn bound_trigger(g: &mut Game, p: Player, target: CardId) {
    let provider = put(g, p, "Grizzly Bears", Zone::Hand);
    cast(g, provider, None);
    for _ in 0..12 {
        if g.state.stack.last().is_some_and(|item| {
            matches!(item.kind, StackKind::TriggeredAbility { .. })
                && item
                    .targets
                    .iter()
                    .any(|t| t.target == Target::Card(target))
        }) {
            return;
        }
        choose(g, |action| {
            matches!(
                action,
                Action::Pass | Action::PutTrigger { .. } | Action::FinishTargets
            ) || *action
                == Action::ChooseTarget {
                    target: Target::Card(target),
                }
        });
    }
    panic!("12-action graveyard target boundary");
}

fn paid_offer(g: &mut Game) {
    for _ in 0..8 {
        if matches!(g.state.pending, Pending::ResolutionCasts { .. }) {
            return;
        }
        act(g, Action::Pass);
    }
    panic!("8-action paid resolution boundary");
}

fn cast_action(g: &Game, card: CardId) -> Action {
    Action::CastExiledDuringResolution {
        object: g.object_ref(card),
        face: CastFace::Normal,
    }
}

fn pay_with_discard(g: &mut Game, discard: ObjectRef, island: CardId) {
    for _ in 0..8 {
        if g.state.announcing.is_none() {
            return;
        }
        choose(g, |action| {
            matches!(action, Action::PayCosts { .. })
                || *action == Action::ChooseAdditionalCost { object: discard }
                || *action == Action::TapMana { card: island }
        });
    }
    panic!("8-action paid child payment boundary");
}

#[test]
fn targeted_paid_cast_uses_actual_mana_and_resumes_owned_parent_once() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GRAVE_OFFER)]);
    let p = Player::P1;
    let mut g = scenario(p);
    let child = put(&mut g, p.other(), "Seize the Spoils", Zone::Graveyard);
    let bystander = put(&mut g, p, "Seize the Spoils", Zone::Graveyard);
    let discard = put(&mut g, p, "Forest", Zone::Hand);
    let island = put(&mut g, p, "Island", Zone::Battlefield);
    bound_trigger(&mut g, p, child);
    g.state.players[p.index()].mana = [0, 0, 0, 0, 0, 2];
    paid_offer(&mut g);

    let action = cast_action(&g, child);
    let forbidden = cast_action(&g, bystander);
    reject(&mut g, forbidden);
    let opponent = g.observe(p.other());
    assert!(
        opponent
            .objects
            .iter()
            .any(|o| o.id == child && o.zone == Zone::Graveyard)
    );
    assert!(!opponent.objects.iter().any(|o| o.id == discard));
    assert!(!opponent.actions.contains(&action));
    let discard_ref = g.object_ref(discard);
    let saved = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| a == &action).unwrap();
    let mut raw = g.clone();
    let mut indexed = g.clone();
    let mut declined = g.clone();
    drop(g);
    act(&mut raw, action.clone());
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(raw.snapshot(), indexed.snapshot());
    for branch in [&mut raw, &mut indexed] {
        assert!(branch.state.resolving.is_some());
        pay_with_discard(branch, discard_ref, island);
        assert_eq!(branch.object_view(discard).zone, Zone::Graveyard);
        let packet = branch.state.stack.last().unwrap();
        let cast = packet.cast.unwrap();
        assert_eq!(packet.controller, p);
        assert_eq!(branch.object_view(child).owner, p.other());
        assert_eq!(cast.origin, Zone::Graveyard);
        assert_eq!(cast.method, CastMethod::Normal);
        assert_eq!(cast.payment, [0, 1, 0, 0, 0, 2]);
        assert!(cast.spend_any_mana && cast.exile_on_graveyard);
        assert_eq!(branch.state.players[p.index()].life, 20);
        act(branch, Action::FinishResolutionCasts);
        assert_eq!(branch.state.players[p.index()].life, 21);
        settle(branch);
        assert_eq!(branch.object_view(child).zone, Zone::Exile);
    }
    assert_eq!(raw.snapshot(), indexed.snapshot());
    indexed.restore(&saved).unwrap();
    assert_eq!(
        indexed.apply_index(p, revision, index),
        Err(Error::StaleRevision)
    );
    act(&mut declined, Action::FinishResolutionCasts);
    assert_eq!(declined.object_view(child).zone, Zone::Graveyard);
    assert_eq!(declined.state.players[p.index()].life, 21);
    reject(&mut declined, Action::AnnounceCast { card: child });
}

#[test]
fn targeted_permission_rejects_unfundable_cast_and_old_incarnation() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GRAVE_OFFER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let child = put(&mut g, p, "Seize the Spoils", Zone::Graveyard);
    bound_trigger(&mut g, p, child);
    let action = cast_action(&g, child);
    let mut stale = g.clone();
    stale.move_object(child, Zone::Exile);
    stale.move_object(child, Zone::Graveyard);
    settle(&mut stale);
    assert_eq!(stale.state.players[p.index()].life, 20);
    assert_eq!(stale.object_view(child).zone, Zone::Graveyard);

    paid_offer(&mut g);
    reject(&mut g, action);
    assert_eq!(g.legal_actions(), [Action::FinishResolutionCasts]);
    act(&mut g, Action::FinishResolutionCasts);
    assert_eq!(g.object_view(child).zone, Zone::Graveyard);
    assert_eq!(g.state.players[p.index()].life, 21);
}

#[test]
fn exact_replacement_distinguishes_countering_bounce_and_own_flashback() {
    let _rules = fixture_rules::activate(&[(Card::GrizzlyBears, &GRAVE_OFFER)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let spell = put(&mut g, p, "Think Twice", Zone::Graveyard);
    let negate = put(&mut g, p.other(), "Negate", Zone::Hand);
    let bounce = put(&mut g, p.other(), "Bilbo's Gambit", Zone::Hand);
    bound_trigger(&mut g, p, spell);
    paid_offer(&mut g);

    let mut flashback = g.clone();
    act(&mut flashback, Action::AnnounceFlashback { card: spell });
    pay(&mut flashback);
    assert_eq!(
        flashback.state.stack.last().unwrap().cast.unwrap().method,
        CastMethod::Flashback
    );
    act(&mut flashback, Action::FinishResolutionCasts);
    settle(&mut flashback);
    assert_eq!(flashback.object_view(spell).zone, Zone::Exile);

    let normal = cast_action(&g, spell);
    act(&mut g, normal);
    pay(&mut g);
    act(&mut g, Action::FinishResolutionCasts);
    act(&mut g, Action::Pass);
    let mut countered = g.clone();
    cast(&mut countered, negate, Some(Target::Spell(spell)));
    settle(&mut countered);
    assert_eq!(countered.object_view(spell).zone, Zone::Exile);

    act(&mut g, Action::AnnounceCast { card: bounce });
    choose(&mut g, |action| {
        matches!(action, Action::ChooseGift { beneficiary: None })
    });
    act(
        &mut g,
        Action::ChooseTarget {
            target: Target::Spell(spell),
        },
    );
    if g.legal_actions().contains(&Action::FinishTargets) {
        act(&mut g, Action::FinishTargets);
    }
    pay(&mut g);
    settle(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Hand);
    assert_eq!(g.object_view(spell).owner, p);
}

fn treasure_count(g: &Game, p: Player) -> usize {
    g.observe(p)
        .objects
        .iter()
        .filter(|object| {
            object.zone == Zone::Battlefield
                && object.controller == p
                && object.card.has_subtype("Treasure")
        })
        .count()
}

#[test]
fn foreign_top_offer_keeps_color_and_creates_treasure_only_without_a_cast() {
    let _rules = fixture_rules::activate(&[(Card::Divination, &TOP_OFFER)]);
    let p = Player::P0;
    for (name, red) in [("Shock", true), ("Shock", false), ("Forest", false)] {
        let mut g = scenario(p);
        let top = put(&mut g, p.other(), name, Zone::Library);
        let own = g.top_library_card(p).unwrap();
        let source = put(&mut g, p, "Divination", Zone::Hand);
        cast(&mut g, source, None);
        g.state.players[p.index()].mana = if red {
            [0, 0, 0, 1, 0, 0]
        } else {
            [0, 1, 0, 0, 0, 0]
        };
        paid_offer(&mut g);
        assert_eq!(g.object_view(top).zone, Zone::Exile);
        assert_eq!(g.top_library_card(p), Some(own));
        let action = cast_action(&g, top);
        let mut declined = g.clone();
        act(&mut declined, Action::FinishResolutionCasts);
        assert_eq!(treasure_count(&declined, p), 1);
        assert_eq!(declined.object_view(top).zone, Zone::Exile);
        if red {
            act(&mut g, action);
            act(
                &mut g,
                Action::ChooseTarget {
                    target: Target::Player(p.other()),
                },
            );
            if g.legal_actions().contains(&Action::FinishTargets) {
                act(&mut g, Action::FinishTargets);
            }
            pay(&mut g);
            act(&mut g, Action::FinishResolutionCasts);
            settle(&mut g);
            assert_eq!(g.object_view(top).zone, Zone::Graveyard);
            assert_eq!(g.state.players[p.other().index()].life, 18);
            assert_eq!(treasure_count(&g, p), 0);
        } else {
            reject(&mut g, action);
            reject(&mut g, Action::PlayLand { card: top });
        }
    }
}
