use log::debug;
use rand::{Rng, RngCore, SeedableRng, rngs::OsRng};
use rand_chacha::ChaCha12Rng;
use serde::{Deserialize, Serialize};

pub type Credits = i64;

const NEW_STAR_THRESHOLD: f32 = 10.0; // average raw class score needed
const MAX_SYSTEMS: usize = 12;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Merchandise {
    Uranium,
    Metals,
    HeavyEquipment,
    Medicine,
    Software,
    Gems,
}

impl Merchandise {
    pub const ALL: [Self; 6] = [
        Merchandise::Uranium,
        Merchandise::Metals,
        Merchandise::HeavyEquipment,
        Merchandise::Medicine,
        Merchandise::Software,
        Merchandise::Gems,
    ];

    #[allow(dead_code)]
    pub fn label(self) -> &'static str {
        match self {
            Merchandise::Uranium => "Uranium",
            Merchandise::Metals => "Metals",
            Merchandise::HeavyEquipment => "Heavy Equip",
            Merchandise::Medicine => "Medicine",
            Merchandise::Software => "Software",
            Merchandise::Gems => "Star Gems",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Merchandise::Uranium => "UR",
            Merchandise::Metals => "MET",
            Merchandise::HeavyEquipment => "HE",
            Merchandise::Medicine => "MED",
            Merchandise::Software => "SOFT",
            Merchandise::Gems => "GEMS",
        }
    }

    #[allow(dead_code)]
    pub fn weight_tons(self) -> u16 {
        match self {
            Merchandise::Uranium
            | Merchandise::Metals
            | Merchandise::HeavyEquipment
            | Merchandise::Medicine => 1,
            Merchandise::Software | Merchandise::Gems => 0,
        }
    }

    pub fn idx(self) -> usize {
        self as usize
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum StarClass {
    I,
    II,
    III,
    IV,
}

impl StarClass {
    pub fn from_score(score: i16) -> Self {
        match score {
            s if s >= 15 => StarClass::I,
            s if s >= 10 => StarClass::II,
            s if s >= 5 => StarClass::III,
            _ => StarClass::IV,
        }
    }

    pub fn raw_score(self) -> i16 {
        match self {
            StarClass::I => 15,
            StarClass::II => 10,
            StarClass::III => 5,
            StarClass::IV => 0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            StarClass::I => "Class I (Cosmopolitan)",
            StarClass::II => "Class II (Developed)",
            StarClass::III => "Class III (Underdeveloped)",
            StarClass::IV => "Class IV (Frontier)",
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GameDate {
    pub day: u16,
    pub year: i32,
}

impl GameDate {
    pub fn new(day: u16, year: i32) -> Self {
        Self { day, year }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StarSystem {
    pub id: usize,
    pub name: String,
    pub class: StarClass,
    pub dev_score: f32,
    pub coords: (i16, i16),
    pub stock: [i32; 6],
    pub last_update: GameDate,
    pub prices: [i32; 6],
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ship {
    pub id: usize,
    pub name: String,
    pub location: usize,
    pub destination: usize,
    pub cash: Credits,
    pub cargo: [i32; 6],
    pub net_tonnage: i32,
    pub eta: GameDate,
    pub delay_weeks: u8,
}

impl Ship {
    #[allow(dead_code)]
    pub fn current_system_id(&self) -> usize {
        self.location
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BankAccount {
    pub owner: usize,
    pub balance: Credits,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PriceModel {
    pub base_prices: [i32; 6],
    pub m: [[f32; 3]; 6],
    pub c: [[f32; 3]; 6],
    pub base_profit_margin: f32,
}

impl PriceModel {
    pub fn original() -> Self {
        // Mirrors MAT READ M,C and the fixed price vector Q from the BASIC version.
        let m = [
            [-0.025, -0.05, -0.025],
            [0.0, -0.025, -0.025],
            [0.0, 0.1, 0.1],
            [-0.025, 0.1, 0.0],
            [0.1, 0.2, 0.1],
            [0.1, -0.025, 0.0],
        ];

        let c = [
            [1.0, 1.5, 0.5],
            [0.75, 0.75, 0.75],
            [-0.25, -0.25, -0.25],
            [0.0, -0.5, 0.5],
            [0.0, -0.5, 0.0],
            [0.5, 1.5, 0.0],
        ];

        let base_prices = [5000, 3500, 4000, 4500, 3000, 3000];

        Self {
            base_prices,
            m,
            c,
            base_profit_margin: 36.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub date: GameDate,
    pub systems: Vec<StarSystem>,
    pub ships: Vec<Ship>,
    pub selected_system: usize,
    pub selected_ship: usize,
    pub selected_good: usize,
    pub price_model: PriceModel,
    pub max_tonnage: i32,
    pub speed_ly_per_day: f32,
    pub delay_probability: f32,
    pub development_increment: f32,
    pub bank_balance: Credits,
    pub bank_last_date: GameDate,
    pub max_bid_rounds: usize,
    pub rng: ChaCha12Rng,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TradeKind {
    BuyFromSystem,
    SellToSystem,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TradeSession {
    pub kind: TradeKind,
    pub ship_id: usize,
    pub system_id: usize,
    pub good_idx: usize,
    pub qty: i32,
    pub price_each: f32,
    pub round: usize,
    pub max_rounds: usize,
}

#[derive(Debug, Clone)]
pub enum HaggleOutcome {
    Accept(String),
    Counter(TradeSession, String),
    Reject(String),
}

impl GameState {
    pub fn demo() -> Self {
        let mut seed_rng = OsRng;
        Self::demo_with_seed(seed_rng.next_u64())
    }

    pub fn demo_with_seed(seed: u64) -> Self {
        let price_model = PriceModel::original();
        let systems = vec![
            StarSystem {
                id: 0,
                name: "SOL".to_string(),
                class: StarClass::I,
                dev_score: 15.0,
                coords: (0, 0),
                stock: [15, 0, 15, 10, 10, 0],
                last_update: GameDate::new(1, 2070),
                prices: [0; 6],
            },
            StarSystem {
                id: 1,
                name: "YORK".to_string(),
                class: StarClass::III,
                dev_score: 5.0,
                coords: (25, 8),
                stock: [-5, -5, 0, 0, 0, 5],
                last_update: GameDate::new(1, 2070),
                prices: [0; 6],
            },
        ];

        let ships = vec![
            Ship {
                id: 0,
                name: "PIONEER".to_string(),
                location: 0,
                destination: 1,
                cash: 5000,
                cargo: [0; 6],
                net_tonnage: 0,
                eta: GameDate::new(1, 2070),
                delay_weeks: 0,
            },
            Ship {
                id: 1,
                name: "VENTURE".to_string(),
                location: 0,
                destination: 1,
                cash: 5000,
                cargo: [0; 6],
                net_tonnage: 0,
                eta: GameDate::new(1, 2070),
                delay_weeks: 0,
            },
        ];

        let mut state = Self {
            date: GameDate::new(1, 2070),
            systems,
            ships,
            selected_system: 0,
            selected_ship: 0,
            selected_good: 0,
            price_model,
            max_tonnage: 30,
            speed_ly_per_day: 2.0 / 7.0,
            delay_probability: 0.1,
            development_increment: 1.25,
            bank_balance: 0,
            bank_last_date: GameDate::new(1, 2070),
            max_bid_rounds: 3,
            rng: ChaCha12Rng::seed_from_u64(seed),
        };

        state.reprice_all();
        state.autopilot_demo_routes();
        state
    }

    pub fn cycle_ship(&mut self, delta: i8) {
        if self.ships.is_empty() {
            return;
        }
        let len = self.ships.len() as i32;
        let idx = (self.selected_ship as i32 + delta as i32).rem_euclid(len);
        self.selected_ship = idx as usize;
    }

    pub fn cycle_system(&mut self, delta: i8) {
        if self.systems.is_empty() {
            return;
        }
        let len = self.systems.len() as i32;
        let idx = (self.selected_system as i32 + delta as i32).rem_euclid(len);
        self.selected_system = idx as usize;
    }

    pub fn cycle_good(&mut self, delta: i8) {
        let len = Merchandise::ALL.len() as i32;
        let idx = (self.selected_good as i32 + delta as i32).rem_euclid(len);
        self.selected_good = idx as usize;
    }

    pub fn selected_ship(&self) -> Option<&Ship> {
        self.ships.get(self.selected_ship)
    }

    pub fn selected_system(&self) -> Option<&StarSystem> {
        self.systems.get(self.selected_system)
    }

    pub fn start_trade_session(
        &self,
        kind: TradeKind,
        ship_id: usize,
        system_id: usize,
        good_idx: usize,
        qty: i32,
    ) -> Result<TradeSession, String> {
        if qty <= 0 {
            return Err("Quantity must be positive".to_string());
        }
        if system_id >= self.systems.len() || ship_id >= self.ships.len() {
            return Err("Invalid selection".to_string());
        }
        if !self.ship_at_system(ship_id, system_id) {
            return Err("Ship is not at this system".to_string());
        }

        match kind {
            TradeKind::BuyFromSystem => {
                if self.systems[system_id].stock[good_idx] < qty {
                    return Err("Not enough stock".to_string());
                }
                let mut weight = self.ships[ship_id].net_tonnage;
                if good_idx < 4 {
                    weight += qty;
                    if weight > self.max_tonnage {
                        return Err("Exceeds tonnage limit".to_string());
                    }
                }
                let price_each = self.systems[system_id].prices[good_idx] as f32;
                if price_each <= 0.0 {
                    return Err("Item not for sale".to_string());
                }
                Ok(TradeSession {
                    kind,
                    ship_id,
                    system_id,
                    good_idx,
                    qty,
                    price_each,
                    round: 1,
                    max_rounds: self.max_bid_rounds,
                })
            }
            TradeKind::SellToSystem => {
                if self.ships[ship_id].cargo[good_idx] < qty {
                    return Err("Not enough cargo".to_string());
                }
                if self.systems[system_id].stock[good_idx] >= 0 {
                    return Err("System not buying this good".to_string());
                }
                let max_buy = -self.systems[system_id].stock[good_idx] * 2;
                if qty > max_buy {
                    return Err(format!("They'll consider up to {} units", max_buy));
                }
                let price_each = self.systems[system_id].prices[good_idx] as f32;
                if price_each <= 0.0 {
                    return Err("Price unavailable".to_string());
                }
                Ok(TradeSession {
                    kind,
                    ship_id,
                    system_id,
                    good_idx,
                    qty,
                    price_each,
                    round: 1,
                    max_rounds: self.max_bid_rounds,
                })
            }
        }
    }

    pub fn evaluate_offer(&mut self, session: TradeSession, total_price: i32) -> HaggleOutcome {
        debug!(
            "Haggle start: {:?} ship={} system={} good={} qty={} round={} total={}",
            session.kind,
            session.ship_id,
            session.system_id,
            session.good_idx,
            session.qty,
            session.round,
            total_price
        );
        match session.kind {
            TradeKind::BuyFromSystem => self.eval_buy_offer(session, total_price),
            TradeKind::SellToSystem => self.eval_sell_offer(session, total_price),
        }
    }

    fn eval_buy_offer(&mut self, mut session: TradeSession, offer_total: i32) -> HaggleOutcome {
        if offer_total <= 0 {
            return HaggleOutcome::Reject("Offer must be positive".to_string());
        }
        let fair_total = session.price_each * session.qty as f32;
        let fnz = self.fnz(
            session.qty,
            session.system_id,
            session.good_idx,
            session.round,
        );
        let min_total = fair_total * (1.0 - fnz);
        let max_total = fair_total * (1.0 + fnz);

        if (offer_total as f32) < min_total {
            debug!(
                "Reject buy: offer {} < min {:.0} (fair {:.0}, fnz {:.3})",
                offer_total, min_total, fair_total, fnz
            );
            return HaggleOutcome::Reject(format!("Rejected: too low (<{:.0})", min_total));
        }

        if offer_total as f32 > max_total {
            debug!(
                "Reject buy: offer {} > max {:.0} (fair {:.0}, fnz {:.3})",
                offer_total, max_total, fair_total, fnz
            );
            return HaggleOutcome::Reject(format!("Rejected: too high (>{:.0})", max_total));
        }

        if offer_total as f32 <= fair_total {
            debug!(
                "Accept buy: offer {} <= fair {:.0} (round {}/{})",
                offer_total, fair_total, session.round, session.max_rounds
            );
            return match self.execute_buy(session, offer_total) {
                Ok(msg) => HaggleOutcome::Accept(msg),
                Err(e) => HaggleOutcome::Reject(e),
            };
        }

        // Counter-offer path on overpay (nudge price back toward fair)
        if session.round >= session.max_rounds {
            debug!("Reject buy: overpay on final round");
            return HaggleOutcome::Reject("They pass after final round.".to_string());
        }

        let target_price = fair_total / session.qty as f32;
        let new_price = 0.6 * session.price_each + 0.4 * target_price;
        session.price_each = new_price;
        session.round += 1;
        debug!(
            "Counter buy: new price_each {:.2}, fair {:.2}, qty {}, round {}/{}",
            new_price, target_price, session.qty, session.round, session.max_rounds
        );
        HaggleOutcome::Counter(
            session,
            format!(
                "Countered; new ask about ${:.0}",
                new_price * session.qty as f32
            ),
        )
    }

    fn eval_sell_offer(&mut self, mut session: TradeSession, ask_total: i32) -> HaggleOutcome {
        if ask_total <= 0 {
            return HaggleOutcome::Reject("Price must be positive".to_string());
        }
        let fair_total = session.price_each * session.qty as f32;
        let fnz = self.fnz(
            session.qty,
            session.system_id,
            session.good_idx,
            session.round,
        );
        let min_total = fair_total * (1.0 - fnz);

        if (ask_total as f32) < min_total {
            debug!(
                "Reject sell: ask {} < min {:.0} (fair {:.0}, fnz {:.3})",
                ask_total, min_total, fair_total, fnz
            );
            return HaggleOutcome::Reject(format!("Rejected: too low (<{:.0})", min_total));
        }

        if ask_total as f32 >= fair_total {
            debug!(
                "Accept sell: ask {} >= fair {:.0} (round {}/{})",
                ask_total, fair_total, session.round, session.max_rounds
            );
            return match self.execute_sell(session, ask_total) {
                Ok(msg) => HaggleOutcome::Accept(msg),
                Err(e) => HaggleOutcome::Reject(e),
            };
        }

        if session.round >= session.max_rounds {
            debug!("Reject sell: underpay on final round");
            return HaggleOutcome::Reject("They pass after final round.".to_string());
        }

        let new_price = 0.8 * session.price_each + 0.2 * (ask_total as f32 / session.qty as f32);
        session.price_each = new_price;
        session.round += 1;
        debug!(
            "Counter sell: new price_each {:.2}, ask {}, fair {:.2}, round {}/{}",
            new_price, ask_total, fair_total, session.round, session.max_rounds
        );
        HaggleOutcome::Counter(
            session,
            format!(
                "Countered; their bid about ${:.0}",
                new_price * session.qty as f32
            ),
        )
    }

    fn execute_buy(&mut self, session: TradeSession, total: i32) -> Result<String, String> {
        let ship = self
            .ships
            .get_mut(session.ship_id)
            .ok_or_else(|| "Bad ship".to_string())?;
        let system = self
            .systems
            .get_mut(session.system_id)
            .ok_or_else(|| "Bad system".to_string())?;

        if system.stock[session.good_idx] < session.qty {
            return Err("Not enough stock".to_string());
        }
        let mut weight = ship.net_tonnage;
        if session.good_idx < 4 {
            weight += session.qty;
            if weight > self.max_tonnage {
                return Err("Exceeds tonnage limit".to_string());
            }
        }
        let cost = total as i64;
        if ship.cash < cost {
            return Err("Not enough cash".to_string());
        }

        ship.cash -= cost;
        ship.cargo[session.good_idx] += session.qty;
        if session.good_idx < 4 {
            ship.net_tonnage = weight;
        }
        system.stock[session.good_idx] -= session.qty;

        Ok(format!(
            "Bought {} {} for ${} (ask was ${:.0})",
            session.qty,
            Merchandise::ALL[session.good_idx].code(),
            cost,
            session.price_each * session.qty as f32
        ))
    }

    fn execute_sell(&mut self, session: TradeSession, total: i32) -> Result<String, String> {
        let ship = self
            .ships
            .get_mut(session.ship_id)
            .ok_or_else(|| "Bad ship".to_string())?;
        let system = self
            .systems
            .get_mut(session.system_id)
            .ok_or_else(|| "Bad system".to_string())?;

        if ship.cargo[session.good_idx] < session.qty {
            return Err("Not enough cargo".to_string());
        }
        let max_buy = -system.stock[session.good_idx] * 2;
        if session.qty > max_buy {
            return Err(format!("They'll consider up to {} units", max_buy));
        }

        let revenue = total as i64;
        ship.cargo[session.good_idx] -= session.qty;
        if session.good_idx < 4 {
            ship.net_tonnage -= session.qty;
        }
        system.stock[session.good_idx] += session.qty;
        ship.cash += revenue;

        Ok(format!(
            "Sold {} {} for ${} (bid was ${:.0})",
            session.qty,
            Merchandise::ALL[session.good_idx].code(),
            revenue,
            session.price_each * session.qty as f32
        ))
    }

    fn ship_at_system(&self, ship_id: usize, system_id: usize) -> bool {
        self.ships
            .get(ship_id)
            .map(|s| s.location == system_id)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn buy(
        &mut self,
        ship_id: usize,
        system_id: usize,
        good_idx: usize,
        qty: i32,
    ) -> Result<String, String> {
        if qty <= 0 {
            return Err("Quantity must be positive".to_string());
        }
        if system_id >= self.systems.len() || ship_id >= self.ships.len() {
            return Err("Invalid selection".to_string());
        }
        let price_each = self.systems[system_id].prices[good_idx];
        if price_each <= 0 {
            return Err("Item not for sale".to_string());
        }

        // Trade must occur where the ship is located.
        if self.ships[ship_id].location != system_id {
            return Err("Ship is not at this system".to_string());
        }

        if self.systems[system_id].stock[good_idx] < qty {
            return Err("Not enough stock".to_string());
        }

        let mut weight = self.ships[ship_id].net_tonnage;
        if good_idx < 4 {
            weight += qty;
            if weight > self.max_tonnage {
                return Err("Exceeds tonnage limit".to_string());
            }
        }

        let cost = price_each as i64 * qty as i64;
        if self.ships[ship_id].cash < cost {
            return Err("Not enough cash".to_string());
        }

        self.ships[ship_id].cash -= cost;
        self.ships[ship_id].cargo[good_idx] += qty;
        if good_idx < 4 {
            self.ships[ship_id].net_tonnage = weight;
        }
        self.systems[system_id].stock[good_idx] -= qty;
        Ok(format!(
            "Bought {} {} for ${}",
            qty,
            Merchandise::ALL[good_idx].code(),
            cost
        ))
    }

    #[allow(dead_code)]
    pub fn sell(
        &mut self,
        ship_id: usize,
        system_id: usize,
        good_idx: usize,
        qty: i32,
    ) -> Result<String, String> {
        if qty <= 0 {
            return Err("Quantity must be positive".to_string());
        }
        if system_id >= self.systems.len() || ship_id >= self.ships.len() {
            return Err("Invalid selection".to_string());
        }
        if self.ships[ship_id].location != system_id {
            return Err("Ship is not at this system".to_string());
        }

        if self.ships[ship_id].cargo[good_idx] < qty {
            return Err("Not enough cargo".to_string());
        }

        // System should be demanding this good (stock < 0).
        if self.systems[system_id].stock[good_idx] >= 0 {
            return Err("System not buying this good".to_string());
        }
        let max_buy = -self.systems[system_id].stock[good_idx] * 2; // as in BASIC
        if qty > max_buy {
            return Err(format!("They'll consider up to {} units", max_buy));
        }

        let price_each = self.systems[system_id].prices[good_idx];
        let revenue = price_each as i64 * qty as i64;

        self.ships[ship_id].cargo[good_idx] -= qty;
        if good_idx < 4 {
            self.ships[ship_id].net_tonnage -= qty;
        }
        self.systems[system_id].stock[good_idx] += qty;
        self.ships[ship_id].cash += revenue;
        Ok(format!(
            "Sold {} {} for ${}",
            qty,
            Merchandise::ALL[good_idx].code(),
            revenue
        ))
    }

    pub fn deposit(&mut self, ship_id: usize, amount: i64) -> Result<String, String> {
        if amount <= 0 {
            return Err("Amount must be positive".to_string());
        }
        if ship_id >= self.ships.len() {
            return Err("Invalid ship".to_string());
        }
        let loc = self.ships[ship_id].location;
        if !Self::has_bank(&self.systems[loc]) {
            return Err("No bank at this system".to_string());
        }
        if self.ships[ship_id].cash < amount {
            return Err("Not enough cash on ship".to_string());
        }
        self.accrue_interest(self.date);
        self.ships[ship_id].cash -= amount;
        self.bank_balance += amount;
        Ok(format!("Deposited ${}", amount))
    }

    pub fn withdraw(&mut self, ship_id: usize, amount: i64) -> Result<String, String> {
        if amount <= 0 {
            return Err("Amount must be positive".to_string());
        }
        if ship_id >= self.ships.len() {
            return Err("Invalid ship".to_string());
        }
        let loc = self.ships[ship_id].location;
        if !Self::has_bank(&self.systems[loc]) {
            return Err("No bank at this system".to_string());
        }
        self.accrue_interest(self.date);
        if self.bank_balance < amount {
            return Err("Not enough in bank".to_string());
        }
        self.bank_balance -= amount;
        self.ships[ship_id].cash += amount;
        Ok(format!("Withdrew ${}", amount))
    }

    pub fn autopilot_demo_routes(&mut self) {
        let dest = if self.systems.len() > 1 { 1 } else { 0 };
        for ship_id in 0..self.ships.len() {
            self.schedule_route(ship_id, dest);
        }
    }

    pub fn schedule_route(&mut self, ship_id: usize, destination: usize) {
        if let Some(ship) = self.ships.get_mut(ship_id) {
            if destination >= self.systems.len() {
                return;
            }
            if ship.location == destination {
                ship.destination = destination;
                ship.eta = self.date;
                ship.delay_weeks = 0;
                debug!(
                    "Route noop: ship {} already at system {}",
                    ship_id, destination
                );
                return;
            }

            let origin_coords = self.systems[ship.location].coords;
            let dest_coords = self.systems[destination].coords;
            let dx = f32::from(dest_coords.0 - origin_coords.0);
            let dy = f32::from(dest_coords.1 - origin_coords.1);
            let distance = (dx * dx + dy * dy).sqrt();
            let base_days = (distance / self.speed_ly_per_day).floor() as u16;

            let mut eta = self.date.add_days(base_days);

            let mut total_delay_weeks = 0u8;
            if self.rng.r#gen::<f32>() <= self.delay_probability / 2.0 {
                let weeks = 1 + self.rng.gen_range(0..3);
                total_delay_weeks += weeks as u8;
                eta = eta.add_days(7 * weeks as u16);
            }

            if self.rng.r#gen::<f32>() <= self.delay_probability / 2.0 {
                let weeks = 1 + self.rng.gen_range(0..3);
                total_delay_weeks += weeks as u8;
                eta = eta.add_days(7 * weeks as u16);
            }

            ship.destination = destination;
            ship.eta = eta;
            ship.delay_weeks = total_delay_weeks;

            debug!(
                "Route ship {}: {} -> {} dist {:.2} ly, base {}d, delays {}w, ETA d{} y{}",
                ship_id,
                ship.location,
                destination,
                distance,
                base_days,
                total_delay_weeks,
                ship.eta.day,
                ship.eta.year
            );
        }
    }

    pub fn route_selected_ship_to_selected_system(&mut self) -> String {
        if self.ships.is_empty() || self.systems.is_empty() {
            return "No ships or systems to route.".to_string();
        }

        let ship_idx = self.selected_ship.min(self.ships.len() - 1);
        let dest_idx = self.selected_system.min(self.systems.len() - 1);

        let at_name = self.systems[self.ships[ship_idx].location].name.clone();
        let dest_name = self.systems[dest_idx].name.clone();

        if self.ships[ship_idx].location == dest_idx {
            return format!("{} is already at {}", self.ships[ship_idx].name, at_name);
        }

        self.schedule_route(ship_idx, dest_idx);
        let ship = &self.ships[ship_idx];
        let mut note = String::new();
        if ship.delay_weeks > 0 {
            note = format!(" ({} wk delay factored)", ship.delay_weeks);
        }

        format!(
            "{} set course to {} — ETA day {}, year {}{}",
            ship.name, dest_name, ship.eta.day, ship.eta.year, note
        )
    }

    fn accrue_interest(&mut self, now: GameDate) {
        let days_now = now.year as f32 * 360.0 + now.day as f32;
        let days_last = self.bank_last_date.year as f32 * 360.0 + self.bank_last_date.day as f32;
        let delta_years = ((days_now - days_last) / 360.0).max(0.0);
        if delta_years > 0.0 && self.bank_balance > 0 {
            let interest = (self.bank_balance as f32) * 0.05 * delta_years;
            self.bank_balance += interest.round() as i64;
        }
        self.bank_last_date = now;
    }

    fn has_bank(system: &StarSystem) -> bool {
        matches!(system.class, StarClass::I | StarClass::II)
    }

    pub fn advance_to_next_arrival(&mut self) -> Option<String> {
        if self.ships.is_empty() {
            return None;
        }

        let next_eta = self.ships.iter().map(|s| s.eta).min()?;

        debug!(
            "Advance time to arrivals at day {} year {}",
            next_eta.day, next_eta.year
        );

        self.date = next_eta;

        let arriving: Vec<usize> = self
            .ships
            .iter()
            .enumerate()
            .filter(|(_, ship)| ship.eta == next_eta)
            .map(|(idx, _)| idx)
            .collect();

        debug!("Arriving ship ids: {:?}", arriving);

        let mut arrival_msgs = Vec::new();
        for ship_id in arriving {
            let (system_id, delay_weeks, ship_name) = {
                let ship = &mut self.ships[ship_id];
                ship.location = ship.destination;
                (ship.location, ship.delay_weeks, ship.name.clone())
            };

            self.update_system_for_date(system_id, self.date);
            self.increment_development(system_id);

            let system_name = self.systems[system_id].name.clone();
            let delay_note = if delay_weeks > 0 {
                format!(" ({} wks late)", delay_weeks)
            } else {
                String::new()
            };

            debug!(
                "Ship {} arrived at system {} ({}), delay {}w",
                ship_id, system_id, system_name, delay_weeks
            );
            arrival_msgs.push(format!(
                "{} arrived at {}{}",
                ship_name, system_name, delay_note
            ));
        }

        if let Some(spawn_msg) = self.maybe_spawn_new_star() {
            debug!("New star spawned: {}", spawn_msg);
            arrival_msgs.push(spawn_msg);
        }

        if arrival_msgs.is_empty() {
            None
        } else {
            Some(arrival_msgs.join(" | "))
        }
    }

    pub fn reprice_all(&mut self) {
        let date = self.date;
        for id in 0..self.systems.len() {
            self.update_system_for_date(id, date);
        }
    }

    fn update_system_for_date(&mut self, system_id: usize, date: GameDate) {
        let Some(system) = self.systems.get_mut(system_id) else {
            return;
        };

        let months = system.last_update.months_until(date);
        let class_band = Self::class_band(system.dev_score);
        for i in 0..6 {
            let g = (1.0 + system.dev_score / 15.0)
                * (self.price_model.m[i][class_band] * system.dev_score
                    + self.price_model.c[i][class_band]);

            if g.abs() <= 0.01 {
                system.prices[i] = 0;
                continue;
            }

            let projected = system.stock[i] as f32 + months * g;
            let cap = (g * 12.0).abs();
            let new_mag = projected.abs().min(cap);
            system.stock[i] = g.signum() as i32 * new_mag.round() as i32;

            let denom = (g * self.price_model.base_profit_margin).abs().max(0.0001);
            let price_factor =
                1.0 - (system.stock[i] as f32).signum() * (system.stock[i].abs() as f32 / denom);
            let mut price = (self.price_model.base_prices[i] as f32 * price_factor).round() as i32;
            price = ((price + 50) / 100) * 100;
            system.prices[i] = price.max(0);
        }

        system.last_update = date;
        system.class = StarClass::from_score(system.dev_score.round() as i16);
    }

    fn increment_development(&mut self, system_id: usize) {
        if let Some(system) = self.systems.get_mut(system_id)
            && system.dev_score < 15.0
        {
            system.dev_score = (system.dev_score + self.development_increment).min(15.0);
            system.class = StarClass::from_score(system.dev_score.round() as i16);
        }
    }

    fn maybe_spawn_new_star(&mut self) -> Option<String> {
        if self.systems.len() >= MAX_SYSTEMS {
            return None;
        }

        let avg_class: f32 = self
            .systems
            .iter()
            .map(|s| s.class.raw_score() as f32)
            .sum::<f32>()
            / (self.systems.len() as f32).max(1.0);

        if avg_class < NEW_STAR_THRESHOLD {
            return None;
        }

        let id = self.systems.len();
        let name = format!("FRONTIER-{}", id + 1);

        let radius = 30.0 + (id as f32 * 3.5);
        let angle = (id as f32 * 1.618) * std::f32::consts::PI;
        let x = (radius * angle.cos()).round() as i16;
        let y = (radius * angle.sin()).round() as i16;

        let mut stock = [0i32; 6];
        for slot in stock.iter_mut() {
            let val = self.rng.gen_range(-6..=6);
            *slot = if val == 0 { 1 } else { val };
        }

        let system = StarSystem {
            id,
            name,
            class: StarClass::IV,
            dev_score: 3.0,
            coords: (x, y),
            stock,
            last_update: self.date,
            prices: [0; 6],
        };

        self.systems.push(system);
        self.update_system_for_date(id, self.date);

        Some(format!(
            "New frontier system discovered: {} at ({},{})",
            self.systems[id].name, x, y
        ))
    }

    #[allow(dead_code)]
    fn acceptance_window_buy(&self, system_id: usize, good_idx: usize, qty: i32) -> f32 {
        let stock = self.systems[system_id].stock[good_idx].abs().max(1) as f32;
        let base = 0.25 + (qty as f32 / (2.0 * stock));
        base.min(0.6)
    }

    #[allow(dead_code)]
    fn acceptance_window_sell(&self, system_id: usize, good_idx: usize, qty: i32) -> f32 {
        let stock = self.systems[system_id].stock[good_idx].abs().max(1) as f32;
        let base = 0.2 + (qty as f32 / (3.0 * stock));
        base.min(0.5)
    }

    fn fnz(&self, qty: i32, system_id: usize, good_idx: usize, round: usize) -> f32 {
        let stock_mag = self.systems[system_id].stock[good_idx].abs().max(1) as f32;
        let fny = qty as f32 >= stock_mag;
        let base = if fny {
            0.5
        } else {
            (qty as f32) / (2.0 * stock_mag)
        };
        base / round.max(1) as f32
    }

    fn class_band(dev_score: f32) -> usize {
        if dev_score >= 10.0 {
            2
        } else if dev_score >= 5.0 {
            1
        } else {
            0
        }
    }
}

impl PartialEq for GameState {
    fn eq(&self, other: &Self) -> bool {
        self.date == other.date
            && self.systems == other.systems
            && self.ships == other.ships
            && self.selected_system == other.selected_system
            && self.selected_ship == other.selected_ship
            && self.selected_good == other.selected_good
            && self.price_model == other.price_model
            && self.max_tonnage == other.max_tonnage
            && (self.speed_ly_per_day - other.speed_ly_per_day).abs() < f32::EPSILON
            && (self.delay_probability - other.delay_probability).abs() < f32::EPSILON
            && (self.development_increment - other.development_increment).abs() < f32::EPSILON
            && self.bank_balance == other.bank_balance
            && self.bank_last_date == other.bank_last_date
            && self.max_bid_rounds == other.max_bid_rounds
            && serde_json::to_vec(&self.rng).ok() == serde_json::to_vec(&other.rng).ok()
    }
}

impl GameDate {
    pub fn add_days(self, days: u16) -> Self {
        let mut total_days = self.day as i32 + days as i32;
        let mut year = self.year;
        while total_days > 360 {
            total_days -= 360;
            year += 1;
        }
        Self {
            day: total_days as u16,
            year,
        }
    }

    pub fn months_until(self, other: GameDate) -> f32 {
        let lhs = self.year as f32 * 360.0 + self.day as f32;
        let rhs = other.year as f32 * 360.0 + other.day as f32;
        ((rhs - lhs) / 30.0).max(0.0)
    }
}

impl PartialEq for GameDate {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year && self.day == other.day
    }
}

impl Eq for GameDate {}

impl PartialOrd for GameDate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for GameDate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.year.cmp(&other.year) {
            std::cmp::Ordering::Equal => self.day.cmp(&other.day),
            ord => ord,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GameState;

    #[test]
    fn demo_world_sets_prices_and_routes() {
        let state = GameState::demo_with_seed(42);

        assert_eq!(state.systems.len(), 2);
        assert_eq!(state.ships.len(), 2);

        for system in &state.systems {
            assert!(
                system.prices.iter().all(|price| *price >= 0),
                "{} has negative prices: {:?}",
                system.name,
                system.prices
            );
            assert!(
                system.prices.iter().any(|price| *price > 0),
                "{} should have at least one priced good",
                system.name
            );
        }

        for ship in &state.ships {
            assert!(ship.destination < state.systems.len());
            assert!(ship.eta >= state.date);
        }
    }
}
