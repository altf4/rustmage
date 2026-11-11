//! Exact stack grants, replacement selection, and one-shot rebound casts.
mod support;
use rustmage as _;
use rustmage_core::cards::authored::CardRules;
use rustmage_core::{cards::*, engine::*};
use support::*;

fn card(name: &str) -> Card {
    Card::from_name(name).unwrap()
}
const REBOUND: CardRules = CardRules {
    effect: Effect::Permanent,
    triggers: &[TriggeredAbility {
        condition: TriggerCondition::CastFromHand(&TriggerCondition::SpellCast {
            relation: CastRelation::You,
            filter: CastFilter::Instant,
            ordinal: None,
        }),
        targets: TargetPlan::NONE,
        program: &[Instruction::GrantCastRebound],
    }],
    ..CardRules::EMPTY
};

fn ready(g: &mut Game, goal: impl Fn(&Game) -> bool) {
    for _ in 0..48 {
        if goal(g) {
            return;
        }
        choose(g, |action| {
            matches!(
                action,
                Action::Pass
                    | Action::PutTrigger { .. }
                    | Action::PayCosts { .. }
                    | Action::FinishTargets
            )
        });
    }
    panic!("rebound scenario did not reach its next decision");
}
fn quiet(g: &mut Game) {
    ready(g, |g| {
        g.state.pending == Pending::Priority
            && g.state.stack.is_empty()
            && g.state.pending_triggers.is_empty()
            && g.state.resolving.is_none()
    });
}
fn upkeep(g: &mut Game, active: Player) {
    g.state.active = active;
    g.begin_step(Step::Upkeep);
    g.give_priority();
    g.prepare_triggers();
}
fn granted(g: &Game, id: CardId) -> bool {
    g.state.stack.iter().any(|item| {
        item.kind == StackKind::Spell
            && item.source.is_some_and(|source| source.id == id)
            && g.spell_has_rebound(item)
    })
}
fn count(g: &Game) -> usize {
    g.state.linked_choices.as_ref().map_or(0, |state| {
        state
            .delayed_steps
            .iter()
            .filter(|row| row.item.kind == StackKind::ReboundCast)
            .count()
    })
}
fn finish_offer(g: &mut Game) {
    if g.legal_actions().contains(&Action::FinishResolutionCasts) {
        act(g, Action::FinishResolutionCasts);
    }
    quiet(g);
}

#[test]
fn redundant_grants_give_one_exact_own_upkeep_offer_after_the_grantors_leave() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &REBOUND)]);
    let p = Player::P0;
    let mut g = scenario(p);
    let sources = [
        put(&mut g, p, "Short Bow", Zone::Battlefield),
        put(&mut g, p, "Short Bow", Zone::Battlefield),
    ];
    let spell = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    ready(&mut g, |g| granted(g, spell));
    assert!(
        g.observe(p.other())
            .stack
            .iter()
            .any(|item| item.kind == StackKind::Spell
                && item.source_keywords.contains(Keywords::REBOUND))
    );
    for id in sources {
        g.move_object(id, Zone::Graveyard);
    }
    quiet(&mut g);
    assert_eq!(count(&g), 1);
    assert_eq!(g.object_view(spell).zone, Zone::Exile);
    assert_eq!(g.state.players[p.other().index()].life, 18);
    let exiled = g.object_ref(spell);

    upkeep(&mut g, p.other());
    quiet(&mut g);
    assert_eq!(count(&g), 1);
    let mut stale = g.clone();
    stale.move_object(spell, Zone::Graveyard);
    stale.move_object(spell, Zone::Exile);
    upkeep(&mut stale, p);
    quiet(&mut stale);
    assert_ne!(stale.object_ref(spell), exiled);
    assert_eq!(count(&stale), 0);
    assert_eq!(stale.object_view(spell).zone, Zone::Exile);

    upkeep(&mut g, p);
    ready(
        &mut g,
        |g| matches!(g.state.pending, Pending::ResolutionCasts { player } if player == p),
    );
    let action = Action::CastExiledDuringResolution {
        object: exiled,
        face: CastFace::Normal,
    };
    let saved = g.snapshot();
    let revision = g.revision();
    let index = g.legal_actions().iter().position(|a| *a == action).unwrap();
    let mut indexed = g.clone();
    let mut decline = g.clone();
    act(&mut g, action);
    indexed.apply_index(p, revision, index).unwrap();
    assert_eq!(g.snapshot(), indexed.snapshot());
    for branch in [&mut g, &mut indexed] {
        act(
            branch,
            Action::ChooseTarget {
                target: Target::Player(p.other()),
            },
        );
        if branch.legal_actions().contains(&Action::FinishTargets) {
            act(branch, Action::FinishTargets);
        }
        if matches!(branch.state.pending, Pending::PayCosts { .. }) {
            pay(branch);
        }
        finish_offer(branch);
    }
    assert_eq!(g.snapshot(), indexed.snapshot());
    assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
    assert_eq!(count(&g), 0);
    assert_eq!(g.state.players[p.other().index()].life, 16);
    decline.restore(&saved).unwrap();
    act(&mut decline, Action::FinishResolutionCasts);
    upkeep(&mut decline, p);
    quiet(&mut decline);
    assert_eq!(count(&decline), 0);
    assert_eq!(decline.object_view(spell).zone, Zone::Exile);
    assert_eq!(decline.state.players[p.other().index()].life, 18);
}

