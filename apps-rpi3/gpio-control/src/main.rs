use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
    response::{IntoResponse},
    http::StatusCode
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use serde_json::{Value, json};
use std::collections::HashMap;
use rppal::gpio::Gpio;


const PORT: u16 = 8080;

const ANIMATIONS: [Animation; 7] = [
    // STEP 1: [WHITE] [RED][GREEN] [BLUE][YELLOW][MAGENTA]
    // STEP 2: [OFF] [OFF][OFF] [OFF][OFF][OFF]
    // STEP 3: [CIEL] [MAGENTA][YELLOW] [BLUE][GREEN][RED]
    Animation { animation_type: AnimationType::Blink },

    // STEP 1: [YELLOW] [RED][MAGENTA] [BLUE][CIEL][GREEN]
    // STEP 2: [GREEN] [YELLOW][RED] [MAGENTA][BLUE][CIEL]
    // STEP 3: [CIEL] [GREEN][YELLOW] [RED][MAGENTA][BLUE]
    Animation { animation_type: AnimationType::Wave },

    // STEP 1: [BLUE] [BLUE][BLUE] [BLUE][BLUE][BLUE]
    // STEP 2: [WHITE] [BLUE][BLUE] [BLUE][BLUE][BLUE]
    // STEP 3: [BLUE] [WHITE][WHITE] [BLUE][BLUE][BLUE]
    // STEP 4: [BLUE] [BLUE][BLUE] [WHITE][WHITE][WHITE]
    Animation { animation_type: AnimationType::Snow },

    // STEP 1: [MAGENTA] [YELLOW][MAGENTA] [YELLOW][MAGENTA][YELLOW]
    // STEP 2: [YELLOW] [MAGENTA][YELLOW] [MAGENTA][YELLOW][MAGENTA]
    // STEP 3: [RED] [CIEL][RED] [CIEL][RED][CIEL]
    // STEP 4: [CIEL] [RED][CIEL] [RED][CIEL][RED]
    Animation { animation_type: AnimationType::Alternate },

    Animation { animation_type: AnimationType::Spiral },

    // STEP 1: [OFF] [OFF][OFF] [BLUE][GREEN][CIEL]
    // STEP 2: [OFF] [MAGENTA][MAGENTA] [BLUE][GREEN][CIEL]
    // STEP 3: [YELLOW] [MAGENTA][MAGENTA] [BLUE][GREEN][CIEL]
    Animation { animation_type: AnimationType::BottomUp },

    // STEP 1: [RED] [OFF][OFF] [OFF][OFF][OFF]
    // STEP 2: [OFF] [OFF][OFF] [OFF][OFF][OFF]
    // STEP 3: [MAGENTA] [RED][RED] [OFF][OFF][OFF]
    // STEP 4: [CIEL] [MAGENTA][MAGENTA] [RED][RED][RED]
    Animation { animation_type: AnimationType::TopDown },
];

const NUM_RGB_LEDS: u8 = 6;
const NUM_NON_RGB_LEDS: u8 = 2;
const NUM_GPIO_PINS: u8 = 20;



#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum RgbState {
    White,
    Red,
    Green,
    Blue,
    Yellow,    // RGB(255, 255, 0)
    Magenta,   // RGB(255, 0, 255)
    Ciel,      // RGB(0, 255, 255)
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
    #[serde(rename = "bottom-up")]
    BottomUp,
    #[serde(rename = "top-down")]
    TopDown,
}



#[derive(Clone, Serialize)]
struct Animation {
    animation_type: AnimationType
}




#[derive(Clone, Copy, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
enum NonRgbRequestState {
    On,
    Blinking,
    Off,
}


#[derive(Deserialize, Debug)]
struct NonRgbLedRequest {
    id: u8,
    state: NonRgbRequestState,
}

struct AppState {
    gpio: Gpio,
    rgb_leds: Vec<RgbState>,        // 6 RGB LEDs
    non_rgb_leds: Vec<NonRgbState>, // 2 non-RGB LEDs
    current_animation_idx: usize,   
}

type SharedState = Arc<Mutex<AppState>>;





///////////////////////////////////////////////////////
////////////// Hardware GPIO Control //////////////////
///////////////////////////////////////////////////////

fn set_physical_rgb_led(idx: u8, desired_rgb_state: &RgbState, gpio: &Gpio) {
    if idx >= NUM_RGB_LEDS {
        return;
    }

    match desired_rgb_state {
        RgbState::White => {
            gpio.get(idx * 3).unwrap().into_output().set_high();           // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_high();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_high();       // BLUE
        },
        RgbState::Red => {
            gpio.get(idx * 3).unwrap().into_output().set_high();           // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_low();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_low();       // BLUE
        }
        RgbState::Green => {
            gpio.get(idx * 3).unwrap().into_output().set_low();           // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_high();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_low();       // BLUE
        },
        RgbState::Blue => {
            gpio.get(idx * 3).unwrap().into_output().set_low();           // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_low();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_high();       // BLUE
        },
        RgbState::Yellow => {
            // RGB(255, 255, 0)
            gpio.get(idx * 3).unwrap().into_output().set_high();           // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_high();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_low();       // BLUE
        },
        RgbState::Magenta => {
            // RGB(255, 0, 255)
            gpio.get(idx * 3).unwrap().into_output().set_high();           // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_low();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_high();       // BLUE
        },
        RgbState::Ciel => {
            // RGB(0, 255, 255)
            gpio.get(idx * 3).unwrap().into_output().set_low();            // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_high();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_high();       // BLUE
        }
        RgbState::Off => {
            gpio.get(idx * 3).unwrap().into_output().set_low();           // RED
            gpio.get(idx * 3 + 1).unwrap().into_output().set_low();       // GREEN
            gpio.get(idx * 3 + 2).unwrap().into_output().set_low();       // BLUE
        }
    }
}



