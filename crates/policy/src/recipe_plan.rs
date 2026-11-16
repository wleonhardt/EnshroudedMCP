//! Admission and arithmetic bounds for caller-supplied crafting plans.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

pub const MAX_QUANTITY: u64 = 1_000_000_000;
pub const MAX_TOTAL: u64 = 1_000_000_000_000;
const MAX_RECIPES: usize = 128;
const MAX_ITEMS: usize = 512;

#[derive(Clone, Debug)]
pub struct Recipe {
    pub id: String,
    pub outputs: BTreeMap<String, u64>,
    pub bundle: bool,
    pub ingredients: BTreeMap<String, u64>,
    pub stations: BTreeSet<String>,
    pub unlocks: BTreeSet<String>,
}

#[derive(Debug)]
pub struct Request {
    pub catalog_label: String,
    pub recipes: BTreeMap<String, Recipe>,
    pub stock: BTreeMap<String, u64>,
    pub goals: BTreeMap<String, u64>,
    pub stations: BTreeSet<String>,
    pub unlocks: BTreeSet<String>,
    /// Producer groups precede their ingredients. Every item occurs once.
    pub order: Vec<Vec<String>>,
}

#[must_use]
pub fn schema() -> Value {
    let name = json!({"type":"string","minLength":1,"maxLength":128});
    let quantities = |max| {
        json!({"type":"array","maxItems":max,"items":{
            "type":"object","additionalProperties":false,
            "properties":{"item":name,"count":{"type":"integer","minimum":1,"maximum":MAX_QUANTITY}},
            "required":["item","count"]
        }})
    };
    let names = |max| json!({"type":"array","maxItems":max,"uniqueItems":true,"items":name});
    let mut outputs = quantities(16);
    outputs["minItems"] = json!(1);
    json!({"type":"object","additionalProperties":false,
        "properties":{
            "catalog_label":name,
            "recipes":{"type":"array","maxItems":MAX_RECIPES,"items":{
                "type":"object","additionalProperties":false,
                "properties":{
                    "id":name,"item":name,"count":{"type":"integer","minimum":1,"maximum":MAX_QUANTITY},
                    "outputs":outputs,
                    "ingredients":quantities(16),"stations":names(16),"unlocks":names(16)
                },"required":["id","ingredients","stations","unlocks"],
                "oneOf":[{"required":["item","count"],"not":{"required":["outputs"]}},
                    {"required":["outputs"],"not":{"anyOf":[{"required":["item"]},{"required":["count"]}]}}]
            }},
            "choices":{"type":"array","maxItems":MAX_ITEMS,"items":{
                "type":"object","additionalProperties":false,
                "properties":{"item":name,"recipe":name},"required":["item","recipe"]
            }},
            "stock":quantities(256),"goals":quantities(32),
            "stations":names(128),"unlocks":names(128)
        },"required":["catalog_label","recipes","stock","goals","stations","unlocks"]
    })
}