#[test]
fn countered_and_fully_illegal_spells_never_install_a_delayed_cast() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &REBOUND)]);
    let p = Player::P0;
    for case in [0, 1, 2] {
        let mut g = scenario(p);
        put(&mut g, p, "Short Bow", Zone::Battlefield);
        let spell = put(&mut g, p, "Shock", Zone::Hand);
        let victim = put(&mut g, p.other(), "Bear Cub", Zone::Battlefield);
        let counter = put(&mut g, p.other(), "Negate", Zone::Hand);
        cast(&mut g, spell, Some(Target::Card(victim)));
        if case != 0 {
            ready(&mut g, |g| granted(g, spell));
        }
        if case == 2 {
            g.move_object(victim, Zone::Hand);
        } else {
            ready(&mut g, |g| {
                g.state.pending == Pending::Priority && g.actor() == Some(p.other())
            });
            cast(&mut g, counter, Some(Target::Spell(spell)));
        }
        quiet(&mut g);
        assert_eq!(g.object_view(spell).zone, Zone::Graveyard);
        assert_eq!(count(&g), 0);
    }
}

#[test]
fn competing_exile_replacement_only_schedules_rebound_when_chosen() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &REBOUND)]);
    let p = Player::P0;
    let mut g = scenario(p);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    put(&mut g, p.other(), "Rest in Peace", Zone::Battlefield);
    let spell = put(&mut g, p, "Shock", Zone::Hand);
    cast(&mut g, spell, Some(Target::Player(p.other())));
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::ZoneReplacement { .. })
    });
    let options = g.observe(p).zone_replacement_choice.unwrap().choices;
    assert!(options.contains(&ZoneReplacementKind::Rebound));
    assert!(options.contains(&ZoneReplacementKind::Exile));
    assert!(g.observe(p.other()).zone_replacement_choice.is_none());
    for effect in [ZoneReplacementKind::Rebound, ZoneReplacementKind::Exile] {
        let mut branch = g.clone();
        act(&mut branch, Action::ChooseZoneReplacement { effect });
        quiet(&mut branch);
        assert_eq!(branch.object_view(spell).zone, Zone::Exile);
        assert_eq!(
            count(&branch),
            usize::from(effect == ZoneReplacementKind::Rebound)
        );
    }
}

#[test]
fn choosing_rebound_over_adventure_can_cast_the_permanent_face_at_upkeep() {
    let _rules = fixture_rules::activate(&[(card("Short Bow"), &REBOUND)]);
    let p = Player::P1;
    let mut g = scenario(p);
    put(&mut g, p, "Short Bow", Zone::Battlefield);
    let spell = put(
        &mut g,
        p,
        "Heartflame Duelist // Heartflame Slash",
        Zone::Hand,
    );
    act(&mut g, Action::AnnounceAdventure { card: spell });
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
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::ZoneReplacement { .. })
    });
    let mut adventure = g.clone();
    act(
        &mut adventure,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::AdventureExile,
        },
    );
    quiet(&mut adventure);
    assert_eq!(count(&adventure), 0);
    assert!(adventure.can_play_adventure(p, spell));

    act(
        &mut g,
        Action::ChooseZoneReplacement {
            effect: ZoneReplacementKind::Rebound,
        },
    );
    quiet(&mut g);
    assert_eq!(count(&g), 1);
    assert!(!g.can_play_adventure(p, spell));
    upkeep(&mut g, p);
    ready(&mut g, |g| {
        matches!(g.state.pending, Pending::ResolutionCasts { .. })
    });
    let object = g.object_ref(spell);
    act(
        &mut g,
        Action::CastExiledDuringResolution {
            object,
            face: CastFace::Normal,
        },
    );
    if matches!(g.state.pending, Pending::PayCosts { .. }) {
        pay(&mut g);
    }
    finish_offer(&mut g);
    assert_eq!(g.object_view(spell).zone, Zone::Battlefield);
    assert!(!g.object_view(spell).keywords.contains(Keywords::REBOUND));
}
