// interface translations (English / Spanish) and the route that switches between them

use axum::{
    extract::Path,
    http::{header::REFERER, HeaderMap},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};
use axum_extra::extract::{cookie::Cookie, PrivateCookieJar};

use crate::{llm::LlmError, state::AppState};

const LANG_COOKIE: &str = "lang";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    En,
    Es,
}

impl Lang {
    pub fn from_jar(jar: &PrivateCookieJar) -> Self {
        match jar.get(LANG_COOKIE).as_ref().map(Cookie::value) {
            Some("es") => Lang::Es,
            _ => Lang::En,
        }
    }

    pub fn t(self) -> &'static Strings {
        match self {
            Lang::En => &EN,
            Lang::Es => &ES,
        }
    }
}

pub fn router() -> axum::Router<AppState> {
    Router::new().route("/lang/:code", get(set_language))
}

/// Saves the chosen language and sends the user back to the page they were on.
async fn set_language(
    Path(code): Path<String>,
    jar: PrivateCookieJar,
    headers: HeaderMap,
) -> Response {
    let code = if code == "es" { "es" } else { "en" };
    let jar = jar.add(
        Cookie::build((LANG_COOKIE, code))
            .path("/")
            .permanent()
            .build(),
    );

    // Only the path of the referring page is kept, so this can't redirect off-site.
    let back = headers
        .get(REFERER)
        .and_then(|value| value.to_str().ok())
        .and_then(|referer| reqwest::Url::parse(referer).ok())
        .map(|url| match url.query() {
            Some(query) => format!("{}?{}", url.path(), query),
            None => url.path().to_string(),
        })
        .filter(|path| path.starts_with('/') && !path.starts_with("//"))
        .unwrap_or_else(|| "/".to_string());

    (jar, Redirect::to(&back)).into_response()
}

pub struct Strings {
    pub lang: Lang,
    pub html_lang: &'static str,
    /// Language name sent to Gemini so recipes come back in the user's language.
    pub recipe_language: &'static str,
    pub switch_code: &'static str,
    pub switch_label: &'static str,
    pub loading: &'static str,

    pub nav_pantry: &'static str,
    pub nav_generate: &'static str,
    pub nav_history: &'static str,
    pub log_out: &'static str,

    pub landing_title: &'static str,
    pub landing_eyebrow: &'static str,
    pub landing_headline: &'static str,
    pub landing_sub: &'static str,
    pub get_started: &'static str,
    pub invite_note: &'static str,
    pub sample_pantry: &'static str,
    pub sample_chips: &'static [&'static str],
    pub sample_recipe_meta: &'static str,
    pub sample_recipe_title: &'static str,
    pub sample_recipe_note: &'static str,
    pub how_title: &'static str,
    pub steps_how: &'static [(&'static str, &'static str)],
    pub closing_line: &'static str,

    pub pantry_title: &'static str,
    pub pantry_heading: &'static str,
    pub pantry_intro: &'static str,
    pub generate_recipe: &'static str,
    pub add_staple: &'static str,
    pub add_placeholder: &'static str,
    pub add_button: &'static str,
    pub pantry_empty: &'static str,
    pub remove: &'static str,

    pub generate_heading: &'static str,
    /// Trusted static HTML (contains a <code> tag).
    pub gemini_missing_html: &'static str,
    pub meal_type_label: &'static str,
    pub must_use_label: &'static str,
    pub must_use_hint: &'static str,
    pub must_use_placeholder: &'static str,
    pub notes_label: &'static str,
    pub optional_hint: &'static str,
    pub notes_placeholder: &'static str,

    pub history_title: &'static str,
    pub history_intro: &'static str,
    pub history_empty: &'static str,
    pub history_empty_link: &'static str,

    pub login_title: &'static str,
    pub register_title: &'static str,
    pub create_account_heading: &'static str,
    pub welcome_back: &'static str,
    pub auth_intro: &'static str,
    pub username: &'static str,
    pub password: &'static str,
    pub invite_code: &'static str,
    pub register_button: &'static str,
    pub login_button: &'static str,
    pub have_account: &'static str,
    pub new_here: &'static str,
    pub create_account_link: &'static str,

    pub serves: &'static str,
    pub prep: &'static str,
    pub cook: &'static str,
    pub minutes: &'static str,
    pub generate_another: &'static str,
    pub ingredients: &'static str,
    pub steps: &'static str,
    pub from_pantry: &'static str,
    pub no_pantry_called_out: &'static str,
    pub optional_shopping: &'static str,
    pub have_everything: &'static str,
}

impl Strings {
    pub fn pantry_count(&self, count: &usize) -> String {
        match (self.lang, *count) {
            (Lang::En, 1) => "Using 1 pantry staple from your list.".to_string(),
            (Lang::En, n) => format!("Using {n} pantry staples from your list."),
            (Lang::Es, 1) => "Usando 1 producto básico de tu lista.".to_string(),
            (Lang::Es, n) => format!("Usando {n} productos básicos de tu lista."),
        }
    }

