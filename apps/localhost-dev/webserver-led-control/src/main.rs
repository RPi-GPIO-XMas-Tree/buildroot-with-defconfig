use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
    response::{IntoResponse},
    http::StatusCode
};
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use serde_json::{Value, json};
use std::collections::HashMap;

const PORT: u16 = 8080;

const ANIMATIONS: [Animation; 7] = [
    // STEP 1: [WHITE] [RED][GREEN] [BLUE][YELLOW][PURPLE]
    // STEP 2: [OFF] [OFF][OFF] [OFF][OFF][OFF]
    // STEP 3: [CIEL] [PURPLE][YELLOW] [BLUE][GREEN][RED]
    Animation { animation_type: AnimationType::Blink },

    // STEP 1: [YELLOW] [RED][PURPLE] [BLUE][CIEL][GREEN]
    // STEP 2: [GREEN] [YELLOW][RED] [PURPLE][BLUE][CIEL]
    // STEP 3: [CIEL] [GREEN][YELLOW] [RED][PURPLE][BLUE]
    Animation { animation_type: AnimationType::Wave },

    // STEP 1: [BLUE] [CIEL][BLUE] [CIEL][CIEL][BLUE]
    // STEP 2: [WHITE] [BLUE][CIEL] [BLUE][WHITE][CIEL]
    // STEP 3: [CIEL] [WHITE][BLUE] [CIEL][BLUE][WHITE]
    Animation { animation_type: AnimationType::Snow },

    // STEP 1: [PURPLE] [YELLOW][PURPLE] [YELLOW][PURPLE][YELLOW]
    // STEP 2: [YELLOW] [PURPLE][YELLOW] [PURPLE][YELLOW][PURPLE]
    // STEP 3: [RED] [CIEL][RED] [CIEL][RED][CIEL]
    Animation { animation_type: AnimationType::Alternate },

    Animation { animation_type: AnimationType::Spiral },

    // STEP 1: [OFF] [OFF][OFF] [BLUE][GREEN][CIEL]
    // STEP 2: [OFF] [PURPLE][PURPLE] [BLUE][GREEN][CIEL]
    // STEP 3: [YELLOW] [PURPLE][PURPLE] [BLUE][GREEN][CIEL]
    Animation { animation_type: AnimationType::BottomUp },

    // STEP 1: [RED] [OFF][OFF] [OFF][OFF][OFF]
    // STEP 2: [PURPLE] [RED][RED] [OFF][OFF][OFF]
    // STEP 3: [CIEL] [PURPLE][PURPLE] [RED][RED][RED]
    Animation { animation_type: AnimationType::TopDown },
];

const NUM_RGB_LEDS: u8 = 6;
const NUM_NON_RGB_LEDS: u8 = 2;



#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RgbState {
    White,
    Red,
    Green,
    Blue,
    Yellow,   // RGB(255, 255, 0)
    Purple,   // RGB(255, 0, 255)
    Ciel,     // RGB(0, 255, 255)
    Off,
}

#[derive(Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum NonRgbState {
    On,
    #[serde(rename = "blinking-on")]
    BlinkingOn,
    #[serde(rename = "blinking-off")]
    BlinkingOff,
    Off,
}




#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum AnimationType {
    Blink,
    Wave,
    Snow,
    Alternate,
    Spiral,
    #[serde(rename = "buttom-up")]
    BottomUp,
    #[serde(rename = "top-down")]
    TopDown,
}



#[derive(Clone, Serialize)]
struct Animation {
    animation_type: AnimationType
}




#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum NonRgbRequestState {
    On,
    Blinking,
    Off,
}


#[derive(Deserialize)]
struct NonRgbLedRequest {
    id: u8,
    state: NonRgbRequestState,
}

struct AppState {
    rgb_leds: Vec<RgbState>,        // 6 RGB LEDs
    non_rgb_leds: Vec<NonRgbState>, // 2 non-RGB LEDs
    current_animation_idx: usize,   
}

type SharedState = Arc<Mutex<AppState>>;




///////////////////////////////////////////////////////
//////////////// LED Animations ///////////////////////
///////////////////////////////////////////////////////



async fn get_current_animation_type(state: &SharedState) -> AnimationType {
    let app_state = state.lock().unwrap();
    let idx = app_state.current_animation_idx;
    ANIMATIONS[idx].animation_type
}