fn set_physical_non_rgb_led(idx: u8, desired_non_rgb_state: &NonRgbState, gpio: &Gpio) {
    if idx >= NUM_NON_RGB_LEDS {
        return;
    }

    match desired_non_rgb_state {
        NonRgbState::On | NonRgbState::BlinkingOn =>
            gpio.get(NUM_RGB_LEDS * 3 + idx).unwrap().into_output().set_high(),
        NonRgbState::Off | NonRgbState::BlinkingOff => 
            gpio.get(NUM_RGB_LEDS * 3 + idx).unwrap().into_output().set_low(),
    }
}



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
        [RgbState::White, RgbState::Red, RgbState::Green, RgbState::Blue, RgbState::Yellow, RgbState::Magenta],
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Ciel, RgbState::Magenta, RgbState::Yellow, RgbState::Blue, RgbState::Green, RgbState::Red],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);

            for (idx, led_state) in app_state.rgb_leds.iter().enumerate() {
                set_physical_rgb_led(idx as u8, led_state, &app_state.gpio);
            }
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::Blink { return; }
    }
}

async fn animation_wave(state: &SharedState) {
    let frames = [
        [RgbState::Yellow, RgbState::Red, RgbState::Magenta, RgbState::Blue, RgbState::Ciel, RgbState::Green],
        [RgbState::Green, RgbState::Yellow, RgbState::Red, RgbState::Magenta, RgbState::Blue, RgbState::Ciel],
        [RgbState::Ciel, RgbState::Green, RgbState::Yellow, RgbState::Red, RgbState::Magenta, RgbState::Blue],
        [RgbState::Blue, RgbState::Ciel, RgbState::Green, RgbState::Yellow, RgbState::Red, RgbState::Magenta],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);

            for (idx, led_state) in app_state.rgb_leds.iter().enumerate() {
                set_physical_rgb_led(idx as u8, led_state, &app_state.gpio);
            }
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::Wave { return; }
    }
}

async fn animation_snow(state: &SharedState) {
    let frames = [
        [RgbState::Blue, RgbState::Blue, RgbState::Blue, RgbState::Blue, RgbState::Blue, RgbState::Blue],
        [RgbState::White, RgbState::Blue, RgbState::Blue, RgbState::Blue, RgbState::Blue, RgbState::Blue],
        [RgbState::Blue, RgbState::White, RgbState::White, RgbState::Blue, RgbState::Blue, RgbState::Blue],
        [RgbState::Blue, RgbState::Blue, RgbState::Blue, RgbState::White, RgbState::White, RgbState::White]
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);

            for (idx, led_state) in app_state.rgb_leds.iter().enumerate() {
                set_physical_rgb_led(idx as u8, led_state, &app_state.gpio);
            }
        }

        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::Snow { return; }
    }
}

async fn animation_alternate(state: &SharedState) {
    let frames = [
        [RgbState::Magenta, RgbState::Yellow, RgbState::Magenta, RgbState::Yellow, RgbState::Magenta, RgbState::Yellow],
        [RgbState::Yellow, RgbState::Magenta, RgbState::Yellow, RgbState::Magenta, RgbState::Yellow, RgbState::Magenta],
        [RgbState::Red, RgbState::Ciel, RgbState::Red, RgbState::Ciel, RgbState::Red, RgbState::Ciel],
        [RgbState::Ciel, RgbState::Red, RgbState::Ciel, RgbState::Red, RgbState::Ciel, RgbState::Red],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);

            for (idx, led_state) in app_state.rgb_leds.iter().enumerate() {
                set_physical_rgb_led(idx as u8, led_state, &app_state.gpio);
            }
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::Alternate { return; }
    }
}

async fn animation_spiral(state: &SharedState) {
    // Indecsii pt o miscare circulara pe brad
    let path = [0, 2, 5, 4, 3, 1];
    let colors = [RgbState::Magenta, RgbState::Yellow, RgbState::Ciel, RgbState::Red];

    for color in colors {
        for &active_idx in path.iter() {
            {
                let mut app_state = state.lock().unwrap();
                for (idx, led_state) in app_state.rgb_leds.iter_mut().enumerate() {
                    *led_state = if idx == active_idx { color } else { RgbState::Off };
                }

                for (idx, led_state) in app_state.rgb_leds.iter().enumerate() {
                    set_physical_rgb_led(idx as u8, led_state, &app_state.gpio);
                }
            }
            sleep(Duration::from_millis(1000)).await;
            if get_current_animation_type(&state).await != AnimationType::Spiral { return; }
        }
    }
}

