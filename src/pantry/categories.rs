// groups pantry items into sections (fruits, vegetables, baking, ...) by matching keywords in the item name

/// Sections in the order they appear on the pantry page. Items that match nothing land in "Other".
const CATEGORIES: &[(&str, &[&str])] = &[
    (
        "Fruits",
        &[
            "apple", "apricot", "avocado", "banana", "berry", "blackberry", "blueberry",
            "cantaloupe", "cherry", "clementine", "coconut", "cranberry", "date", "fig",
            "fruit", "grape", "grapefruit", "kiwi", "lemon", "lime", "mango", "melon",
            "nectarine", "olive", "orange", "papaya", "peach", "pear", "pineapple", "plum",
            "pomegranate", "raisin", "raspberry", "strawberry", "tangerine", "watermelon",
        ],
    ),
    (
        "Vegetables",
        &[
            "artichoke", "arugula", "asparagus", "bell pepper", "beet", "bok choy", "broccoli",
            "brussels sprout", "cabbage", "carrot", "cauliflower", "celery", "chard",
            "cucumber", "eggplant", "garlic", "ginger", "green bean", "green onion", "jalapeno",
            "kale", "leek", "lettuce", "mushroom", "onion", "parsnip", "pea", "potato",
            "pumpkin", "radish", "romaine", "scallion", "shallot", "spinach", "squash",
            "sweet potato", "tomato", "vegetable", "zucchini", "corn", "cilantro", "parsley",
            "basil", "fresh herb",
        ],
    ),
    (
        "Meat & Seafood",
        &[
            "bacon", "beef", "chicken", "chorizo", "cod", "crab", "fish", "ground beef",
            "ground turkey", "ham", "lamb", "lobster", "meat", "pork", "prosciutto",
            "salami", "salmon", "sausage", "scallop", "shrimp", "steak", "tilapia", "tuna",
            "turkey", "pepperoni",
        ],
    ),
    (
        "Dairy & Eggs",
        &[
            "butter", "buttermilk", "cheddar", "cheese", "cottage cheese", "cream",
            "cream cheese", "egg", "feta", "half and half", "heavy cream", "milk",
            "mozzarella", "parmesan", "ricotta", "sour cream", "whipping cream", "yogurt",
        ],
    ),
    (
        "Grains, Pasta & Bread",
        &[
            "bagel", "barley", "bread", "breadcrumb", "bun", "cereal", "couscous", "cracker",
            "farro", "grain", "lasagna", "macaroni", "noodle", "oat", "oatmeal", "panko",
            "pasta", "penne", "quinoa", "rice", "spaghetti", "tortilla", "bulgur",
        ],
    ),
    (
        "Baking & Sweeteners",
        &[
            "agave", "almond extract", "baking powder", "baking soda", "brown sugar",
            "chocolate", "chocolate chip", "cocoa", "cocoa powder", "cornmeal", "cornstarch",
            "extract", "flour", "honey", "maple syrup", "molasses", "powdered sugar",
            "sprinkle", "sugar", "vanilla", "vanilla extract", "yeast", "shortening",
        ],
    ),
    (
        "Canned & Jarred",
        &[
            "bean", "black bean", "broth", "chickpea", "coconut milk", "crushed tomato",
            "diced tomato", "kidney bean", "lentil", "pickle", "pinto bean", "refried bean",
            "soup", "stock", "tomato paste", "tomato sauce", "canned tuna", "salsa",
            "marinara", "pasta sauce", "jam", "jelly", "artichoke heart",
        ],
    ),
    (
        "Condiments & Sauces",
        &[
            "barbecue sauce", "bbq sauce", "dressing", "fish sauce", "hoisin", "hot sauce",
            "ketchup", "mayo", "mayonnaise", "mustard", "peanut butter", "pesto", "relish",
            "sauce", "soy sauce", "sriracha", "tahini", "teriyaki", "worcestershire",
        ],
    ),
    (
        "Oils & Vinegars",
        &[
            "avocado oil", "canola oil", "coconut oil", "cooking spray", "oil", "olive oil",
            "sesame oil", "vegetable oil", "vinegar",
        ],
    ),
    (
        "Spices & Seasonings",
        &[
            "allspice", "bay leaf", "black pepper", "cayenne", "chili flake", "chili powder",
            "cinnamon", "clove", "cumin", "curry", "dried basil", "dried oregano",
            "dry mustard", "garlic powder", "garlic salt", "herb", "italian seasoning",
            "mustard seed", "nutmeg", "onion powder", "oregano", "paprika", "pepper",
            "red pepper flake", "rosemary", "sage", "salt", "seasoning", "spice", "thyme",
            "turmeric", "ground ginger",
        ],
    ),
    (
        "Nuts & Seeds",
        &[
            "almond", "cashew", "chia", "flax", "nut", "peanut", "pecan", "pistachio",
            "pumpkin seed", "seed", "sesame seed", "sunflower seed", "walnut",
        ],
    ),
    (
        "Snacks",
        &[
            "chip", "cookie", "granola", "popcorn", "pretzel", "tortilla chip", "potato chip",
            "snack",
        ],
    ),
    (
        "Frozen",
        &["frozen", "ice cream"],
    ),
    (
        "Beverages",
        &["coffee", "juice", "soda", "tea", "wine", "beer", "sparkling water"],
    ),
];

