// groups pantry items into sections (fruits, vegetables, baking, ...) by matching keywords in the item name

use crate::i18n::Lang;

/// Sections in the order they appear on the pantry page. Items that match nothing land in "Other".
/// Each entry is (English name, Spanish name, keywords). Keywords are lowercase and unaccented,
/// and cover both English and Spanish item names.
const CATEGORIES: &[(&str, &str, &[&str])] = &[
    (
        "Fruits",
        "Frutas",
        &[
            "apple", "apricot", "avocado", "banana", "berry", "blackberry", "blueberry",
            "cantaloupe", "cherry", "clementine", "coconut", "cranberry", "date", "fig",
            "fruit", "grape", "grapefruit", "kiwi", "lemon", "lime", "mango", "melon",
            "nectarine", "olive", "orange", "papaya", "peach", "pear", "pineapple", "plum",
            "pomegranate", "raisin", "raspberry", "strawberry", "tangerine", "watermelon",
            // Spanish
            "aceituna", "aguacate", "albaricoque", "arandano", "cereza", "ciruela", "coco",
            "datil", "durazno", "frambuesa", "fresa", "fruta", "granada", "higo", "lima",
            "limon", "mandarina", "manzana", "melocoton", "mora", "naranja", "oliva", "pasa",
            "pera", "pina", "platano", "sandia", "toronja", "uva",
        ],
    ),
    (
        "Vegetables",
        "Verduras",
        &[
            "artichoke", "arugula", "asparagus", "bell pepper", "beet", "bok choy", "broccoli",
            "brussels sprout", "cabbage", "carrot", "cauliflower", "celery", "chard",
            "cucumber", "eggplant", "garlic", "ginger", "green bean", "green onion", "jalapeno",
            "kale", "leek", "lettuce", "mushroom", "onion", "parsnip", "pea", "potato",
            "pumpkin", "radish", "romaine", "scallion", "shallot", "spinach", "squash",
            "sweet potato", "tomato", "vegetable", "zucchini", "corn", "cilantro", "parsley",
            "basil", "fresh herb",
            // Spanish
            "ajo", "albahaca", "alcachofa", "apio", "berenjena", "betabel", "brocoli",
            "calabacin", "calabaza", "camote", "cebolla", "cebollin", "champinon", "chicharo",
            "chile", "col", "coliflor", "ejote", "elote", "espinaca", "esparrago", "guisante",
            "hongo", "jengibre", "jitomate", "lechuga", "maiz", "papa", "patata", "pepino",
            "perejil", "pimiento", "rabano", "remolacha", "repollo", "tomate", "verdura",
            "zanahoria",
        ],
    ),
    (
        "Meat & Seafood",
        "Carnes y mariscos",
        &[
            "bacon", "beef", "chicken", "chorizo", "cod", "crab", "fish", "ground beef",
            "ground turkey", "ham", "lamb", "lobster", "meat", "pork", "prosciutto",
            "salami", "salmon", "sausage", "scallop", "shrimp", "steak", "tilapia", "tuna",
            "turkey", "pepperoni",
            // Spanish
            "atun", "bacalao", "bistec", "camaron", "cangrejo", "carne", "carne molida",
            "cerdo", "cordero", "filete", "jamon", "langosta", "pavo", "pescado", "pollo",
            "puerco", "res", "salchicha", "tocino",
        ],
    ),
    (
        "Dairy & Eggs",
        "Lácteos y huevos",
        &[
            "butter", "buttermilk", "cheddar", "cheese", "cottage cheese", "cream",
            "cream cheese", "egg", "feta", "half and half", "heavy cream", "milk",
            "mozzarella", "parmesan", "ricotta", "sour cream", "whipping cream", "yogurt",
            // Spanish
            "crema", "crema agria", "huevo", "leche", "mantequilla", "nata", "queso",
            "queso crema", "requeson", "yogur",
        ],
    ),
    (
        "Grains, Pasta & Bread",
        "Granos, pastas y pan",
        &[
            "bagel", "barley", "bread", "breadcrumb", "bun", "cereal", "couscous", "cracker",
            "farro", "grain", "lasagna", "macaroni", "noodle", "oat", "oatmeal", "panko",
            "pasta", "penne", "quinoa", "rice", "spaghetti", "tortilla", "bulgur",
            // Spanish
            "arroz", "avena", "cebada", "cuscus", "espagueti", "fideo", "macarron", "pan",
            "pan molido", "pan rallado", "quinua", "tallarin",
        ],
    ),
    (
        "Baking & Sweeteners",
        "Repostería y endulzantes",
        &[
            "agave", "almond extract", "baking powder", "baking soda", "brown sugar",
            "chocolate", "chocolate chip", "cocoa", "cocoa powder", "cornmeal", "cornstarch",
            "extract", "flour", "honey", "maple syrup", "molasses", "powdered sugar",
            "sprinkle", "sugar", "vanilla", "vanilla extract", "yeast", "shortening",
            // Spanish
            "azucar", "azucar glas", "azucar morena", "bicarbonato", "cacao", "cacao en polvo",
            "chispas de chocolate", "extracto de vainilla", "fecula de maiz", "harina",
            "jarabe de maple", "levadura", "maicena", "manteca", "miel", "polvo de hornear",
            "polvo para hornear", "vainilla",
        ],
    ),
    (
        "Canned & Jarred",
        "Enlatados y en frasco",
        &[
            "bean", "black bean", "broth", "chickpea", "coconut milk", "crushed tomato",
            "diced tomato", "kidney bean", "lentil", "pickle", "pinto bean", "refried bean",
            "soup", "stock", "tomato paste", "tomato sauce", "canned tuna", "salsa",
            "marinara", "pasta sauce", "jam", "jelly", "artichoke heart",
            // Spanish
            "caldo", "consome", "frijol", "garbanzo", "leche de coco", "lenteja", "mermelada",
            "pasta de tomate", "pepinillo", "pure de tomate", "salsa de tomate", "sopa",
        ],
    ),
    (
        "Condiments & Sauces",
        "Condimentos y salsas",
        &[
            "barbecue sauce", "bbq sauce", "dressing", "fish sauce", "hoisin", "hot sauce",
            "ketchup", "mayo", "mayonnaise", "mustard", "peanut butter", "pesto", "relish",
            "sauce", "soy sauce", "sriracha", "tahini", "teriyaki", "worcestershire",
            // Spanish
            "aderezo", "catsup", "crema de cacahuate", "mantequilla de cacahuate",
            "mantequilla de mani", "mayonesa", "mostaza", "salsa barbacoa", "salsa de soja",
            "salsa de soya", "salsa inglesa", "salsa picante",
        ],
    ),
    (
        "Oils & Vinegars",
        "Aceites y vinagres",
        &[
            "avocado oil", "canola oil", "coconut oil", "cooking spray", "oil", "olive oil",
            "sesame oil", "vegetable oil", "vinegar",
            // Spanish
            "aceite", "aceite de coco", "aceite de oliva", "aceite vegetal", "vinagre",
        ],
    ),
    (
        "Spices & Seasonings",
        "Especias y sazonadores",
        &[
            "allspice", "bay leaf", "black pepper", "cayenne", "chili flake", "chili powder",
            "cinnamon", "clove", "cumin", "curry", "dried basil", "dried oregano",
            "dry mustard", "garlic powder", "garlic salt", "herb", "italian seasoning",
            "mustard seed", "nutmeg", "onion powder", "oregano", "paprika", "pepper",
            "red pepper flake", "rosemary", "sage", "salt", "seasoning", "spice", "thyme",
            "turmeric", "ground ginger",
            // Spanish
            "ajo en polvo", "canela", "cebolla en polvo", "chile en polvo", "clavo", "comino",
            "curcuma", "especia", "hierba", "hoja de laurel", "laurel", "nuez moscada",
            "oregano", "pimenton", "pimienta", "pimienta negra", "romero", "sal", "sazonador",
            "tomillo",
        ],
    ),
    (
        "Nuts & Seeds",
        "Frutos secos y semillas",
        &[
            "almond", "cashew", "chia", "flax", "nut", "peanut", "pecan", "pistachio",
            "pumpkin seed", "seed", "sesame seed", "sunflower seed", "walnut",
            // Spanish
            "ajonjoli", "almendra", "cacahuate", "linaza", "mani", "nuez", "pistache",
            "pistacho", "semilla",
        ],
    ),
    (
        "Snacks",
        "Snacks",
        &[
            "chip", "cookie", "granola", "popcorn", "pretzel", "tortilla chip", "potato chip",
            "snack",
            // Spanish
            "botana", "galleta", "palomita", "papas fritas", "totopo",
        ],
    ),
    (
        "Frozen",
        "Congelados",
        &["frozen", "ice cream", "helado"],
    ),
    (
        "Beverages",
        "Bebidas",
        &[
            "coffee", "juice", "soda", "tea", "wine", "beer", "sparkling water",
            // Spanish
            "agua mineral", "cafe", "cerveza", "jugo", "refresco", "te", "vino",
        ],
    ),
];

