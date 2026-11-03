//! Recording-only compatibility audit for the sparse Tarkir wire additions.
//! Compatible JSON is returned byte-for-byte: old facts and acquisition origins
//! are never stripped to make a changed rule look equivalent to schema 199.
use super::*;
use serde_json::Value;

const NEW_FIELDS: &[&str] = &[
    "token_size",
    "sampled_size",
    "announced_division",
    "distinct_colors",
    "beheld",
    "inserted_phases",
    "turn_callbacks",
    "attack_without_defender",
    "delve_choice",
    "behold_cost",
    "suspend_cards",
    "keep_exiled",
    "end_combat",
    "damage_source_choice",
    "chosen_source_shields",
    "stack_object",
    "stack_objects",
];
const NEW_VARIANTS: &[&str] = &[
    "BeginDelve",
    "ChooseDelver",
    "ChooseColorCounter",
    "ChooseCastBehold",
    "ChooseBeholdCost",
    "CastFreeDuringResolution",
    "AnnounceFreeGraveyardAdditionalCosts",
    "AnnounceGrantedHarmonize",
    "GrantedHarmonize",
    "DecayedAttack",
    "SuspendTick",
    "SuspendCast",
    "NamedMode",
    "NamedModes",
    "DeathCounters",
    "ExileRevealedLinkedOptional",
    "DiscardNonlandReflexive",
    "BeholdOptional",
    "ColorCounter",
    "SacrificeManaValue",
    "GraveyardUnderControllerOnce",
    "CreatureSpells",
    "SpellSubtypeAny",
    "LibraryTop",
    "ChooseDamageSource",
    "ChooseStackDamageSource",
    "ChosenSource",
    "cast_behold",
    "behold_cost",
    "delve",
    "damage_source",
];

fn bounded_number(value: &Value, maximum: u64, label: &str) -> Result<(), String> {
    if value.as_u64().is_some_and(|number| number <= maximum) {
        Ok(())
    } else {
        Err(format!("{label} exceeds the schema 199 domain"))
    }
}
fn check(value: &Value, context: Option<&str>) -> Result<(), String> {
    match value {
        Value::String(text) if NEW_VARIANTS.contains(&text.as_str()) => Err(format!(
            "new Tarkir variant {text} has no schema 199 representation"
        )),
        Value::Array(rows) => rows.iter().try_for_each(|row| check(row, context)),
        Value::Object(fields) => {
            if context == Some("graveyard_cast_permissions")
                && fields.get("kind").is_some_and(|value| value == "Harmonize")
            {
                return Err("granted Harmonize permission is new to schema 200".into());
            }
            if fields
                .get("target")
                .is_some_and(|target| target.get("Stack").is_some())
                && fields.get("object").is_some_and(|object| !object.is_null())
            {
                return Err("a physical stack target incarnation is new to schema 200".into());
            }
            if fields.contains_key("tokens") && fields.contains_key("own_end_step") {
                return Err("controller-end-step token sacrifice is new to schema 200".into());
            }
            if fields.contains_key("exiled") && fields.contains_key("ability_origin") {
                return Err("origin-qualified linked exile is new to schema 200".into());
            }
            if fields
                .get("additional_creature_types")
                .is_some_and(Value::is_boolean)
            {
                return Err("additive animation subtypes are new to schema 200".into());
            }
            if fields.contains_key("remainder")
                && (fields.get("remainder").is_some_and(|v| v == "keep")
                    || fields.get("destination").is_some_and(|v| v == "graveyard"))
            {
                return Err(
                    "new private top-card continuation has no schema 199 representation".into(),
                );
            }
            let allocation_action = fields
                .get("kind")
                .is_some_and(|v| v == "AllocateTargetAmount");
            for (key, value) in fields {
                if NEW_FIELDS.contains(&key.as_str()) || NEW_VARIANTS.contains(&key.as_str()) {
                    return Err(format!(
                        "new Tarkir field {key} has no schema 199 representation"
                    ));
                }
                if matches!(
                    key.as_str(),
                    "keywords"
                        | "source_keywords"
                        | "granted_keywords"
                        | "gained_keywords"
                        | "keyword"
                ) {
                    bounded_number(value, u64::from(u32::MAX), "keyword bits")?;
                }
                if context == Some("endure_choice") && key == "amount" {
                    bounded_number(value, u64::from(u16::MAX), "Endure amount")?;
                }
                if (allocation_action || context == Some("target_allocations")) && key == "amount"
                    || context == Some("target_amount")
                        && matches!(key.as_str(), "remaining" | "minimum" | "maximum")
                {
                    bounded_number(value, u64::from(u32::MAX), "target division")?;
                }
                check(value, Some(key))?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

pub(super) fn project_prior_schema200(input: &str) -> Result<String, String> {
    check(
        &serde_json::from_str(input).map_err(|error| error.to_string())?,
        None,
    )?;
    Ok(input.to_owned())
}
pub(super) fn prior_schema200_actions_supported(actions: &[Action]) -> bool {
    serde_json::to_value(actions).is_ok_and(|value| check(&value, None).is_ok())
}

#[test]
fn prior199_retains_inherited_bytes_and_origin_facts() {
    let inherited = r#"[{"schema":199,"keywords":4294967295,"source_ability_origin":17,"delayed_returns":[{"own_end_step":true}],"copy":{"additional_creature_types":["Warrior"]},"target_allocations":[{"amount":4294967295}]}]"#;
    assert_eq!(project_prior_schema200(inherited).unwrap(), inherited);
    let changed = inherited.replace("source_ability_origin\":17", "source_ability_origin\":18");
    assert_ne!(project_prior_schema200(&changed).unwrap(), inherited);
}

#[test]
fn prior199_refuses_new_semantics_instead_of_erasing_them() {
    for current in [
        r#"{"keywords":4294967296}"#,
        r#"{"endure_choice":{"amount":65536}}"#,
        r#"{"target_allocations":[{"amount":4294967296}]}"#,
        r#"{"target_amount":{"remaining":4294967296}}"#,
        r#"{"kind":"AllocateTargetAmount","amount":4294967296}"#,
        r#"{"tokens":[],"own_end_step":true}"#,
        r#"{"exiled":{"card":1,"generation":0},"ability_origin":17}"#,
        r#"{"additional_creature_types":true}"#,
        r#"{"kind":"ChooseDamageSource","object":{"card":1,"generation":0}}"#,
        r#"{"graveyard_cast_permissions":[{"kind":"Harmonize"}]}"#,
        r#"{"target":{"Stack":1},"object":{"card":1,"generation":0}}"#,
        r#"{"remainder":"keep","destination":"hand"}"#,
        r#"{"remainder":"bottom_any","destination":"graveyard"}"#,
    ] {
        assert!(project_prior_schema200(current).is_err(), "{current}");
    }
    // Even null/empty spellings cannot come from the actual sparse serializer.
    for field in NEW_FIELDS {
        for value in ["null", "[]", "false", "0"] {
            let current = format!(r#"{{"{field}":{value}}}"#);
            assert!(project_prior_schema200(&current).is_err(), "{current}");
        }
    }
}
