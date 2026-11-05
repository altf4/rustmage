//! Recording-only strict compatibility audit for unlinked nonland hand exile.
//! Callers normalize the schema integer; compatible bytes are never rewritten.
use super::*;
use serde_json::Value;

const NEW_KIND: &str = "ExileRevealedNonland";
fn check(value: &Value, context: Option<&str>) -> Result<(), String> {
    match value {
        Value::String(kind)
            if kind == NEW_KIND && (context.is_none() || context == Some("kind")) =>
        {
            Err("unlinked nonland hand choice has no schema 200 representation".into())
        }
        Value::Array(rows) => rows.iter().try_for_each(|row| check(row, context)),
        Value::Object(fields) => {
            if fields.contains_key(NEW_KIND) {
                return Err("new nonland hand-choice variant cannot be reinterpreted".into());
            }
            fields
                .iter()
                .try_for_each(|(key, value)| check(value, Some(key)))
        }
        _ => Ok(()),
    }
}
pub(super) fn project_prior_schema201(input: &str) -> Result<String, String> {
    check(
        &serde_json::from_str(input).map_err(|e| e.to_string())?,
        None,
    )?;
    Ok(input.to_owned())
}
pub(super) fn prior_schema201_state_supported(state: &State) -> bool {
    state
        .linked_choices
        .as_ref()
        .and_then(|choices| choices.hand.as_deref())
        .is_none_or(|hand| hand.kind != HandChoiceKind::ExileRevealedNonland)
}
#[test]
fn prior200_preserves_every_inherited_byte_and_positive_union_kind() {
    let bytes = r#"[{"schema":200,"hand_choice":{"kind":{"ExileRevealedKinds":{"kinds":17}},"actor":"P0","victim":"P1","required":1,"selected":[],"candidates":[{"card":7,"generation":3}],"optional":false},"source_ability_origin":19,"semantic_objects":[{"card":8,"generation":2}]}]"#;
    assert_eq!(project_prior_schema201(bytes).unwrap(), bytes);
    let changed = bytes.replace("generation\":3", "generation\":4");
    assert_ne!(project_prior_schema201(&changed).unwrap(), bytes);
    // A rules name is not a HandChoiceKind discriminant.
    let name = r#"{"name":"ExileRevealedNonland"}"#;
    assert_eq!(project_prior_schema201(name).unwrap(), name);
}
#[test]
fn prior200_refuses_the_new_kind_without_stripping_or_renaming_it() {
    for bytes in [
        r#""ExileRevealedNonland""#,
        r#"{"hand_choice":{"kind":"ExileRevealedNonland"}}"#,
        r#"{"ExileRevealedNonland":null}"#,
        r#"[{"schema":200,"hand_choice":{"kind":"ExileRevealedNonland","required":1}}]"#,
    ] {
        assert!(project_prior_schema201(bytes).is_err(), "{bytes}");
    }
}
#[test]
fn actual_nonland_choice_is_new_only_for_chooser_but_owned_state_cannot_project_for_either_viewer()
{
    use rustmage_core::cards::{self as card_rules, authored::CardRules};
    const RULES: CardRules = CardRules {
        effect: Effect::Spell(
            TargetPlan::Single(TargetSpec::one(TargetRule::Player {
                opponent_only: true,
            })),
            &[Instruction::RevealTargetHandExileNonland],
        ),
        ..CardRules::EMPTY
    };
    let _scope = card_rules::fixture_rules::activate(&[(Card::Shock, &RULES)]);
    let mut game = crate::support::scenario(Player::P0);
    let creature = crate::support::put(&mut game, Player::P1, "Bear Cub", Zone::Hand);
    let land = crate::support::put(&mut game, Player::P1, "Forest", Zone::Hand);
    let private = crate::support::put(&mut game, Player::P0, "Cancel", Zone::Hand);
    let spell = crate::support::put(&mut game, Player::P0, "Shock", Zone::Hand);
    crate::support::cast(&mut game, spell, Some(Target::Player(Player::P1)));
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        crate::support::act(&mut game, Action::Pass);
    }
    assert!(matches!(
        game.state.pending,
        Pending::HandCardChoice { player: Player::P0 }
    ));
    let own = game.observe(Player::P0);
    assert_eq!(own.schema, OBSERVATION_SCHEMA);
    assert_eq!(OBSERVATION_SCHEMA, 201);
    let choice = own.hand_choice.as_ref().unwrap();
    assert_eq!(choice.kind, HandChoiceKind::ExileRevealedNonland);
    assert_eq!(choice.candidates, vec![game.object_ref(creature)]);
    assert_eq!(choice.required, 1);
    assert!(!choice.optional);
    assert!(project_prior_schema201(&serde_json::to_string(&own).unwrap()).is_err());
    assert!(!prior_schema201_state_supported(&game.state));
    let mut opposing = game.observe(Player::P1);
    assert!(opposing.hand_choice.is_none());
    assert!(!opposing.objects.iter().any(|object| object.id == private));
    opposing.schema = 200;
    let bytes = serde_json::to_string(&opposing).unwrap();
    assert_eq!(project_prior_schema201(&bytes).unwrap(), bytes);
    // The inherited-looking opponent frame cannot certify the owned continuation.
    let saved = game.snapshot();
    let mut child = game.clone();
    drop(game);
    child.restore(&saved).unwrap();
    assert!(!prior_schema201_state_supported(&child.state));
    let object = child.object_ref(land);
    crate::support::reject(
        &mut child,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
    let object = child.object_ref(creature);
    crate::support::act(
        &mut child,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
    crate::support::settle(&mut child);
    assert_eq!(child.object_view(creature).zone, Zone::Exile);
    assert!(prior_schema201_state_supported(&child.state));
    let mut resolved = child.observe(Player::P0);
    resolved.schema = 200;
    let bytes = serde_json::to_string(&resolved).unwrap();
    assert_eq!(project_prior_schema201(&bytes).unwrap(), bytes);
}

#[test]
fn existing_positive_union_actual_intimidation_can_still_select_artifact_land() {
    let mut game = crate::support::scenario(Player::P0);
    let land = crate::support::put(&mut game, Player::P1, "Forest", Zone::Hand);
    // Only the real immutable Hand characteristics are exercised; this gated
    // card is never cast or played and no behavior is overlaid.
    game.state.objects[land as usize].card = Card::from_name("Scene of the Crime").unwrap();
    let source = crate::support::put(&mut game, Player::P0, "Intimidation Tactics", Zone::Hand);
    crate::support::cast(&mut game, source, Some(Target::Player(Player::P1)));
    for _ in 0..8 {
        if matches!(game.state.pending, Pending::HandCardChoice { .. }) {
            break;
        }
        assert_eq!(game.state.pending, Pending::Priority);
        crate::support::act(&mut game, Action::Pass);
    }
    let mut observation = game.observe(Player::P0);
    assert_eq!(
        observation.hand_choice.as_ref().unwrap().kind,
        HandChoiceKind::ExileRevealedKinds { kinds: 1 | 16 }
    );
    assert_eq!(
        observation.hand_choice.as_ref().unwrap().candidates,
        vec![game.object_ref(land)]
    );
    assert!(prior_schema201_state_supported(&game.state));
    observation.schema = 200;
    let bytes = serde_json::to_string(&observation).unwrap();
    assert_eq!(project_prior_schema201(&bytes).unwrap(), bytes);
    let object = game.object_ref(land);
    crate::support::act(
        &mut game,
        Action::ChooseHandCard {
            object: Some(object),
        },
    );
    crate::support::settle(&mut game);
    assert_eq!(game.object_view(land).zone, Zone::Exile);
}