    pub fn meal_type<'a>(&self, code: &'a str) -> &'a str {
        let (en, es) = match code {
            "breakfast" => ("Breakfast", "Desayuno"),
            "lunch" => ("Lunch", "Almuerzo"),
            "dinner" => ("Dinner", "Cena"),
            "snack" => ("Snack", "Merienda"),
            "dessert" => ("Dessert", "Postre"),
            other => return other,
        };
        match self.lang {
            Lang::En => en,
            Lang::Es => es,
        }
    }

    /// Translates an error code passed through a `?error=` query string.
    /// Unknown values (e.g. already-translated Gemini errors) are shown as-is.
    pub fn error<'a>(&self, code: &'a str) -> &'a str {
        let (en, es) = match code {
            "missing_credentials" => (
                "Username and password are required",
                "Se requieren usuario y contraseña",
            ),
            "invalid_credentials" => (
                "Invalid username or password",
                "Usuario o contraseña incorrectos",
            ),
            "username_too_short" => (
                "Username must be at least 3 characters",
                "El usuario debe tener al menos 3 caracteres",
            ),
            "password_too_short" => (
                "Password must be at least 6 characters",
                "La contraseña debe tener al menos 6 caracteres",
            ),
            "username_taken" => (
                "That username is already taken",
                "Ese nombre de usuario ya está en uso",
            ),
            "invalid_invite" => (
                "That invite code isn't right. Ask the person who shared the app with you.",
                "Ese código de invitación no es correcto. Pídeselo a quien te compartió la aplicación.",
            ),
            "gemini_not_configured" => (
                "GEMINI_API_KEY is not configured",
                "GEMINI_API_KEY no está configurada",
            ),
            other => return other,
        };
        match self.lang {
            Lang::En => en,
            Lang::Es => es,
        }
    }

    pub fn llm_error(&self, error: &LlmError) -> String {
        match self.lang {
            Lang::En => error.to_string(),
            Lang::Es => match error {
                LlmError::QuotaExceeded => "Se agotó la cuota de Gemini. Se terminó la asignación gratuita de este modelo. Inténtalo más tarde o cambia a otro modelo.".to_string(),
                LlmError::BadResponse => "Gemini devolvió una respuesta inesperada. Inténtalo de nuevo.".to_string(),
                LlmError::InvalidJson => "No se pudo leer la receta de Gemini. Inténtalo de nuevo.".to_string(),
                LlmError::Network(e) => format!("Error de red al comunicarse con Gemini: {e}"),
                LlmError::Api(message) => format!("Error de Gemini: {message}"),
            },
        }
    }
}

static EN: Strings = Strings {
    lang: Lang::En,
    html_lang: "en",
    recipe_language: "English",
    switch_code: "es",
    switch_label: "Español",
    loading: "Thinking... this may take a minute",

    nav_pantry: "Pantry",
    nav_generate: "Generate",
    nav_history: "History",
    log_out: "Log out",

    landing_title: "Cook with what you have",
    landing_eyebrow: "Your kitchen, your recipes",
    landing_headline: "Dinner ideas from the food already in your kitchen.",
    landing_sub: "Pantry Chef keeps track of the staples you always have on hand and turns them into practical recipes, so you shop less and waste less.",
    get_started: "Get started",
    invite_note: "Invite only: you'll need a code from whoever shared Pantry Chef with you.",
    sample_pantry: "In your pantry",
    sample_chips: &["Olive oil", "Garlic", "Rice", "Lemons", "Eggs", "Spinach"],
    sample_recipe_meta: "Dinner · 25 min",
    sample_recipe_title: "Lemon garlic fried rice",
    sample_recipe_note: "Uses 6 of your staples. Nothing to buy.",
    how_title: "How it works",
    steps_how: &[
        ("Stock your pantry", "Add the basics you always keep around. They're sorted into sections like fruits, baking and spices."),
        ("Pick a meal", "Choose breakfast, dinner or dessert, and add any ingredient you want to use up or a note like \"30 minutes max\"."),
        ("Get a recipe", "A full recipe with ingredients, steps and anything worth buying, saved to your history for next time."),
    ],
    closing_line: "Free for friends and family. Available in English and Spanish.",

    pantry_title: "Pantry",
    pantry_heading: "Your pantry",
    pantry_intro: "Staples you always have on hand. Recipes will prefer these ingredients.",
    generate_recipe: "Generate recipe",
    add_staple: "Add staple",
    add_placeholder: "e.g. olive oil, garlic, rice",
    add_button: "Add",
    pantry_empty: "No staples yet. Add a few basics you always keep stocked.",
    remove: "Remove",

    generate_heading: "Generate a recipe",
    gemini_missing_html: "GEMINI_API_KEY is not configured. Add it to your <code>.env</code> file and restart the app.",
    meal_type_label: "Meal type",
    must_use_label: "Must-use ingredients",
    must_use_hint: "(optional, comma-separated)",
    must_use_placeholder: "chicken thighs, spinach",
    notes_label: "Notes",
    optional_hint: "(optional)",
    notes_placeholder: "30 minutes max, no dairy, kid-friendly",

    history_title: "Recipe history",
    history_intro: "Recipes you've generated before.",
    history_empty: "No recipes yet.",
    history_empty_link: "Generate your first one",

    login_title: "Login",
    register_title: "Register",
    create_account_heading: "Create account",
    welcome_back: "Welcome back",
    auth_intro: "Save your pantry staples and generate recipes from what you have on hand.",
    username: "Username",
    password: "Password",
    invite_code: "Invite code",
    register_button: "Register",
    login_button: "Log in",
    have_account: "Already have an account?",
    new_here: "New here?",
    create_account_link: "Create an account",

    serves: "Serves",
    prep: "Prep",
    cook: "Cook",
    minutes: "min",
    generate_another: "Generate another",
    ingredients: "Ingredients",
    steps: "Steps",
    from_pantry: "From your pantry",
    no_pantry_called_out: "No pantry staples called out explicitly.",
    optional_shopping: "Optional shopping",
    have_everything: "You should have everything you need.",
};

