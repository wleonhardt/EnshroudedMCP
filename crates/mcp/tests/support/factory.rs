use enshwick::resources::{Constant, Field, Kind, Type};

pub const FACTORY: &str = "keen::ecs::FactoryStation";

// Deliberately synthetic layout, including padding and a different record size.
// Resolve by published field names; never copy offsets from a game binary.
pub fn types() -> Vec<Type<'static>> {
    let f = Field::new;
    let mut result = vec![
        Type::new("keen::bool", Kind::Bool, 1, 1),
        Type::new("keen::uint8", Kind::Uint8, 1, 1),
        Type::new("keen::uint32", Kind::Uint32, 4, 4),
        Type::new("keen::sint64", Kind::Sint64, 8, 8),
        Type::new("keen::ecs::Component", Kind::Struct, 1, 1),
        Type::new("keen::Time", Kind::Struct, 8, 8).with_fields(vec![f(
            "value",
            0,
            "keen::sint64",
        )]),
        Type::new("keen::RecipeInputCategorySelection", Kind::Struct, 8, 4).with_fields(vec![
            f("id", 0, "keen::HashKey32"),
            f("selectedIndex", 4, "keen::uint8"),
        ]),
        Type::new(
            "keen::StaticArray<keen::RecipeInputCategorySelection,4>",
            Kind::StaticArray,
            32,
            4,
        )
        .with_inner("keen::RecipeInputCategorySelection")
        .with_count(4),
        Type::new("keen::RecipeInputCategorySelections", Kind::Struct, 32, 4).with_fields(vec![f(
            "selections",
            0,
            "keen::StaticArray<keen::RecipeInputCategorySelection,4>",
        )]),
        Type::new("keen::ecs::FactoryStationState", Kind::Enum, 1, 1)
            .with_inner("keen::uint8")
            .with_constants(
                [
                    "NoRecipe",
                    "MissingMaterials",
                    "Paused",
                    "Stopped",
                    "Running",
                    "MissingOutputSpace",
                ]
                .into_iter()
                .enumerate()
                .map(|(n, s)| Constant::new(s, n as u64))
                .collect(),
            ),
        Type::new("keen::ecs::FactoryStationRequirement", Kind::Enum, 1, 1)
            .with_inner("keen::uint8")
            .with_constants(
                [
                    "Recipe",
                    "EnergySourceInRange",
                    "EnergyAmount",
                    "InputMaterials",
                    "OutputSpace",
                    "AboveWater",
                ]
                .into_iter()
                .enumerate()
                .map(|(n, s)| Constant::new(s, n as u64))
                .collect(),
            ),
        Type::new(
            "keen::ecs::FactoryStationRequirementMask",
            Kind::Bitmask8,
            1,
            1,
        )
        .with_inner("keen::ecs::FactoryStationRequirement"),
        Type::new(FACTORY, Kind::Struct, 128, 8)
            .with_inner("keen::ecs::Component")
            .with_fields(vec![
                f("state", 0, "keen::ecs::FactoryStationState"),
                f("runningRecipe", 8, "keen::RecipeId"),
                f(
                    "runningRecipeInputCategorySelections",
                    16,
                    "keen::RecipeInputCategorySelections",
                ),
                f("nextRecipe", 52, "keen::RecipeId"),
                f(
                    "nextRecipeInputCategorySelections",
                    56,
                    "keen::RecipeInputCategorySelections",
                ),
                f("recipeStart", 96, "keen::Time"),
                f("recipePauseDuration", 104, "keen::Time"),
                f("useAllAvailableCategoryIngredients", 116, "keen::bool"),
                f("isStoppedByPlayer", 117, "keen::bool"),
                f(
                    "missingRequirements",
                    118,
                    "keen::ecs::FactoryStationRequirementMask",
                ),
            ]),
    ];
    for name in ["keen::HashKey32", "keen::RecipeId"] {
        result.push(Type::new(name, Kind::Struct, 4, 4).with_fields(vec![f(
            "value",
            0,
            "keen::uint32",
        )]));
    }
    result
}