async fn animation_blink(state: &SharedState) {
    let frames = [
        // Toate LED-urile aprinse cu un mix de culori
        [RgbState::White, RgbState::Red, RgbState::Green, RgbState::Blue, RgbState::Yellow, RgbState::Purple],
        // Toate stinse
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        // Culori secundare si Ciel
        [RgbState::Ciel, RgbState::Purple, RgbState::Yellow, RgbState::Blue, RgbState::Green, RgbState::Red],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::Blink { return; }
    }
}

async fn animation_wave(state: &SharedState) {
    let frames = [
        [RgbState::Yellow, RgbState::Red, RgbState::Purple, RgbState::Blue, RgbState::Ciel, RgbState::Green],
        [RgbState::Green, RgbState::Yellow, RgbState::Red, RgbState::Purple, RgbState::Blue, RgbState::Ciel],
        [RgbState::Ciel, RgbState::Green, RgbState::Yellow, RgbState::Red, RgbState::Purple, RgbState::Blue],
        [RgbState::Blue, RgbState::Ciel, RgbState::Green, RgbState::Yellow, RgbState::Red, RgbState::Purple],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);
        }
        sleep(Duration::from_millis(800)).await;
        if get_current_animation_type(&state).await != AnimationType::Blink { return; }
    }
}

async fn animation_snow(state: &SharedState) {
    let frames = [
        [RgbState::Ciel, RgbState::Blue, RgbState::Ciel, RgbState::Blue, RgbState::Ciel, RgbState::Blue],
        [RgbState::White, RgbState::Ciel, RgbState::Blue, RgbState::Ciel, RgbState::White, RgbState::Ciel],
        [RgbState::Ciel, RgbState::White, RgbState::Ciel, RgbState::White, RgbState::Blue, RgbState::White],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);
        }

        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::Blink { return; }
    }
}

async fn animation_alternate(state: &SharedState) {
    let frames = [
        [RgbState::Purple, RgbState::Yellow, RgbState::Purple, RgbState::Yellow, RgbState::Purple, RgbState::Yellow],
        [RgbState::Yellow, RgbState::Purple, RgbState::Yellow, RgbState::Purple, RgbState::Yellow, RgbState::Purple],
        [RgbState::Red, RgbState::Ciel, RgbState::Red, RgbState::Ciel, RgbState::Red, RgbState::Ciel],
        [RgbState::Ciel, RgbState::Red, RgbState::Ciel, RgbState::Red, RgbState::Ciel, RgbState::Red],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::Alternate { return; }
    }
}

async fn animation_spiral(state: &SharedState) {
    // Secventa de indecsi pentru a simula miscarea circulara pe brad
    let path = [0, 2, 5, 4, 3, 1];
    let colors = [RgbState::Purple, RgbState::Yellow, RgbState::Ciel, RgbState::Red];

    for color in colors {
        for &active_idx in path.iter() {
            {
                let mut app_state = state.lock().unwrap();
                for (idx, led_state) in app_state.rgb_leds.iter_mut().enumerate() {
                    *led_state = if idx == active_idx { color } else { RgbState::Off };
                }
            }
            sleep(Duration::from_millis(300)).await;
            if get_current_animation_type(&state).await != AnimationType::Spiral { return; }
        }
    }
}

async fn animation_bottom_up(state: &SharedState) {
    let frames = [
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Blue, RgbState::Green, RgbState::Ciel],
        [RgbState::Off, RgbState::Purple, RgbState::Purple, RgbState::Blue, RgbState::Green, RgbState::Ciel],
        [RgbState::Yellow, RgbState::Purple, RgbState::Purple, RgbState::Blue, RgbState::Green, RgbState::Ciel],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::BottomUp { return; }
    }
}

async fn animation_top_down(state: &SharedState) {
    let frames = [
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Red, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Purple, RgbState::Red, RgbState::Red, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Ciel, RgbState::Purple, RgbState::Purple, RgbState::Red, RgbState::Red, RgbState::Red],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::TopDown { return; }
    }
}