/// Strictly validates nested objects as well as the MCP's top-level arguments.
pub fn check(args: &Map<String, Value>) -> Result<Request, String> {
    crate::args::check(&schema(), args)?;
    let catalog_label = name(&args["catalog_label"])?;
    let stock = quantities(&args["stock"], 256)?;
    let goals = quantities(&args["goals"], 32)?;
    if goals.is_empty() {
        return Err("goals must contain at least one item".into());
    }
    let stations = names(&args["stations"], 128)?;
    let unlocks = names(&args["unlocks"], 128)?;
    let mut choices = BTreeMap::new();
    if let Some(value) = args.get("choices") {
        for value in array(value, MAX_ITEMS)? {
            let fields = object(value, &["item", "recipe"])?;
            if choices
                .insert(name(&fields["item"])?, name(&fields["recipe"])?)
                .is_some()
            {
                return Err("duplicate recipe choice for item".into());
            }
        }
    }
    let mut candidates: BTreeMap<String, Vec<Recipe>> = BTreeMap::new();
    let mut ids = BTreeSet::new();
    let mut items: BTreeSet<_> = stock.keys().chain(goals.keys()).cloned().collect();
    for value in array(&args["recipes"], MAX_RECIPES)? {
        let bundle = value.get("outputs").is_some();
        let fields: &[&str] = if bundle {
            &["id", "outputs", "ingredients", "stations", "unlocks"]
        } else {
            &["id", "item", "count", "ingredients", "stations", "unlocks"]
        };
        let r = object(value, fields)?;
        let outputs = if bundle {
            quantities(&r["outputs"], 16)?
        } else {
            BTreeMap::from([(name(&r["item"])?, quantity(&r["count"])?)])
        };
        if outputs.is_empty() {
            return Err("recipe outputs must contain at least one item".into());
        }
        let recipe = Recipe {
            id: name(&r["id"])?,
            outputs,
            bundle,
            ingredients: quantities(&r["ingredients"], 16)?,
            stations: names(&r["stations"], 16)?,
            unlocks: names(&r["unlocks"], 16)?,
        };
        if !ids.insert(recipe.id.clone()) {
            return Err(format!("duplicate recipe id '{}'", recipe.id));
        }
        items.extend(recipe.outputs.keys().cloned());
        items.extend(recipe.ingredients.keys().cloned());
        for item in recipe.outputs.keys() {
            candidates
                .entry(item.clone())
                .or_default()
                .push(recipe.clone());
        }
    }
    if items.len() > MAX_ITEMS {
        return Err(format!(
            "catalogue and stock contain more than {MAX_ITEMS} distinct items"
        ));
    }
    let mut recipes = BTreeMap::new();
    for (item, options) in candidates {
        let choice = choices.remove(&item);
        if options.len() > 1 && choice.is_none() {
            return Err(format!(
                "ambiguous recipe for '{item}': supply an explicit choice"
            ));
        }
        let recipe = options
            .into_iter()
            .find(|r| choice.as_ref().is_none_or(|id| *id == r.id))
            .ok_or_else(|| format!("chosen recipe does not produce '{item}'"))?;
        recipes.insert(item, recipe);
    }
    if !choices.is_empty() {
        return Err("recipe choice names an item without a producer".into());
    }
    let order = order(&recipes, &stock, &goals)?;
    Ok(Request {
        catalog_label,
        recipes,
        stock,
        goals,
        stations,
        unlocks,
        order,
    })
}

fn object<'a>(value: &'a Value, fields: &[&str]) -> Result<&'a Map<String, Value>, String> {
    let object = value.as_object().ok_or("expected an object")?;
    if object.len() != fields.len() || object.keys().any(|key| !fields.contains(&key.as_str())) {
        return Err(format!(
            "expected exactly these fields: {}",
            fields.join(", ")
        ));
    }
    Ok(object)
}

fn array(value: &Value, max: usize) -> Result<&[Value], String> {
    let values = value.as_array().ok_or("expected an array")?;
    if values.len() > max {
        return Err(format!("array must contain at most {max} entries"));
    }
    Ok(values)
}

fn name(value: &Value) -> Result<String, String> {
    let s = value
        .as_str()
        .ok_or("item, recipe, and requirement names must be strings")?;
    if s.is_empty() || s.len() > 128 || s.trim() != s || s.chars().any(char::is_control) {
        return Err("names must be 1..128 UTF-8 bytes, without surrounding whitespace or control characters".into());
    }
    Ok(s.to_owned())
}

fn quantity(value: &Value) -> Result<u64, String> {
    value
        .as_u64()
        .filter(|n| (1..=MAX_QUANTITY).contains(n))
        .ok_or_else(|| format!("quantity must be an integer in 1..={MAX_QUANTITY}"))
}

fn quantities(value: &Value, max: usize) -> Result<BTreeMap<String, u64>, String> {
    let mut result = BTreeMap::new();
    for value in array(value, max)? {
        let fields = object(value, &["item", "count"])?;
        let item = name(&fields["item"])?;
        if result
            .insert(item.clone(), quantity(&fields["count"])?)
            .is_some()
        {
            return Err(format!(
                "duplicate item '{item}'; aggregate its quantity first"
            ));
        }
    }
    Ok(result)
}

fn names(value: &Value, max: usize) -> Result<BTreeSet<String>, String> {
    let mut result = BTreeSet::new();
    for value in array(value, max)? {
        if !result.insert(name(value)?) {
            return Err("duplicate station or unlock name".into());
        }
    }
    Ok(result)
}