async fn animation_bottom_up(state: &SharedState) {
    let frames = [
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Blue, RgbState::Green, RgbState::Ciel],
        [RgbState::Off, RgbState::Magenta, RgbState::Magenta, RgbState::Blue, RgbState::Green, RgbState::Ciel],
        [RgbState::Yellow, RgbState::Magenta, RgbState::Magenta, RgbState::Blue, RgbState::Green, RgbState::Ciel],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);


            for (idx, led_state) in app_state.rgb_leds.iter().enumerate() {
                set_physical_rgb_led(idx as u8, led_state, &app_state.gpio);
            }
        }
        sleep(Duration::from_millis(1000)).await;
        if get_current_animation_type(&state).await != AnimationType::BottomUp { return; }
    }
}

async fn animation_top_down(state: &SharedState) {
    let frames = [
        [RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Red, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Magenta, RgbState::Red, RgbState::Red, RgbState::Off, RgbState::Off, RgbState::Off],
        [RgbState::Ciel, RgbState::Magenta, RgbState::Magenta, RgbState::Red, RgbState::Red, RgbState::Red],
    ];

    for frame in frames {
        {
            let mut app_state = state.lock().unwrap();
            app_state.rgb_leds.copy_from_slice(&frame);

            for (idx, led_state) in app_state.rgb_leds.iter().enumerate() {
                set_physical_rgb_led(idx as u8, led_state, &app_state.gpio);
            }
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
                        NonRgbState::BlinkingOff, // Toggle: ON -> OFF
                    NonRgbState::BlinkingOff =>
                        NonRgbState::BlinkingOn   // Toggle: OFF -> ON
                };

                s.non_rgb_leds[idx as usize] = next_led_state;
                set_physical_non_rgb_led(idx, &next_led_state, &s.gpio);
            }
        }
        sleep(Duration::from_millis(1000)).await;
    }
}


///////////////////////////////////////////////////////
///////////////// HTTP endpoints //////////////////////
///////////////////////////////////////////////////////



async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "msg": "Serverul e bine, sanatos"
    }))
}


async fn get_ip_api_request() -> impl IntoResponse {
    let response = match reqwest::get("http://ip-api.com/json").await {
        Ok(res) => res,
        Err(err) => {
            return Json(json!({
                "err": format!("Failed to send request: {}", err)
            }));
        }
    };

    if !response.status().is_success() {
        return Json(json!({
            "err": format!("Server returned status: {}", response.status())
        }));
    }

    // Parsarea corpului raspunsului ca JSON
    match response.json::<Value>().await {
        Ok(data) => Json(data),
        Err(err) => Json(json!({
            "err": format!("Failed to parse JSON response: {}", err)
        })),
    }
}


async fn get_possible_led_states() -> impl IntoResponse {
    Json(json!({
        "RGB LED": [
            "white",
            "red",
            "green",
            "blue", 
            "yellow",
            "magenta",
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
            // Nu intrerupe daca este deja in animatie
            if s.non_rgb_leds[request.id as usize] != NonRgbState::BlinkingOn
                && s.non_rgb_leds[request.id as usize] != NonRgbState::BlinkingOff {
                    // LED-ul va fi setat imediat (<= 1 sec) la ON in GPIO loop
                    s.non_rgb_leds[request.id as usize] = NonRgbState::BlinkingOff;
            }
        }
    }

    Ok(StatusCode::OK)
}



#[tokio::main]
async fn main() {
    let gpio: Gpio = Gpio::new().unwrap();

    for i in 0..NUM_GPIO_PINS {
        let mut pin = gpio.get(i).unwrap().into_output();
        pin.set_low();
    }

    let state: SharedState = Arc::new(Mutex::new(AppState {
        gpio,
        rgb_leds: vec![RgbState::Off; 6],
        non_rgb_leds: vec![NonRgbState::Off; 2],
        current_animation_idx: 0,
    }));


    tokio::spawn(gpio_control_rgb_leds_loop(Arc::clone(&state)));
    tokio::spawn(gpio_control_non_rgb_leds_loop(Arc::clone(&state)));

    let app = Router::new()
        .route("/api/health-check", get(health_check))
        .route("/api/pub-ip-info", get(get_ip_api_request))
        .route("/api/possible-led-states", get(get_possible_led_states))
        .route("/api/rgb-leds", get(get_all_rgb_leds))
        .route("/api/non-rgb-leds", get(get_all_non_rgb_leds))
        .route("/api/animations", get(get_all_animations))
        .route("/api/current-animation", get(get_current_animation))
        .route("/api/current-animation/{id}", post(set_animation))
        .route("/api/next-animation", post(next_animation))
        .route("/api/prev-animation", post(prev_animation))
		.route("/api/non-rgb-led", post(set_non_rgb_led))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", PORT)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