///////////////////////////////////////////////////////
/// Un task separat pentru un loop ////////////////////
/// care controleaza pinii GPIO ///////////////////////
//// ai LED-urilor RGB ////////////////////////////////
///////////////////////////////////////////////////////


async fn gpio_control_rgb_leds_loop(app_state: SharedState) {
    loop {
        let idx: usize;
        {
            idx = app_state.lock().unwrap().current_animation_idx;
        }


        match ANIMATIONS[idx].animation_type {
            AnimationType::Blink => 
                animation_blink(&app_state).await,
            
            AnimationType::Wave => 
                animation_wave(&app_state).await,
            
            AnimationType::Snow => 
                animation_snow(&app_state).await,
            
            AnimationType::Alternate => 
                animation_alternate(&app_state).await,
            
            AnimationType::Spiral => 
                animation_spiral(&app_state).await,
            
            AnimationType::BottomUp => 
                animation_bottom_up(&app_state).await,
            
            AnimationType::TopDown => 
                animation_top_down(&app_state).await,
        }
    }
}


///////////////////////////////////////////////////////
/// Un task separat pentru un loop ////////////////////
/// care controleaza pinii GPIO ///////////////////////
//// ai LED-urilor non-RGB ////////////////////////////
///////////////////////////////////////////////////////
async fn gpio_control_non_rgb_leds_loop(app_state: SharedState) {
    loop {
        {
            let mut s = app_state.lock().unwrap();

            for idx in 0..NUM_NON_RGB_LEDS {
                let next_led_state =  match s.non_rgb_leds[idx as usize] {
                    NonRgbState::On => NonRgbState::On,
                    NonRgbState::Off => NonRgbState::Off,
                    NonRgbState::BlinkingOn =>
                        NonRgbState::BlinkingOff, // Change state: ON -> OFF
                    NonRgbState::BlinkingOff =>
                        NonRgbState::BlinkingOn   // Change state: OFF -> ON
                };

                s.non_rgb_leds[idx as usize] = next_led_state;
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}


///////////////////////////////////////////////////////
/////////////// HTTP Methods //////////////////////////
///////////////////////////////////////////////////////



async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "msg": "Serverul e bine, sanatos"
    }))
}


async fn get_possible_led_states() -> impl IntoResponse {
    Json(json!({
        "RGB LED": [
            "white",
            "red",
            "green",
            "blue", 
            "yellow",
            "purple",
            "ciel",
            "off"
        ],
        "non-RGB LED": [
            "on",
            "blinking",
            "off"
        ]
    }))
}


async fn get_all_rgb_leds(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let s = state.lock().unwrap();

    let mut map: HashMap<usize, &RgbState> = HashMap::new();

    for (idx, led_state) in s.rgb_leds.iter().enumerate() {
        map.insert(idx, led_state);
    }

    (StatusCode::OK, Json(json!(map)))
}


async fn get_all_non_rgb_leds(
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let s = state.lock().unwrap();

    let mut map: HashMap<usize, &NonRgbState> = HashMap::new();

    for (idx, led_state) in s.non_rgb_leds.iter().enumerate() {
        map.insert(idx, led_state);
    }

    (StatusCode::OK, Json(json!(map)))
}

async fn get_all_animations() -> impl IntoResponse {
    let mut map: HashMap<usize, AnimationType> = HashMap::new();

    for (idx, animation) in ANIMATIONS.iter().enumerate() {
        map.insert(idx, animation.animation_type);
    }

    (StatusCode::OK, Json(json!(map)))
}


async fn set_non_rgb_led(
    State(state): State<SharedState>,
    Json(request): Json<NonRgbLedRequest>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut s = state.lock().unwrap();

    if request.id >= NUM_NON_RGB_LEDS {
        let error_response = (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid non-RGB LED ID='{}'. Expected a value between 0 and {}",
                    request.id, NUM_NON_RGB_LEDS - 1)
            }))
        );
        return Err(error_response);
    }

    match request.state {
        NonRgbRequestState::On =>
            s.non_rgb_leds[request.id as usize] = NonRgbState::On,
        
        NonRgbRequestState::Off =>
            s.non_rgb_leds[request.id as usize] = NonRgbState::Off,

        NonRgbRequestState::Blinking => {
            // If it's already blinking, SKIP it!
            if s.non_rgb_leds[request.id as usize] != NonRgbState::BlinkingOn
                && s.non_rgb_leds[request.id as usize] != NonRgbState::BlinkingOff {
                    // The LED will be imediately (1 <= sec) set ON in the GPIO loop
                    s.non_rgb_leds[request.id as usize] = NonRgbState::BlinkingOff;
            }
        }
    }

    Ok(StatusCode::OK)
}