fn order(
    recipes: &BTreeMap<String, Recipe>,
    stock: &BTreeMap<String, u64>,
    goals: &BTreeMap<String, u64>,
) -> Result<Vec<Vec<String>>, String> {
    let mut items: BTreeSet<_> = stock.keys().chain(goals.keys()).cloned().collect();
    let mut producer_keys = BTreeMap::new();
    for (item, recipe) in recipes {
        items.insert(item.clone());
        items.extend(recipe.ingredients.keys().cloned());
        producer_keys.entry(&recipe.id).or_insert(item);
    }
    let owners: BTreeMap<_, _> = items
        .iter()
        .map(|item| {
            let key = recipes
                .get(item)
                .map_or(item, |recipe| producer_keys[&recipe.id]);
            (item.clone(), key.clone())
        })
        .collect();
    let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (item, key) in &owners {
        groups.entry(key.clone()).or_default().push(item.clone());
    }
    let mut indegree: BTreeMap<_, usize> = groups.keys().map(|key| (key.clone(), 0)).collect();
    let mut dependencies = BTreeMap::new();
    for key in groups.keys() {
        let inputs: BTreeSet<_> = recipes
            .get(key)
            .into_iter()
            .flat_map(|recipe| recipe.ingredients.keys())
            .map(|item| owners[item].clone())
            .collect();
        for input in &inputs {
            *indegree
                .get_mut(input)
                .expect("all ingredients have owners") += 1;
        }
        dependencies.insert(key.clone(), inputs);
    }
    let mut ready: BTreeSet<_> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(key, _)| key.clone())
        .collect();
    let mut result = Vec::new();
    while let Some(key) = ready.pop_first() {
        for input in &dependencies[&key] {
            let degree = indegree
                .get_mut(input)
                .expect("all ingredients have owners");
            *degree -= 1;
            if *degree == 0 {
                ready.insert(input.clone());
            }
        }
        result.push(groups[&key].clone());
    }
    if result.len() != groups.len() {
        return Err("recipe cycle detected; supply an acyclic set of chosen recipes".into());
    }
    Ok(result)
}

/// Bounds derived quantities even if input values each pass admission.
pub fn add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b)
        .filter(|v| *v <= MAX_TOTAL)
        .ok_or_else(|| "derived quantity exceeds planning limit".into())
}