static ES: Strings = Strings {
    lang: Lang::Es,
    html_lang: "es",
    recipe_language: "Spanish",
    switch_code: "en",
    switch_label: "English",
    loading: "Pensando... esto puede tardar un minuto",

    nav_pantry: "Despensa",
    nav_generate: "Generar",
    nav_history: "Historial",
    log_out: "Cerrar sesión",

    landing_title: "Cocina con lo que tienes",
    landing_eyebrow: "Tu cocina, tus recetas",
    landing_headline: "Ideas para la cena con lo que ya hay en tu cocina.",
    landing_sub: "Pantry Chef guarda los productos básicos que siempre tienes a mano y los convierte en recetas prácticas, para que compres menos y desperdicies menos.",
    get_started: "Comenzar",
    invite_note: "Solo con invitación: necesitas un código de quien te compartió Pantry Chef.",
    sample_pantry: "En tu despensa",
    sample_chips: &["Aceite de oliva", "Ajo", "Arroz", "Limones", "Huevos", "Espinacas"],
    sample_recipe_meta: "Cena · 25 min",
    sample_recipe_title: "Arroz frito con ajo y limón",
    sample_recipe_note: "Usa 6 de tus productos. No hace falta comprar nada.",
    how_title: "Cómo funciona",
    steps_how: &[
        ("Llena tu despensa", "Agrega lo básico que siempre tienes. Se ordena en secciones como frutas, repostería y especias."),
        ("Elige una comida", "Escoge desayuno, cena o postre, y agrega un ingrediente que quieras aprovechar o una nota como \"30 minutos máximo\"."),
        ("Recibe una receta", "Una receta completa con ingredientes, pasos y lo que valga la pena comprar, guardada en tu historial."),
    ],
    closing_line: "Gratis para amigos y familia. Disponible en inglés y español.",

    pantry_title: "Despensa",
    pantry_heading: "Tu despensa",
    pantry_intro: "Productos básicos que siempre tienes a mano. Las recetas darán preferencia a estos ingredientes.",
    generate_recipe: "Generar receta",
    add_staple: "Agregar producto",
    add_placeholder: "p. ej. aceite de oliva, ajo, arroz",
    add_button: "Agregar",
    pantry_empty: "Aún no tienes productos. Agrega algunos básicos que siempre tengas en casa.",
    remove: "Quitar",

    generate_heading: "Genera una receta",
    gemini_missing_html: "GEMINI_API_KEY no está configurada. Agrégala a tu archivo <code>.env</code> y reinicia la aplicación.",
    meal_type_label: "Tipo de comida",
    must_use_label: "Ingredientes obligatorios",
    must_use_hint: "(opcional, separados por comas)",
    must_use_placeholder: "muslos de pollo, espinacas",
    notes_label: "Notas",
    optional_hint: "(opcional)",
    notes_placeholder: "30 minutos máximo, sin lácteos, apto para niños",

    history_title: "Historial de recetas",
    history_intro: "Recetas que has generado antes.",
    history_empty: "Aún no hay recetas.",
    history_empty_link: "Genera la primera",

    login_title: "Iniciar sesión",
    register_title: "Registrarse",
    create_account_heading: "Crear cuenta",
    welcome_back: "¡Hola de nuevo!",
    auth_intro: "Guarda tus productos básicos y genera recetas con lo que tienes en casa.",
    username: "Usuario",
    password: "Contraseña",
    invite_code: "Código de invitación",
    register_button: "Registrarse",
    login_button: "Iniciar sesión",
    have_account: "¿Ya tienes una cuenta?",
    new_here: "¿Primera vez aquí?",
    create_account_link: "Crea una cuenta",

    serves: "Porciones:",
    prep: "Preparación",
    cook: "Cocción",
    minutes: "min",
    generate_another: "Generar otra",
    ingredients: "Ingredientes",
    steps: "Pasos",
    from_pantry: "De tu despensa",
    no_pantry_called_out: "No se mencionaron productos de la despensa.",
    optional_shopping: "Compras opcionales",
    have_everything: "Deberías tener todo lo necesario.",
};
