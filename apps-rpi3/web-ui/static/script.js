let animations = {}; 

// `http://127.0.0.1:8080/api` e tzeapa mare
// eu credeam ca request-ul se duce catre serviciul de GPIO Control,
// dar de fapt se face la mine pe calculator (nu in QEMU)!!!
const GPIO_BASE_API = '/api';

const colorMap = {
    "white":  "rgb(255, 255, 255)",
    "red":    "rgb(255, 0, 0)",
    "green":  "rgb(0, 255, 0)",
    "blue":   "rgb(0, 0, 255)",
    "yellow": "rgb(255, 255, 0)",
    "magenta": "rgb(255, 0, 255)",
    "ciel":   "rgb(0, 255, 255)",
    "off":    "rgb(17, 17, 17)"
};

// Culori diferite pentru LED-uri Non-RGB
const nonRgbColors = {
    "0": "rgb(255, 165, 0)", 
    "1": "rgb(255, 20, 147)" 
};

async function fetchPubIpInfo() {
    try {
        const response = await fetch(`${GPIO_BASE_API}/pub-ip-info`);
        const data = await response.json();
        document.getElementById('ip-address').innerText = data.query;
        document.getElementById('location').innerText = `${data.city}, ${data.country}`;
    } catch (e) {}
}

// LOOP: RGB LEDs
async function updateRGBLedsLoop() {
    try {
        const response = await fetch(`${GPIO_BASE_API}/rgb-leds`, { 
            headers: { 'Cache-Control': 'no-cache' } 
        });
        if (response.ok) {
            const data = await response.json();
            Object.keys(data).forEach(id => {
                const led = document.getElementById(`rgb-${id}`);
                if (led) {
                    const status = data[id].toLowerCase();
                    const colorValue = colorMap[status] || colorMap["off"];
                    led.style.backgroundColor = colorValue;
                    led.style.boxShadow = (status !== "off") ? `0 0 12px ${colorValue}` : "none";
                }
            });
        }
    } catch (e) {}
    setTimeout(updateRGBLedsLoop, 1000);
}

// LOOP: Non-RGB LEDs
async function updateNonRgbLedsLoop() {
    try {
        const response = await fetch(`${GPIO_BASE_API}/non-rgb-leds`, { 
            headers: { 'Cache-Control': 'no-cache' } 
        });
        if (response.ok) {
            const data = await response.json();
            Object.keys(data).forEach(id => {
                const led = document.getElementById(`non-rgb-${id}`);
                const ctrl = document.getElementById(`non-rgb-ctrl-${id}`);
                const state = data[id];

                if (led) {
                    const isOn = (state === "on" || state === "blinking-on");
                    const colorValue = isOn ? nonRgbColors[id] : "rgb(30, 30, 30)";
                    led.style.backgroundColor = colorValue;
                    led.style.boxShadow = isOn ? `0 0 15px ${colorValue}` : "none";
                }

                if (ctrl) {
                    // Sincronizare dropdown
                    if (state.includes("blinking")) {
                        ctrl.value = "blinking";
                    } else {
                        ctrl.value = state;
                    }
                }
            });
        }
    } catch (e) {}
    setTimeout(updateNonRgbLedsLoop, 1000);
}

async function setNonRgbState(id, state) {
    try {
        await fetch(`${GPIO_BASE_API}/non-rgb-led`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ id: parseInt(id), state: state })
        });
    } catch (e) {
        console.error("Error setting non-rgb state");
    }
}

async function updateCurrentAnimationText() {
    try {
        const response = await fetch(`${GPIO_BASE_API}/current-animation`);
        const data = await response.json();
        document.getElementById('current-animation').innerText = animations[data.id] || data.name;
        document.getElementById('animation-dropdown').value = data.id;
    } catch (e) {}
}

async function initAnimations() {
    try {
        const response = await fetch(`${GPIO_BASE_API}/animations`);
        animations = await response.json();
        const dropdown = document.getElementById('animation-dropdown');
        dropdown.innerHTML = ''; 
        for (const [id, name] of Object.entries(animations)) {
            const option = document.createElement('option');
            option.value = id; option.text = name.toUpperCase();
            dropdown.appendChild(option);
        }
        await updateCurrentAnimationText();
    } catch (e) {}
}

async function selectAnim() {
    const val = document.getElementById('animation-dropdown').value;
    if (val) {
        await fetch(`${GPIO_BASE_API}/current-animation/${val}`, { method: 'POST' });
        setTimeout(updateCurrentAnimationText, 100);
    }
}

async function nextAnim() {
    await fetch(`${GPIO_BASE_API}/next-animation`, { method: 'POST' });
    setTimeout(updateCurrentAnimationText, 100);
}

async function prevAnim() {
    await fetch(`${GPIO_BASE_API}/prev-animation`, { method: 'POST' });
    setTimeout(updateCurrentAnimationText, 100);
}

document.addEventListener('DOMContentLoaded', () => {
    fetchPubIpInfo();
    initAnimations();
    updateRGBLedsLoop();
    updateNonRgbLedsLoop();
});