const OTHER: &str = "Other";

pub struct PantrySection {
    pub name: &'static str,
    pub items: Vec<String>,
}

/// Groups items into sections, keeping each item's original order and skipping empty sections.
pub fn group_items(items: Vec<String>) -> Vec<PantrySection> {
    let mut sections: Vec<PantrySection> = CATEGORIES
        .iter()
        .map(|(name, _)| name)
        .chain(std::iter::once(&OTHER))
        .map(|name| PantrySection { name, items: Vec::new() })
        .collect();

    for item in items {
        let index = category_index(&item).unwrap_or(CATEGORIES.len());
        sections[index].items.push(item);
    }

    sections.retain(|section| !section.items.is_empty());
    sections
}

/// Picks the category whose keyword matches the most words of the item name
/// ("olive oil" beats "olive"); ties go to the earlier category.
/// Items described as canned or frozen go straight to those sections.
fn category_index(item: &str) -> Option<usize> {
    let words: Vec<String> = item
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect();

    let forced = if words.iter().any(|w| w == "canned" || w == "tinned") {
        Some("Canned & Jarred")
    } else if words.iter().any(|w| w == "frozen") {
        Some("Frozen")
    } else {
        None
    };
    if let Some(name) = forced {
        return CATEGORIES.iter().position(|(n, _)| *n == name);
    }

    let mut best: Option<(usize, usize)> = None; // (category index, matched word count)
    for (index, (_, keywords)) in CATEGORIES.iter().enumerate() {
        for keyword in *keywords {
            let keyword_words: Vec<&str> = keyword.split(' ').collect();
            let len = keyword_words.len();
            if best.is_some_and(|(_, best_len)| best_len >= len) {
                continue;
            }
            if words
                .windows(len)
                .any(|window| window.iter().zip(&keyword_words).all(|(w, k)| word_matches(w, k)))
            {
                best = Some((index, len));
            }
        }
    }
    best.map(|(index, _)| index)
}

/// Matches a word against a keyword, allowing simple plurals (tomato/tomatoes, berry/berries).
fn word_matches(word: &str, keyword: &str) -> bool {
    if word == keyword {
        return true;
    }
    if let Some(stem) = word.strip_suffix("ies") {
        if keyword.strip_suffix('y') == Some(stem) {
            return true;
        }
    }
    word.strip_suffix("es") == Some(keyword) || word.strip_suffix('s') == Some(keyword)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category_of(item: &str) -> &'static str {
        category_index(item).map_or(OTHER, |i| CATEGORIES[i].0)
    }

    #[test]
    fn categorizes_common_items() {
        let cases = [
            ("avocado", "Fruits"),
            ("Lemons", "Fruits"),
            ("onions", "Vegetables"),
            ("garlic", "Vegetables"),
            ("red bell peppers", "Vegetables"),
            ("milk", "Dairy & Eggs"),
            ("sour cream", "Dairy & Eggs"),
            ("eggs", "Dairy & Eggs"),
            ("rice", "Grains, Pasta & Bread"),
            ("flour", "Baking & Sweeteners"),
            ("chocolate chips", "Baking & Sweeteners"),
            ("cocoa powder", "Baking & Sweeteners"),
            ("honey", "Baking & Sweeteners"),
            ("diced tomatoes", "Canned & Jarred"),
            ("canned peaches", "Canned & Jarred"),
            ("pickles", "Canned & Jarred"),
            ("coconut milk", "Canned & Jarred"),
            ("ketchup", "Condiments & Sauces"),
            ("dijon mustard", "Condiments & Sauces"),
            ("worchestershire sauce", "Condiments & Sauces"),
            ("olive oil", "Oils & Vinegars"),
            ("all common spices", "Spices & Seasonings"),
            ("garlic powder", "Spices & Seasonings"),
            ("black pepper", "Spices & Seasonings"),
            ("tortilla chips", "Snacks"),
            ("frozen peas", "Frozen"),
            ("mystery item", OTHER),
        ];
        for (item, expected) in cases {
            assert_eq!(category_of(item), expected, "item: {item}");
        }
    }

    #[test]
    fn skips_empty_sections_and_keeps_order() {
        let sections = group_items(vec![
            "apples".into(),
            "bananas".into(),
            "widgets".into(),
        ]);
        let names: Vec<_> = sections.iter().map(|s| s.name).collect();
        assert_eq!(names, ["Fruits", "Other"]);
        assert_eq!(sections[0].items, ["apples", "bananas"]);
    }
}