pub fn multiply(a: u64, b: u64) -> Result<u64, String> {
    a.checked_mul(b)
        .filter(|v| *v <= MAX_TOTAL)
        .ok_or_else(|| "derived quantity exceeds planning limit".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> Value {
        json!({"catalog_label":"fixture", "recipes":[], "stock":[],
            "goals":[{"item":"wood","count":1}],"stations":[],"unlocks":[]})
    }

    fn validate(v: &Value) -> Result<Request, String> {
        check(v.as_object().unwrap())
    }

    #[test]
    fn malformed_nested_fields_and_quantities_are_refused() {
        for bad in [
            json!(0),
            json!(-1),
            json!(1.5),
            json!(MAX_QUANTITY + 1),
            json!("1"),
        ] {
            let mut v = request();
            v["goals"][0]["count"] = bad;
            assert!(validate(&v).is_err());
        }
        let mut v = request();
        v["goals"][0]["extra"] = json!(true);
        assert!(validate(&v).is_err());
        let mut v = request();
        v["goals"][0] = json!({"item":"wood"});
        assert!(validate(&v).is_err());
    }

    #[test]
    fn arrays_names_duplicates_and_unknown_top_level_are_refused() {
        let mut v = request();
        v["goals"] = json!(vec![json!({"item":"wood","count":1}); 33]);
        assert!(validate(&v).is_err());
        for bad in [json!({}), json!("wood"), json!(null)] {
            let mut v = request();
            v["stock"] = bad;
            assert!(validate(&v).is_err());
        }
        for bad in ["", " wood", "wood\n", &"x".repeat(129)] {
            let mut v = request();
            v["goals"][0]["item"] = json!(bad);
            assert!(validate(&v).is_err());
        }
        let mut v = request();
        v["goals"] = json!([{"item":"wood","count":1},{"item":"wood","count":2}]);
        assert!(validate(&v).is_err());
        let mut v = request();
        v["unexpected"] = json!(1);
        assert!(validate(&v).is_err());
    }

    #[test]
    fn ambiguous_producers_cycles_and_duplicate_ids_are_refused() {
        let r = |id, item, input| json!({"id":id,"item":item,"count":1,"ingredients":[{"item":input,"count":1}],"stations":[],"unlocks":[]});
        for recipes in [
            json!([r("a", "x", "z"), r("b", "x", "z")]),
            json!([r("a", "x", "y"), r("b", "y", "x")]),
            json!([r("a", "x", "z"), r("a", "y", "z")]),
        ] {
            let mut v = request();
            v["recipes"] = recipes;
            assert!(validate(&v).is_err());
        }
    }

    fn alternatives() -> Value {
        let mut v = request();
        v["recipes"] = json!([
            {"id":"cycle","item":"wood","count":1,"ingredients":[{"item":"wood","count":1}],"stations":[],"unlocks":[]},
            {"id":"gather","item":"wood","count":1,"ingredients":[],"stations":[],"unlocks":[]}
        ]);
        v["choices"] = json!([{"item":"wood","recipe":"gather"}]);
        v
    }

    #[test]
    fn only_selected_dependencies_participate_in_cycle_detection() {
        let mut v = alternatives();
        assert_eq!(validate(&v).unwrap().recipes["wood"].id, "gather");
        v["choices"][0]["recipe"] = json!("cycle");
        assert!(validate(&v).unwrap_err().contains("cycle"));
        v["choices"] = json!([]);
        assert!(validate(&v).unwrap_err().contains("ambiguous"));
    }

    #[test]
    fn choices_refuse_unknown_mismatched_duplicate_or_malformed_entries() {
        for choices in [
            json!([{"item":"wood","recipe":"absent"}]),
            json!([{"item":"stone","recipe":"gather"}]),
            json!([{"item":"wood","recipe":"gather"},{"item":"wood","recipe":"cycle"}]),
            json!([{"item":"wood","recipe":"gather","dry":true}]),
            json!([{"item":"wood"}]),
            json!([{"item":"wood","recipe":0}]),
            json!([{"item":"wood","recipe":" gather"}]),
            json!(vec![json!({"item":"wood","recipe":"gather"}); 129]),
            json!(null),
        ] {
            let mut v = alternatives();
            v["choices"] = choices;
            assert!(validate(&v).is_err());
        }
        let mut v = alternatives();
        v["recipes"][0]["id"] = json!("gather");
        assert!(validate(&v).unwrap_err().contains("duplicate recipe id"));
        let mut v = alternatives();
        v["recipes"][0]["item"] = json!("stone");
        v["choices"][0]["recipe"] = json!("cycle");
        assert!(validate(&v).unwrap_err().contains("does not produce"));
    }

    #[test]
    fn unselected_recipes_still_obey_catalogue_bounds_and_validation() {
        let mut v = alternatives();
        v["recipes"][0]["ingredients"][0]["count"] = json!(0);
        assert!(validate(&v).is_err());
        let mut v = alternatives();
        v["recipes"] = json!((0..33).map(|i| json!({"id":format!("r{i}"),"item":"wood","count":1,
            "ingredients":(0..16).map(|j| json!({"item":format!("input-{i}-{j}"),"count":1})).collect::<Vec<_>>(),
            "stations":[],"unlocks":[]})).collect::<Vec<_>>());
        v["choices"][0]["recipe"] = json!("r0");
        assert!(validate(&v).unwrap_err().contains("512 distinct items"));
    }

    #[test]
    fn optional_choices_preserve_single_producer_requests() {
        let mut v = alternatives();
        v["recipes"] = json!([v["recipes"][1].clone()]);
        let explicit = validate(&v).unwrap();
        v.as_object_mut().unwrap().remove("choices");
        let implicit = validate(&v).unwrap();
        assert_eq!(explicit.order, implicit.order);
        assert_eq!(explicit.recipes["wood"].id, implicit.recipes["wood"].id);
        v["choices"] = json!([]);
        assert_eq!(validate(&v).unwrap().order, implicit.order);
    }

    #[test]
    fn derived_arithmetic_is_bounded() {
        assert_eq!(add(MAX_TOTAL - 1, 1), Ok(MAX_TOTAL));
        assert!(add(MAX_TOTAL, 1).is_err());
        assert!(multiply(MAX_TOTAL, 2).is_err());
        assert!(multiply(u64::MAX, 2).is_err());
    }

    fn bundled() -> Value {
        let mut v = request();
        v["recipes"] = json!([{"id":"split","outputs":[{"item":"wood","count":2},{"item":"scrap","count":1}],
            "ingredients":[],"stations":[],"unlocks":[]}]);
        v
    }

    #[test]
    fn bundle_outputs_are_strict_nonempty_unique_and_bounded() {
        for outputs in [
            json!([]),
            json!(null),
            json!([{"item":"x","count":0}]),
            json!([{"item":"x","count":-1}]),
            json!([{"item":"x","count":1,"extra":true}]),
            json!([{"item":"x","count":1},{"item":"x","count":2}]),
            json!(
                (0..17)
                    .map(|i| json!({"item":format!("x{i}"),"count":1}))
                    .collect::<Vec<_>>()
            ),
        ] {
            let mut v = bundled();
            v["recipes"][0]["outputs"] = outputs;
            assert!(validate(&v).is_err());
        }
        for (field, value) in [
            ("item", json!("wood")),
            ("count", json!(1)),
            ("extra", json!(false)),
        ] {
            let mut v = bundled();
            v["recipes"][0][field] = value;
            assert!(validate(&v).is_err());
        }
        let mut v = bundled();
        v["recipes"][0]["ingredients"] = json!([{"item":"scrap","count":1}]);
        assert!(validate(&v).unwrap_err().contains("cycle"));
    }

    #[test]
    fn all_bundle_outputs_count_toward_catalogue_bounds_and_choices() {
        let mut v = bundled();
        v["recipes"] = json!((0..33).map(|i|json!({"id":format!("r{i}"),
            "outputs":(0..16).map(|j|json!({"item":format!("out-{i}-{j}"),"count":1})).collect::<Vec<_>>(),
            "ingredients":[],"stations":[],"unlocks":[]})).collect::<Vec<_>>());
        assert!(validate(&v).unwrap_err().contains("512 distinct items"));
        let mut v = bundled();
        v["recipes"].as_array_mut().unwrap().push(json!({"id":"other","item":"scrap","count":1,"ingredients":[],"stations":[],"unlocks":[]}));
        assert!(validate(&v).unwrap_err().contains("ambiguous"));
        v["choices"] = json!([{"item":"scrap","recipe":"other"}]);
        let valid = validate(&v).unwrap();
        assert_eq!(valid.recipes["wood"].id, "split");
        assert_eq!(valid.recipes["scrap"].id, "other");
    }

    #[test]
    fn bundle_choices_cover_more_than_128_items_and_exclude_unselected_cycles() {
        let mut v = bundled();
        let mut recipes = Vec::new();
        let mut choices = Vec::new();
        for i in 0..9 {
            let outputs: Vec<_> = (0..16)
                .map(|j| json!({"item":format!("out-{i}-{j}"),"count":1}))
                .collect();
            for branch in 0..2 {
                recipes.push(json!({"id":format!("r{i}-{branch}"),"outputs":outputs,"ingredients":[],"stations":[],"unlocks":[]}));
            }
            choices.extend(
                (0..16).map(|j| json!({"item":format!("out-{i}-{j}"),"recipe":format!("r{i}-0")})),
            );
        }
        v["recipes"] = json!(recipes);
        v["choices"] = json!(choices);
        assert!(validate(&v).is_ok());
        let mut v = bundled();
        v["recipes"][0]["ingredients"] = json!([{"item":"scrap","count":1}]);
        for item in ["wood", "scrap"] {
            v["recipes"].as_array_mut().unwrap().push(json!({"id":item,"item":item,"count":1,"ingredients":[],"stations":[],"unlocks":[]}));
        }
        v["choices"] = json!([{"item":"wood","recipe":"wood"},{"item":"scrap","recipe":"scrap"}]);
        assert!(validate(&v).is_ok());
        v["choices"][0]["recipe"] = json!("split");
        assert!(validate(&v).is_ok());
        v["choices"][1]["recipe"] = json!("split");
        assert!(validate(&v).unwrap_err().contains("cycle"));
    }
}
