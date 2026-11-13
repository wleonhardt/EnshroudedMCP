//! Admission and arithmetic bounds for caller-supplied crafting plans.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

pub const MAX_QUANTITY: u64 = 1_000_000_000;
pub const MAX_TOTAL: u64 = 1_000_000_000_000;
const MAX_RECIPES: usize = 128;
const MAX_ITEMS: usize = 512;

#[derive(Debug)]
pub struct Recipe {
    pub id: String,
    pub item: String,
    pub count: u64,
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
    /// Producers precede their ingredients. Every item occurs exactly once.
    pub order: Vec<String>,
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
    json!({"type":"object","additionalProperties":false,
        "properties":{
            "catalog_label":name,
            "recipes":{"type":"array","maxItems":MAX_RECIPES,"items":{
                "type":"object","additionalProperties":false,
                "properties":{
                    "id":name,"item":name,"count":{"type":"integer","minimum":1,"maximum":MAX_QUANTITY},
                    "ingredients":quantities(16),"stations":names(16),"unlocks":names(16)
                },"required":["id","item","count","ingredients","stations","unlocks"]
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
    let mut recipes = BTreeMap::new();
    let mut ids = BTreeSet::new();
    for value in array(&args["recipes"], MAX_RECIPES)? {
        let r = object(
            value,
            &["id", "item", "count", "ingredients", "stations", "unlocks"],
        )?;
        let recipe = Recipe {
            id: name(&r["id"])?,
            item: name(&r["item"])?,
            count: quantity(&r["count"])?,
            ingredients: quantities(&r["ingredients"], 16)?,
            stations: names(&r["stations"], 16)?,
            unlocks: names(&r["unlocks"], 16)?,
        };
        if !ids.insert(recipe.id.clone()) {
            return Err(format!("duplicate recipe id '{}'", recipe.id));
        }
        if recipes.insert(recipe.item.clone(), recipe).is_some() {
            return Err("ambiguous recipe: supply exactly one chosen producer per item".into());
        }
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
) -> Result<Vec<String>, String> {
    let mut indegree: BTreeMap<String, usize> = stock
        .keys()
        .chain(goals.keys())
        .map(|s| (s.clone(), 0))
        .collect();
    for (item, recipe) in recipes {
        indegree.entry(item.clone()).or_default();
        for ingredient in recipe.ingredients.keys() {
            *indegree.entry(ingredient.clone()).or_default() += 1;
        }
    }
    if indegree.len() > MAX_ITEMS {
        return Err(format!(
            "catalogue and stock contain more than {MAX_ITEMS} distinct items"
        ));
    }
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter(|(_, degree)| **degree == 0)
        .map(|(item, _)| item.clone())
        .collect();
    let mut result = Vec::new();
    while let Some(item) = ready.pop_first() {
        if let Some(recipe) = recipes.get(&item) {
            for ingredient in recipe.ingredients.keys() {
                let degree = indegree
                    .get_mut(ingredient)
                    .expect("all ingredients were inserted");
                *degree -= 1;
                if *degree == 0 {
                    ready.insert(ingredient.clone());
                }
            }
        }
        result.push(item);
    }
    if result.len() != indegree.len() {
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

    #[test]
    fn derived_arithmetic_is_bounded() {
        assert_eq!(add(MAX_TOTAL - 1, 1), Ok(MAX_TOTAL));
        assert!(add(MAX_TOTAL, 1).is_err());
        assert!(multiply(MAX_TOTAL, 2).is_err());
        assert!(multiply(u64::MAX, 2).is_err());
    }
}
