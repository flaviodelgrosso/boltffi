use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Location {
    pub id: i64,
    pub lat: f64,
    pub lng: f64,
    pub rating: f64,
    pub review_count: i32,
    pub is_open: bool,
}

#[wasm_bindgen]
impl Location {
    #[wasm_bindgen(constructor)]
    pub fn new(id: i64, lat: f64, lng: f64, rating: f64, review_count: i32, is_open: bool) -> Location {
        Location { id, lat, lng, rating, review_count, is_open }
    }
}

#[wasm_bindgen]
pub struct Trade {
    pub id: i64,
    pub symbol_id: i32,
    pub price: f64,
    pub quantity: i64,
    pub bid: f64,
    pub ask: f64,
    pub volume: i64,
    pub timestamp: i64,
    pub is_buy: bool,
}

#[wasm_bindgen]
impl Trade {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: i64, symbol_id: i32, price: f64, quantity: i64,
        bid: f64, ask: f64, volume: i64, timestamp: i64, is_buy: bool
    ) -> Trade {
        Trade { id, symbol_id, price, quantity, bid, ask, volume, timestamp, is_buy }
    }
}

#[wasm_bindgen]
pub struct Particle {
    pub id: i64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub mass: f64,
    pub charge: f64,
    pub active: bool,
}

#[wasm_bindgen]
impl Particle {
    #[wasm_bindgen(constructor)]
    pub fn new(
        id: i64, x: f64, y: f64, z: f64,
        vx: f64, vy: f64, vz: f64,
        mass: f64, charge: f64, active: bool
    ) -> Particle {
        Particle { id, x, y, z, vx, vy, vz, mass, charge, active }
    }
}

#[wasm_bindgen]
pub struct SensorReading {
    pub sensor_id: i64,
    pub timestamp: i64,
    pub temperature: f64,
    pub humidity: f64,
    pub pressure: f64,
    pub light: f64,
    pub battery: f64,
    pub signal_strength: i32,
    pub is_valid: bool,
}

#[wasm_bindgen]
impl SensorReading {
    #[wasm_bindgen(constructor)]
    pub fn new(
        sensor_id: i64, timestamp: i64, temperature: f64, humidity: f64,
        pressure: f64, light: f64, battery: f64, signal_strength: i32, is_valid: bool
    ) -> SensorReading {
        SensorReading { sensor_id, timestamp, temperature, humidity, pressure, light, battery, signal_strength, is_valid }
    }
}

#[wasm_bindgen]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub timestamp: i64,
}

#[wasm_bindgen]
impl DataPoint {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, timestamp: i64) -> DataPoint {
        DataPoint { x, y, timestamp }
    }
}

#[wasm_bindgen]
pub fn noop() {}

#[wasm_bindgen]
pub fn echo_i32(value: i32) -> i32 {
    value
}

#[wasm_bindgen]
pub fn echo_f64(value: f64) -> f64 {
    value
}

#[wasm_bindgen]
pub fn echo_string(value: &str) -> String {
    value.to_string()
}

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn multiply(a: f64, b: f64) -> f64 {
    a * b
}

#[wasm_bindgen]
pub fn generate_string(size: i32) -> String {
    "x".repeat(size as usize)
}

#[wasm_bindgen]
pub fn generate_locations(count: i32) -> Vec<Location> {
    (0..count)
        .map(|i| Location {
            id: i as i64,
            lat: 37.7749 + (i as f64 * 0.001),
            lng: -122.4194 + (i as f64 * 0.001),
            rating: 3.0 + ((i % 20) as f64 * 0.1),
            review_count: 10 + (i * 5),
            is_open: i % 2 == 0,
        })
        .collect()
}

#[wasm_bindgen]
pub fn sum_location_ratings(locations: Vec<Location>) -> f64 {
    locations.iter().map(|l| l.rating).sum()
}

#[wasm_bindgen]
pub fn generate_trades(count: i32) -> Vec<Trade> {
    (0..count)
        .map(|i| Trade {
            id: i as i64,
            symbol_id: i % 500,
            price: 100.0 + (i as f64 * 0.01),
            quantity: (i as i64 % 1000) + 1,
            bid: 99.95 + (i as f64 * 0.01),
            ask: 100.05 + (i as f64 * 0.01),
            volume: (i as i64) * 1000,
            timestamp: 1700000000000 + (i as i64 * 1000),
            is_buy: i % 2 == 0,
        })
        .collect()
}