const OTHER: (&str, &str) = ("Other", "Otros");

pub struct PantrySection {
    pub name: &'static str,
    pub items: Vec<String>,
}

/// Groups items into sections, keeping each item's original order and skipping empty sections.
pub fn group_items(items: Vec<String>, lang: Lang) -> Vec<PantrySection> {
    let mut sections: Vec<PantrySection> = CATEGORIES
        .iter()
        .map(|(en, es, _)| (*en, *es))
        .chain(std::iter::once(OTHER))
        .map(|(en, es)| PantrySection {
            name: if lang == Lang::Es { es } else { en },
            items: Vec::new(),
        })
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
        .map(normalize)
        .collect();

    let forced = if words
        .iter()
        .any(|w| w == "canned" || w == "tinned" || w.starts_with("enlatad"))
    {
        Some("Canned & Jarred")
    } else if words
        .iter()
        .any(|w| w == "frozen" || w.starts_with("congelad"))
    {
        Some("Frozen")
    } else {
        None
    };
    if let Some(name) = forced {
        return CATEGORIES.iter().position(|(n, _, _)| *n == name);
    }

    let mut best: Option<(usize, usize)> = None; // (category index, matched word count)
    for (index, (_, _, keywords)) in CATEGORIES.iter().enumerate() {
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

/// Lowercases a word and strips Spanish accents so "Limón" matches "limon".
fn normalize(word: &str) -> String {
    word.to_lowercase()
        .chars()
        .map(|c| match c {
            'á' => 'a',
            'é' => 'e',
            'í' => 'i',
            'ó' => 'o',
            'ú' | 'ü' => 'u',
            'ñ' => 'n',
            other => other,
        })
        .collect()
}

/// Matches a word against a keyword, allowing simple plurals
/// (tomato/tomatoes, berry/berries, nuez/nueces).
fn word_matches(word: &str, keyword: &str) -> bool {
    if word == keyword {
        return true;
    }
    if let Some(stem) = word.strip_suffix("ies") {
        if keyword.strip_suffix('y') == Some(stem) {
            return true;
        }
    }
    if let Some(stem) = word.strip_suffix("ces") {
        if keyword.strip_suffix('z') == Some(stem) {
            return true;
        }
    }
    word.strip_suffix("es") == Some(keyword) || word.strip_suffix('s') == Some(keyword)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category_of(item: &str) -> &'static str {
        category_index(item).map_or(OTHER.0, |i| CATEGORIES[i].0)
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
            ("mystery item", OTHER.0),
        ];
        for (item, expected) in cases {
            assert_eq!(category_of(item), expected, "item: {item}");
        }
    }

    #[test]
    fn categorizes_spanish_items() {
        let cases = [
            ("Limones", "Fruits"),
            ("cebollas", "Vegetables"),
            ("chiles jalapeños", "Vegetables"),
            ("pechuga de pollo", "Meat & Seafood"),
            ("leche", "Dairy & Eggs"),
            ("leche de coco", "Canned & Jarred"),
            ("arroz", "Grains, Pasta & Bread"),
            ("azúcar morena", "Baking & Sweeteners"),
            ("polvo para hornear", "Baking & Sweeteners"),
            ("frijoles negros", "Canned & Jarred"),
            ("pasta de tomate", "Canned & Jarred"),
            ("mantequilla de cacahuate", "Condiments & Sauces"),
            ("salsa de soya", "Condiments & Sauces"),
            ("aceite de oliva", "Oils & Vinegars"),
            ("ajo en polvo", "Spices & Seasonings"),
            ("nueces", "Nuts & Seeds"),
            ("nuez moscada", "Spices & Seasonings"),
            ("chícharos congelados", "Frozen"),
            ("café", "Beverages"),
        ];
        for (item, expected) in cases {
            assert_eq!(category_of(item), expected, "item: {item}");
        }
    }

    #[test]
    fn skips_empty_sections_and_keeps_order() {
        let items = || vec!["apples".to_string(), "bananas".into(), "widgets".into()];

        let sections = group_items(items(), Lang::En);
        let names: Vec<_> = sections.iter().map(|s| s.name).collect();
        assert_eq!(names, ["Fruits", "Other"]);
        assert_eq!(sections[0].items, ["apples", "bananas"]);

        let names: Vec<_> = group_items(items(), Lang::Es).iter().map(|s| s.name).collect();
        assert_eq!(names, ["Frutas", "Otros"]);
    }
}