async fn get_current_animation(
    State(state): State<SharedState>,
) -> Json<serde_json::Value> {
    let s = state.lock().unwrap();

    let idx: usize = s.current_animation_idx;
    Json(serde_json::json!({
        "id": idx,
        "name": serde_json::to_value(&ANIMATIONS[idx].animation_type).unwrap()
    }))
}


async fn set_animation(
    State(state): State<SharedState>,
    Path(id): Path<usize>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    let mut s = state.lock().unwrap();
    if id < ANIMATIONS.len() {
        s.current_animation_idx = id;
        Ok(StatusCode::OK)
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid animation ID='{}'. Expected a value between 0 and {}",
                    id, ANIMATIONS.len() - 1)
            }))
        ))
    }
}


async fn next_animation(State(state): State<SharedState>) -> impl IntoResponse {
    let mut s = state.lock().unwrap();


    s.current_animation_idx = (s.current_animation_idx + 1) % ANIMATIONS.len();
    let idx: usize = s.current_animation_idx;

    (
        StatusCode::OK,
        Json(json!({
            "id": idx,
            "name": ANIMATIONS[idx].animation_type
        }))
    )
}


async fn prev_animation(State(state): State<SharedState>) -> impl IntoResponse {
    let mut s = state.lock().unwrap();


    s.current_animation_idx = match s.current_animation_idx {
        0 => ANIMATIONS.len() - 1,
        idx => idx - 1
    };

    let idx: usize = s.current_animation_idx;

    (
        StatusCode::OK,
        Json(json!({
            "id": idx,
            "name": ANIMATIONS[idx].animation_type
        }))
    )
}


async fn get_rgb_led(
    State(state): State<SharedState>,
    Path(id): Path<usize>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> {
    let s = state.lock().unwrap();

    if id > NUM_RGB_LEDS as usize {
        let error_response = (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("Invalid non-RGB LED ID='{}'. Expected a value between 1 and {}",
                    id, NUM_NON_RGB_LEDS)
            }))
        );
        return Err(error_response);
    }

    let led = &s.rgb_leds[id];


    // Json(serde_json::json!({ "state": led }))
    let success_response = (
        StatusCode::OK,
        Json(json!({
            "status": led
        }))
    );

    Ok(success_response)
}


async fn get_non_rgb_led(
    State(state): State<SharedState>,
    Path(id): Path<usize>,
) -> Json<serde_json::Value> {
    let s = state.lock().unwrap();
    let led = &s.non_rgb_leds[id];
    Json(serde_json::json!({ "state": led }))
}



#[tokio::main]
async fn main() {
    let state: SharedState = Arc::new(Mutex::new(AppState {
        rgb_leds: vec![RgbState::Off; 6],
        non_rgb_leds: vec![NonRgbState::Off; 2],
        current_animation_idx: 0,
    }));


    tokio::spawn(gpio_control_rgb_leds_loop(Arc::clone(&state)));
    tokio::spawn(gpio_control_non_rgb_leds_loop(Arc::clone(&state)));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/health-check", get(health_check))
        .route("/api/possible-led-states", get(get_possible_led_states))
        .route("/api/rgb-leds", get(get_all_rgb_leds))
        .route("/api/non-rgb-leds", get(get_all_non_rgb_leds))
        .route("/api/animations", get(get_all_animations))
        .route("/api/current-animation", get(get_current_animation))
        .route("/api/current-animation/{id}", post(set_animation))
        .route("/api/next-animation", post(next_animation))
        .route("/api/prev-animation", post(prev_animation))
        .route("/api/rgb-led/{id}", get(get_rgb_led))
        .route("/api/non-rgb-led/{id}", get(get_non_rgb_led))
        .route("/api/non-rgb-led", post(set_non_rgb_led))
        .with_state(state)
        .layer(cors);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", PORT)).await.unwrap();
    println!(
        "Server running on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}