#[wasm_bindgen]
pub fn sum_trade_volumes(trades: Vec<Trade>) -> i64 {
    trades.iter().map(|t| t.volume).sum()
}

#[wasm_bindgen]
pub fn generate_particles(count: i32) -> Vec<Particle> {
    (0..count)
        .map(|i| Particle {
            id: i as i64,
            x: (i as f64) * 0.1,
            y: (i as f64) * 0.2,
            z: (i as f64) * 0.3,
            vx: (i as f64) * 0.01,
            vy: (i as f64) * 0.02,
            vz: (i as f64) * 0.03,
            mass: 1.0 + (i as f64 * 0.001),
            charge: if i % 2 == 0 { 1.0 } else { -1.0 },
            active: i % 10 != 0,
        })
        .collect()
}

#[wasm_bindgen]
pub fn sum_particle_masses(particles: Vec<Particle>) -> f64 {
    particles.iter().map(|p| p.mass).sum()
}

#[wasm_bindgen]
pub fn generate_sensor_readings(count: i32) -> Vec<SensorReading> {
    (0..count)
        .map(|i| SensorReading {
            sensor_id: (i % 100) as i64,
            timestamp: 1700000000000 + (i as i64 * 100),
            temperature: 20.0 + ((i % 30) as f64),
            humidity: 40.0 + ((i % 40) as f64),
            pressure: 1013.25 + ((i % 20) as f64),
            light: (i % 1000) as f64,
            battery: 100.0 - ((i % 100) as f64),
            signal_strength: -50 - (i % 50),
            is_valid: i % 20 != 0,
        })
        .collect()
}

#[wasm_bindgen]
pub fn avg_sensor_temperature(readings: Vec<SensorReading>) -> f64 {
    let sum: f64 = readings.iter().map(|r| r.temperature).sum();
    sum / readings.len() as f64
}

#[wasm_bindgen]
pub fn generate_bytes(size: i32) -> Vec<u8> {
    vec![42u8; size as usize]
}

#[wasm_bindgen]
pub fn generate_i32_vec(count: i32) -> Vec<i32> {
    (0..count).collect()
}

#[wasm_bindgen]
pub fn sum_i32_vec(values: Vec<i32>) -> i64 {
    values.iter().map(|&v| v as i64).sum()
}

#[wasm_bindgen]
pub fn generate_f64_vec(count: i32) -> Vec<f64> {
    (0..count).map(|i| i as f64 * 0.1).collect()
}

#[wasm_bindgen]
pub fn sum_f64_vec(values: Vec<f64>) -> f64 {
    values.iter().sum()
}

#[wasm_bindgen]
pub struct Counter {
    value: u64,
}

#[wasm_bindgen]
impl Counter {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Counter {
        Counter { value: 0 }
    }

    pub fn increment(&mut self) {
        self.value += 1;
    }

    pub fn get(&self) -> u64 {
        self.value
    }

    pub fn set(&mut self, value: u64) {
        self.value = value;
    }
}

#[wasm_bindgen]
pub struct DataStore {
    items: Vec<DataPoint>,
}

#[wasm_bindgen]
impl DataStore {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DataStore {
        DataStore { items: Vec::new() }
    }

    pub fn add(&mut self, point: DataPoint) {
        self.items.push(point);
    }

    pub fn add_parts(&mut self, x: f64, y: f64, timestamp: i64) {
        self.items.push(DataPoint { x, y, timestamp });
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn sum(&self) -> f64 {
        self.items.iter().map(|p| p.x + p.y).sum()
    }
}

#[wasm_bindgen]
pub struct Accumulator {
    value: i64,
}

#[wasm_bindgen]
impl Accumulator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Accumulator {
        Accumulator { value: 0 }
    }

    pub fn add(&mut self, amount: i64) {
        self.value += amount;
    }

    pub fn get(&self) -> i64 {
        self.value
    }

    pub fn reset(&mut self) {
        self.value = 0;
    }
}

#[wasm_bindgen]
pub async fn async_add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn find_even(value: i32) -> Option<i32> {
    if value % 2 == 0 { Some(value) } else { None }
}
