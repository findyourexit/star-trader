use std::fs::{File, read_to_string};
use std::io::{Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use log::{info, warn};
use serde::{Deserialize, Serialize};

use crate::model::{GameState, HaggleOutcome, Merchandise, TradeKind, TradeSession};

const EVENT_LOG_LIMIT: usize = 6;
const HIGHLIGHT_TICKS: u64 = 14;
const LAUNCH_SCREEN_DURATION: Duration = Duration::from_secs(2);
const SAVE_PATH: &str = "savegame.json";
const KEYBINDS_PATH: &str = "keybindings.json";

#[derive(Debug, Clone, Deserialize, Default)]
struct KeyBindingsConfig {
    quit: Option<String>,
    reset: Option<String>,
    focus_systems: Option<String>,
    focus_market: Option<String>,
    focus_ships: Option<String>,
    prev_good: Option<String>,
    next_good: Option<String>,
    set_course: Option<String>,
    buy: Option<String>,
    sell: Option<String>,
    deposit: Option<String>,
    withdraw: Option<String>,
    set_destination: Option<String>,
    pick_destination: Option<String>,
    next_arrival: Option<String>,
    map: Option<String>,
    report: Option<String>,
    help: Option<String>,
    help_alt: Option<String>,
    save: Option<String>,
    load: Option<String>,
}

#[derive(Debug, Clone)]
struct KeyBindings {
    quit: KeyCode,
    reset: KeyCode,
    focus_systems: KeyCode,
    focus_market: KeyCode,
    focus_ships: KeyCode,
    prev_good: KeyCode,
    next_good: KeyCode,
    set_course: KeyCode,
    buy: KeyCode,
    sell: KeyCode,
    deposit: KeyCode,
    withdraw: KeyCode,
    set_destination: KeyCode,
    pick_destination: KeyCode,
    next_arrival: KeyCode,
    map: KeyCode,
    report: KeyCode,
    help: KeyCode,
    help_alt: KeyCode,
    save: KeyCode,
    load: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: KeyCode::Char('q'),
            reset: KeyCode::Char('r'),
            focus_systems: KeyCode::Char('1'),
            focus_market: KeyCode::Char('2'),
            focus_ships: KeyCode::Char('3'),
            prev_good: KeyCode::Char('['),
            next_good: KeyCode::Char(']'),
            set_course: KeyCode::Char('g'),
            buy: KeyCode::Char('b'),
            sell: KeyCode::Char('s'),
            deposit: KeyCode::Char('k'),
            withdraw: KeyCode::Char('l'),
            set_destination: KeyCode::Char('p'),
            pick_destination: KeyCode::Char('d'),
            next_arrival: KeyCode::Char('n'),
            map: KeyCode::Char('m'),
            report: KeyCode::Char('t'),
            help: KeyCode::Char('?'),
            help_alt: KeyCode::Char('h'),
            save: KeyCode::Char('v'),
            load: KeyCode::Char('o'),
        }
    }
}

impl KeyBindings {
    fn parse_key(label: &str) -> Result<KeyCode, String> {
        let lower = label.trim().to_lowercase();
        match lower.as_str() {
            "up" => Ok(KeyCode::Up),
            "down" => Ok(KeyCode::Down),
            "left" => Ok(KeyCode::Left),
            "right" => Ok(KeyCode::Right),
            "esc" | "escape" => Ok(KeyCode::Esc),
            "tab" => Ok(KeyCode::Tab),
            "backtab" | "shift-tab" => Ok(KeyCode::BackTab),
            s if s.len() == 1 => Ok(KeyCode::Char(s.chars().next().unwrap())),
            _ => Err(format!("Unrecognized key '{label}'")),
        }
    }

    fn from_config(cfg: KeyBindingsConfig) -> Result<Self, String> {
        let mut kb = KeyBindings::default();

        let apply = |slot: &mut KeyCode, val: Option<String>| -> Result<(), String> {
            if let Some(v) = val {
                *slot = Self::parse_key(&v)?;
            }
            Ok(())
        };

        apply(&mut kb.quit, cfg.quit)?;
        apply(&mut kb.reset, cfg.reset)?;
        apply(&mut kb.focus_systems, cfg.focus_systems)?;
        apply(&mut kb.focus_market, cfg.focus_market)?;
        apply(&mut kb.focus_ships, cfg.focus_ships)?;
        apply(&mut kb.prev_good, cfg.prev_good)?;
        apply(&mut kb.next_good, cfg.next_good)?;
        apply(&mut kb.set_course, cfg.set_course)?;
        apply(&mut kb.buy, cfg.buy)?;
        apply(&mut kb.sell, cfg.sell)?;
        apply(&mut kb.deposit, cfg.deposit)?;
        apply(&mut kb.withdraw, cfg.withdraw)?;
        apply(&mut kb.set_destination, cfg.set_destination)?;
        apply(&mut kb.pick_destination, cfg.pick_destination)?;
        apply(&mut kb.next_arrival, cfg.next_arrival)?;
        apply(&mut kb.map, cfg.map)?;
        apply(&mut kb.report, cfg.report)?;
        apply(&mut kb.help, cfg.help)?;
        apply(&mut kb.help_alt, cfg.help_alt)?;
        apply(&mut kb.save, cfg.save)?;
        apply(&mut kb.load, cfg.load)?;

        Ok(kb)
    }

    fn load(path: &str) -> (Self, Option<String>) {
        match read_to_string(path) {
            Ok(text) => match serde_json::from_str::<KeyBindingsConfig>(&text) {
                Ok(cfg) => match Self::from_config(cfg) {
                    Ok(kb) => (kb, Some(format!("Loaded keybindings from {path}"))),
                    Err(e) => (
                        KeyBindings::default(),
                        Some(format!("Keybindings fallback to defaults: {e}")),
                    ),
                },
                Err(e) => (
                    KeyBindings::default(),
                    Some(format!("Keybindings parse failed: {e}")),
                ),
            },
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (KeyBindings::default(), None),
            Err(e) => (
                KeyBindings::default(),
                Some(format!("Keybindings load failed: {e}")),
            ),
        }
    }

    fn matches_help(&self, key: KeyCode) -> bool {
        key == self.help || key == self.help_alt
    }

    fn label(&self, key: KeyCode) -> String {
        match key {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::BackTab => "Shift+Tab".to_string(),
            other => format!("{other:?}"),
        }
    }

    fn pair_label(&self, a: KeyCode, b: KeyCode) -> String {
        format!("{}/{}", self.label(a), self.label(b))
    }

    fn triple_label(&self, a: KeyCode, b: KeyCode, c: KeyCode) -> String {
        format!("{}/{}/{}", self.label(a), self.label(b), self.label(c))
    }

    fn footer_hints(&self) -> Vec<(String, String)> {
        vec![
            (
                self.triple_label(self.focus_systems, self.focus_market, self.focus_ships),
                "focus panels".to_string(),
            ),
            ("arrows".to_string(), "move active".to_string()),
            (
                self.pair_label(self.prev_good, self.next_good),
                "cycle goods".to_string(),
            ),
            (self.label(self.set_course), "set course".to_string()),
            (self.pair_label(self.buy, self.sell), "trade".to_string()),
            (
                self.pair_label(self.deposit, self.withdraw),
                "bank".to_string(),
            ),
            (
                format!("{} #", self.label(self.set_destination)),
                "jump system".to_string(),
            ),
            (self.label(self.pick_destination), "pick dest".to_string()),
            (self.label(self.next_arrival), "next arrival".to_string()),
            (self.label(self.map), "map".to_string()),
            (self.label(self.report), "report".to_string()),
            (
                self.pair_label(self.help, self.help_alt),
                "help".to_string(),
            ),
            (self.label(self.save), "save".to_string()),
            (self.label(self.load), "load".to_string()),
            (self.label(self.reset), "reset".to_string()),
            (self.label(self.quit), "quit".to_string()),
        ]
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
struct Highlight {
    until: u64,
    delta: i64,
}

impl Highlight {
    fn new(until: u64, delta: i64) -> Self {
        Self { until, delta }
    }

    fn active(&self, now: u64) -> bool {
        now <= self.until && self.delta != 0
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActivePanel {
    Systems,
    Market,
    Ships,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InputMode {
    Buy,
    Sell,
    Deposit,
    Withdraw,
    SetDestination,
    PickDestination,
    TradePrice,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Overlay {
    Map,
    Report,
    Help,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Screen {
    Launch,
    Menu,
    Game,
    Instructions,
    About,
}

impl Screen {
    fn game() -> Self {
        Screen::Game
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen::Launch
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MenuItem {
    NewGame,
    LoadGame,
    Instructions,
    About,
    Quit,
}

impl MenuItem {
    pub fn label(&self) -> &'static str {
        match self {
            MenuItem::NewGame => "New Game",
            MenuItem::LoadGame => "Load Game",
            MenuItem::Instructions => "Instructions",
            MenuItem::About => "About",
            MenuItem::Quit => "Quit",
        }
    }

    pub fn detail(&self) -> &'static str {
        match self {
            MenuItem::NewGame => "Start a fresh demo run",
            MenuItem::LoadGame => "Resume the last save",
            MenuItem::Instructions => "Controls and gameplay tips",
            MenuItem::About => "Credits and version info",
            MenuItem::Quit => "Exit Star Trader",
        }
    }
}

const MENU_ITEMS: [MenuItem; 5] = [
    MenuItem::NewGame,
    MenuItem::LoadGame,
    MenuItem::Instructions,
    MenuItem::About,
    MenuItem::Quit,
];

#[derive(Debug, Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    app: SavedApp,
}

#[derive(Debug, Serialize, Deserialize)]
struct SavedApp {
    state: GameState,
    ticks: u64,
    status: String,
    events: Vec<String>,
    #[serde(default = "Screen::game")]
    screen: Screen,
    #[serde(default)]
    menu_index: usize,
    active_panel: ActivePanel,
    input_mode: Option<InputMode>,
    input_buffer: String,
    overlay: Option<Overlay>,
    bank_hi: Highlight,
    ship_cash_hi: Vec<Highlight>,
    ship_ton_hi: Vec<Highlight>,
    cargo_hi: Vec<Vec<Highlight>>,
    stock_hi: Vec<Vec<Highlight>>,
    price_hi: Vec<Vec<Highlight>>,
}

impl SavedApp {
    fn from_app(app: &App) -> Self {
        Self {
            state: app.state.clone(),
            ticks: app.ticks,
            status: app.status.clone(),
            events: app.events.clone(),
            screen: app.screen,
            menu_index: app.menu_index,
            active_panel: app.active_panel,
            input_mode: app.input_mode,
            input_buffer: app.input_buffer.clone(),
            overlay: app.overlay,
            bank_hi: app.bank_hi,
            ship_cash_hi: app.ship_cash_hi.clone(),
            ship_ton_hi: app.ship_ton_hi.clone(),
            cargo_hi: app.cargo_hi.clone(),
            stock_hi: app.stock_hi.clone(),
            price_hi: app.price_hi.clone(),
        }
    }

    fn into_app(self) -> App {
        let mut app = App {
            state: self.state,
            quit: false,
            ticks: self.ticks,
            status: self.status,
            input_mode: self.input_mode,
            input_buffer: self.input_buffer,
            pending_trade: None,
            overlay: self.overlay,
            prev_state: None,
            events: self.events,
            bank_hi: self.bank_hi,
            ship_cash_hi: self.ship_cash_hi,
            ship_ton_hi: self.ship_ton_hi,
            cargo_hi: self.cargo_hi,
            stock_hi: self.stock_hi,
            price_hi: self.price_hi,
            screen: self.screen,
            menu_index: self.menu_index,
            active_panel: self.active_panel,
            keybinds: KeyBindings::default(),
            launch_started: Instant::now(),
        };

        app.prev_state = Some(app.state.clone());
        app
    }
}

#[derive(Debug)]
pub struct App {
    pub state: GameState,
    quit: bool,
    ticks: u64,
    status: String,
    pub input_mode: Option<InputMode>,
    pub input_buffer: String,
    pending_trade: Option<TradeContext>,
    overlay: Option<Overlay>,
    prev_state: Option<GameState>,
    events: Vec<String>,
    bank_hi: Highlight,
    ship_cash_hi: Vec<Highlight>,
    ship_ton_hi: Vec<Highlight>,
    cargo_hi: Vec<Vec<Highlight>>, // ship -> goods
    stock_hi: Vec<Vec<Highlight>>, // system -> goods
    price_hi: Vec<Vec<Highlight>>, // system -> goods
    screen: Screen,
    menu_index: usize,
    pub active_panel: ActivePanel,
    keybinds: KeyBindings,
    launch_started: Instant,
}

#[derive(Debug, Clone, Copy)]
pub struct TradeContext {
    pub session: TradeSession,
}

impl TradeContext {
    pub fn from_session(session: TradeSession) -> Self {
        Self { session }
    }
}

impl Default for App {
    fn default() -> Self {
        let state = GameState::demo();
        let (ship_cash_hi, ship_ton_hi, cargo_hi, stock_hi, price_hi) =
            Self::init_highlights(&state);
        Self {
            state,
            quit: false,
            ticks: 0,
            status: "Launching Star Trader...".to_string(),
            input_mode: None,
            input_buffer: String::new(),
            pending_trade: None,
            overlay: None,
            prev_state: None,
            events: Vec::new(),
            bank_hi: Highlight::default(),
            ship_cash_hi,
            ship_ton_hi,
            cargo_hi,
            stock_hi,
            price_hi,
            screen: Screen::Launch,
            menu_index: 0,
            active_panel: ActivePanel::Systems,
            keybinds: KeyBindings::default(),
            launch_started: Instant::now(),
        }
    }
}

impl App {
    pub fn boot() -> Self {
        let (keybinds, kb_notice) = KeyBindings::load(KEYBINDS_PATH);

        let mut app = App::default();
        app.keybinds = keybinds;

        if Path::new(SAVE_PATH).exists() {
            app.status = "Main menu - save found; choose Load Game to resume.".to_string();
        }

        if let Some(msg) = kb_notice {
            if msg.contains("failed") || msg.contains("fallback") {
                app.status = msg.clone();
                warn!("{msg}");
            } else {
                info!("{msg}");
            }
            app.log_event(msg);
        }

        app
    }

    pub fn handle_key(&mut self, key: KeyCode) {
        match self.screen {
            Screen::Launch => {
                self.finish_launch();
                return;
            }
            Screen::Menu => {
                self.handle_menu_key(key);
                return;
            }
            Screen::Instructions | Screen::About => {
                if self.handle_static_screen_key(key) {
                    return;
                }
                return;
            }
            Screen::Game => {}
        }

        if self.input_mode.is_some() {
            self.handle_input_mode(key);
            return;
        }

        if self.overlay.is_some() {
            if self.handle_overlay_key(key) {
                return;
            }
            // Overlay is modal; ignore other keys while active.
            return;
        }
        let kb = &self.keybinds;

        match key {
            KeyCode::Up => match self.active_panel {
                ActivePanel::Systems => self.state.cycle_system(-1),
                ActivePanel::Market => self.state.cycle_good(-1),
                ActivePanel::Ships => self.state.cycle_ship(-1),
            },
            KeyCode::Down => match self.active_panel {
                ActivePanel::Systems => self.state.cycle_system(1),
                ActivePanel::Market => self.state.cycle_good(1),
                ActivePanel::Ships => self.state.cycle_ship(1),
            },
            KeyCode::Left => match self.active_panel {
                ActivePanel::Systems => self.state.cycle_system(-1),
                ActivePanel::Market => self.state.cycle_good(-1),
                ActivePanel::Ships => self.state.cycle_ship(-1),
            },
            KeyCode::Right => match self.active_panel {
                ActivePanel::Systems => self.state.cycle_system(1),
                ActivePanel::Market => self.state.cycle_good(1),
                ActivePanel::Ships => self.state.cycle_ship(1),
            },
            code if code == kb.prev_good => self.state.cycle_good(-1),
            code if code == kb.next_good => self.state.cycle_good(1),
            code if code == kb.focus_systems => {
                self.active_panel = ActivePanel::Systems;
                self.status = "Focused Star Systems".to_string();
            }
            code if code == kb.focus_market => {
                self.active_panel = ActivePanel::Market;
                self.status = "Focused Market".to_string();
            }
            code if code == kb.focus_ships => {
                self.active_panel = ActivePanel::Ships;
                self.status = "Focused Ships".to_string();
            }
            code if code == kb.set_course => {
                let msg = self.state.route_selected_ship_to_selected_system();
                self.status = msg.clone();
                self.log_event(msg);
                info!("{}", self.status);
            }
            code if code == kb.buy => {
                self.begin_input(InputMode::Buy, self.buy_qty_prompt());
            }
            code if code == kb.sell => {
                self.begin_input(InputMode::Sell, self.sell_qty_prompt());
            }
            code if code == kb.deposit => {
                self.begin_input(InputMode::Deposit, self.deposit_prompt());
            }
            code if code == kb.withdraw => {
                self.begin_input(InputMode::Withdraw, self.withdraw_prompt());
            }
            code if code == kb.set_destination => {
                self.begin_input(InputMode::SetDestination, self.destination_prompt());
            }
            code if code == kb.pick_destination => {
                self.input_mode = Some(InputMode::PickDestination);
                self.status =
                    "Pick destination with arrows, Enter to set, Esc to cancel.".to_string();
            }
            code if code == kb.next_arrival => {
                if let Some(msg) = self.state.advance_to_next_arrival() {
                    self.status = msg.clone();
                    self.log_event(msg);
                    info!("{}", self.status);
                } else {
                    self.status = "No scheduled arrivals.".to_string();
                }
            }
            code if code == kb.map => self.open_overlay(Overlay::Map, "Map overlay — Esc to close"),
            code if code == kb.report => {
                self.open_overlay(Overlay::Report, "Trade report — Esc to close")
            }
            code if kb.matches_help(code) => {
                self.open_overlay(Overlay::Help, "Help — Esc to close")
            }
            code if code == kb.save => match self.save_to_path(SAVE_PATH) {
                Ok(msg) => {
                    self.status = msg.clone();
                    self.log_event(msg);
                    info!("{}", self.status);
                }
                Err(e) => {
                    self.status = e.clone();
                    self.log_event(e);
                    warn!("{}", self.status);
                }
            },
            code if code == kb.load => {
                if let Err(e) = self.load_game_from_disk() {
                    self.status = e.clone();
                    self.log_event(e);
                    warn!("{}", self.status);
                }
            }
            code if code == kb.reset => {
                self.reset_demo_state();
            }
            code if code == kb.quit => {
                self.request_quit();
            }
            _ => {}
        }
    }

    fn handle_menu_key(&mut self, key: KeyCode) {
        let len = MENU_ITEMS.len();
        match key {
            KeyCode::Up => {
                self.menu_index = self.menu_index.saturating_add(len - 1) % len;
            }
            KeyCode::Down => {
                self.menu_index = (self.menu_index + 1) % len;
            }
            KeyCode::Enter => {
                if let Some(item) = MENU_ITEMS.get(self.menu_index).copied() {
                    self.activate_menu_item(item);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                self.request_quit();
            }
            _ => {}
        }

        if let Some(item) = MENU_ITEMS.get(self.menu_index) {
            self.status = format!("{} - {}", item.label(), item.detail());
        }
    }

    fn handle_static_screen_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Enter => {
                self.enter_menu();
                true
            }
            KeyCode::Char('q') => {
                self.request_quit();
                true
            }
            _ => false,
        }
    }

    fn enter_menu(&mut self) {
        self.clear_modals();
        self.screen = Screen::Menu;
        self.menu_index = 0;
        self.status = "Main menu — use Up/Down then Enter.".to_string();
    }

    fn clear_modals(&mut self) {
        self.input_mode = None;
        self.input_buffer.clear();
        self.pending_trade = None;
        self.overlay = None;
    }

    fn activate_menu_item(&mut self, item: MenuItem) {
        match item {
            MenuItem::NewGame => self.start_new_game(),
            MenuItem::LoadGame => {
                if let Err(e) = self.load_game_from_disk() {
                    self.status = e.clone();
                    self.log_event(e);
                    warn!("{}", self.status);
                }
            }
            MenuItem::Instructions => {
                self.clear_modals();
                self.screen = Screen::Instructions;
                self.status = "Instructions - Esc/Enter to return.".to_string();
            }
            MenuItem::About => {
                self.clear_modals();
                self.screen = Screen::About;
                self.status = "About - Esc/Enter to return.".to_string();
            }
            MenuItem::Quit => self.request_quit(),
        }
    }

    pub fn tick(&mut self) {
        self.ticks = self.ticks.saturating_add(1);

        if self.screen == Screen::Launch
            && self.launch_started.elapsed() >= LAUNCH_SCREEN_DURATION
        {
            self.finish_launch();
        }
    }

    pub fn request_quit(&mut self) {
        self.quit = true;
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    fn finish_launch(&mut self) {
        if self.screen == Screen::Launch {
            let preserve_status = if self.status != "Launching Star Trader..." {
                Some(self.status.clone())
            } else {
                None
            };
            self.enter_menu();
            if let Some(status) = preserve_status {
                self.status = status;
            }
        }
    }

    fn start_new_game(&mut self) {
        self.apply_demo_state(
            "New game started. Press ? for help; v save, o load.",
            "New game started",
        );
    }

    pub fn reset_demo_state(&mut self) {
        self.apply_demo_state("Demo state reset.", "Reset demo state");
    }

    fn apply_demo_state(&mut self, status_msg: &str, event_msg: &str) {
        self.state = GameState::demo();
        self.ticks = 0;
        self.clear_modals();
        self.status = status_msg.to_string();
        self.events.clear();
        self.log_event(event_msg);
        info!("{}", self.status);
        let (ship_cash_hi, ship_ton_hi, cargo_hi, stock_hi, price_hi) =
            Self::init_highlights(&self.state);
        self.ship_cash_hi = ship_cash_hi;
        self.ship_ton_hi = ship_ton_hi;
        self.cargo_hi = cargo_hi;
        self.stock_hi = stock_hi;
        self.price_hi = price_hi;
        self.bank_hi = Highlight::default();
        self.prev_state = None;
        self.active_panel = ActivePanel::Systems;
        self.screen = Screen::Game;
    }

    fn load_game_from_disk(&mut self) -> Result<(), String> {
        let (kb_reload, kb_notice) = KeyBindings::load(KEYBINDS_PATH);
        let mut loaded = App::load_from_path(SAVE_PATH)?;
        loaded.keybinds = kb_reload;
        loaded.screen = Screen::Game;
        loaded.menu_index = 0;
        loaded.clear_modals();
        loaded.status = format!("Loaded from {}", SAVE_PATH);
        if let Some(msg) = kb_notice {
            loaded.log_event(msg.clone());
            if msg.contains("failed") || msg.contains("fallback") {
                loaded.status = msg;
                warn!("{}", loaded.status);
            }
        }
        loaded.log_event(loaded.status.clone());
        *self = loaded;
        info!("{}", self.status);
        Ok(())
    }

    fn log_event(&mut self, msg: impl Into<String>) {
        self.events.push(msg.into());
        if self.events.len() > EVENT_LOG_LIMIT {
            self.events.remove(0);
        }
    }

    pub fn update_highlights(&mut self) {
        if let Some(prev) = self.prev_state.take() {
            if self.state.bank_balance != prev.bank_balance {
                self.bank_hi = Highlight::new(
                    self.ticks + HIGHLIGHT_TICKS,
                    self.state.bank_balance - prev.bank_balance,
                );
            }

            let max_ships = self.state.ships.len().max(prev.ships.len());
            if self.ship_cash_hi.len() < max_ships {
                self.ship_cash_hi.resize(max_ships, Highlight::default());
                self.ship_ton_hi.resize(max_ships, Highlight::default());
            }
            for idx in 0..max_ships {
                let cur = self.state.ships.get(idx);
                let old = prev.ships.get(idx);
                if let (Some(c), Some(p)) = (cur, old) {
                    if c.cash != p.cash {
                        self.ship_cash_hi[idx] =
                            Highlight::new(self.ticks + HIGHLIGHT_TICKS, c.cash - p.cash);
                    }
                    if c.net_tonnage != p.net_tonnage {
                        self.ship_ton_hi[idx] = Highlight::new(
                            self.ticks + HIGHLIGHT_TICKS,
                            (c.net_tonnage - p.net_tonnage) as i64,
                        );
                    }
                }
            }

            let ships_len = self.state.ships.len();
            if self.cargo_hi.len() < ships_len {
                self.cargo_hi.resize(
                    ships_len,
                    vec![Highlight::default(); Merchandise::ALL.len()],
                );
            }
            for ship_idx in 0..ships_len {
                let goods_len = Merchandise::ALL.len();
                if self.cargo_hi[ship_idx].len() < goods_len {
                    self.cargo_hi[ship_idx].resize(goods_len, Highlight::default());
                }
                if let (Some(c), Some(p)) =
                    (self.state.ships.get(ship_idx), prev.ships.get(ship_idx))
                {
                    for good in 0..goods_len {
                        if c.cargo[good] != p.cargo[good] {
                            self.cargo_hi[ship_idx][good] = Highlight::new(
                                self.ticks + HIGHLIGHT_TICKS,
                                (c.cargo[good] - p.cargo[good]) as i64,
                            );
                        }
                    }
                }
            }

            let sys_len = self.state.systems.len();
            if self.stock_hi.len() < sys_len {
                self.stock_hi
                    .resize(sys_len, vec![Highlight::default(); Merchandise::ALL.len()]);
                self.price_hi
                    .resize(sys_len, vec![Highlight::default(); Merchandise::ALL.len()]);
            }
            for sys in 0..sys_len {
                if self.stock_hi[sys].len() < Merchandise::ALL.len() {
                    self.stock_hi[sys].resize(Merchandise::ALL.len(), Highlight::default());
                    self.price_hi[sys].resize(Merchandise::ALL.len(), Highlight::default());
                }
                if let (Some(c), Some(p)) = (self.state.systems.get(sys), prev.systems.get(sys)) {
                    for good in 0..Merchandise::ALL.len() {
                        if c.stock[good] != p.stock[good] {
                            self.stock_hi[sys][good] = Highlight::new(
                                self.ticks + HIGHLIGHT_TICKS,
                                (c.stock[good] - p.stock[good]) as i64,
                            );
                        }
                        if c.prices[good] != p.prices[good] {
                            self.price_hi[sys][good] = Highlight::new(
                                self.ticks + HIGHLIGHT_TICKS,
                                (c.prices[good] - p.prices[good]) as i64,
                            );
                        }
                    }
                }
            }
        }

        self.prev_state = Some(self.state.clone());
    }

    #[allow(clippy::type_complexity)]
    fn init_highlights(
        state: &GameState,
    ) -> (
        Vec<Highlight>,
        Vec<Highlight>,
        Vec<Vec<Highlight>>,
        Vec<Vec<Highlight>>,
        Vec<Vec<Highlight>>,
    ) {
        let ships = state.ships.len();
        let goods = Merchandise::ALL.len();
        let systems = state.systems.len();
        (
            vec![Highlight::default(); ships],
            vec![Highlight::default(); ships],
            vec![vec![Highlight::default(); goods]; ships],
            vec![vec![Highlight::default(); goods]; systems],
            vec![vec![Highlight::default(); goods]; systems],
        )
    }

    fn begin_input(&mut self, mode: InputMode, prompt: impl Into<String>) {
        self.input_mode = Some(mode);
        self.input_buffer.clear();
        self.status = prompt.into();
    }

    fn buy_qty_prompt(&self) -> String {
        let good_idx = self.state.selected_good;
        let good = Merchandise::ALL[good_idx].code();
        let stock = self
            .state
            .systems
            .get(self.state.selected_system)
            .map(|s| s.stock[good_idx].max(0))
            .unwrap_or(0);
        let ton_left = self
            .state
            .ships
            .get(self.state.selected_ship)
            .map(|ship| self.state.max_tonnage.saturating_sub(ship.net_tonnage))
            .unwrap_or(self.state.max_tonnage);

        if good_idx < 4 {
            format!("Buy qty ({good}, stock {stock}, ton left {ton_left})")
        } else {
            format!("Buy qty ({good}, stock {stock})")
        }
    }

    fn sell_qty_prompt(&self) -> String {
        let good_idx = self.state.selected_good;
        let good = Merchandise::ALL[good_idx].code();
        let cargo = self
            .state
            .ships
            .get(self.state.selected_ship)
            .map(|ship| ship.cargo[good_idx])
            .unwrap_or(0);

        format!("Sell qty ({good}, cargo {cargo})")
    }

    fn deposit_prompt(&self) -> String {
        let ship_cash = self
            .state
            .ships
            .get(self.state.selected_ship)
            .map(|ship| ship.cash)
            .unwrap_or(0);
        format!(
            "Deposit amount (ship ${ship_cash}, bank ${})",
            self.state.bank_balance
        )
    }

    fn withdraw_prompt(&self) -> String {
        let ship_cash = self
            .state
            .ships
            .get(self.state.selected_ship)
            .map(|ship| ship.cash)
            .unwrap_or(0);
        format!(
            "Withdraw amount (bank ${}, ship ${ship_cash})",
            self.state.bank_balance
        )
    }

    fn destination_prompt(&self) -> String {
        let total = self.state.systems.len();
        if total == 0 {
            "No systems available".to_string()
        } else {
            format!("Destination system # (1-{total})")
        }
    }

    fn trade_price_prompt(&self, session: &TradeSession, verb: &str) -> String {
        let good = Merchandise::ALL[session.good_idx].code();
        let fair_total = session.price_each * session.qty as f32;
        format!(
            "{verb} total for {qty} {good} (round {}/{}, fair ~${:.0})",
            session.round,
            session.max_rounds,
            fair_total,
            verb = verb,
            qty = session.qty,
            good = good
        )
    }

    fn handle_overlay_key(&mut self, key: KeyCode) -> bool {
        let kb = &self.keybinds;
        match key {
            KeyCode::Esc => {
                self.overlay = None;
                self.status = "Closed overlay".to_string();
                return true;
            }
            code if code == kb.quit => {
                self.request_quit();
                return true;
            }
            code if code == kb.reset => {
                self.reset_demo_state();
                return true;
            }
            code if code == kb.map => {
                self.toggle_overlay(Overlay::Map, "Map overlay — Esc to close");
                return true;
            }
            code if code == kb.report => {
                self.toggle_overlay(Overlay::Report, "Trade report — Esc to close");
                return true;
            }
            code if kb.matches_help(code) => {
                self.toggle_overlay(Overlay::Help, "Help — Esc to close");
                return true;
            }
            KeyCode::Tab => {
                self.state.cycle_ship(1);
                return true;
            }
            KeyCode::BackTab => {
                self.state.cycle_ship(-1);
                return true;
            }
            KeyCode::Enter => {
                let msg = self.state.route_selected_ship_to_selected_system();
                self.status = msg.clone();
                self.log_event(msg);
                return true;
            }
            key @ (KeyCode::Char('r') | KeyCode::Char('R')) if key != kb.reset => {
                if let Some(ship) = self.state.selected_ship() {
                    self.state.selected_system = ship
                        .location
                        .min(self.state.systems.len().saturating_sub(1));
                }
                return true;
            }
            KeyCode::Up => {
                self.state.cycle_ship(-1);
                return true;
            }
            KeyCode::Down => {
                self.state.cycle_ship(1);
                return true;
            }
            KeyCode::Left => {
                self.state.cycle_system(-1);
                return true;
            }
            KeyCode::Right => {
                self.state.cycle_system(1);
                return true;
            }
            _ => {}
        }
        false
    }

    fn handle_input_mode(&mut self, key: KeyCode) {
        match key {
            KeyCode::Esc => {
                self.input_mode = None;
                self.input_buffer.clear();
                self.status = "Canceled.".to_string();
                self.pending_trade = None;
            }
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
                if matches!(self.input_mode, Some(InputMode::PickDestination)) =>
            {
                let delta = match key {
                    KeyCode::Up => -1,
                    KeyCode::Down => 1,
                    KeyCode::Left => -1,
                    KeyCode::Right => 1,
                    _ => 0,
                };
                self.state.cycle_system(delta);
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                self.input_buffer.push(c);
            }
            KeyCode::Enter => {
                let input = self.input_buffer.clone();
                let parsed = input.parse::<i32>().ok();
                let mode = self.input_mode.take();
                self.input_buffer.clear();
                if let Some(m) = mode {
                    self.apply_input(m, parsed);
                }
            }
            _ => {}
        }
    }

    fn apply_input(&mut self, mode: InputMode, parsed: Option<i32>) {
        match mode {
            InputMode::Buy => {
                if let Some(qty) = parsed {
                    match self.state.start_trade_session(
                        TradeKind::BuyFromSystem,
                        self.state.selected_ship,
                        self.state.selected_system,
                        self.state.selected_good,
                        qty,
                    ) {
                        Ok(sess) => {
                            self.pending_trade = Some(TradeContext::from_session(sess));
                            self.input_mode = Some(InputMode::TradePrice);
                            if let Some(ctx) = self.pending_trade.as_ref() {
                                self.status = self.trade_price_prompt(&ctx.session, "Offer");
                            }
                        }
                        Err(e) => {
                            self.status = format!("{e}. {}", self.buy_qty_prompt());
                            self.input_mode = Some(InputMode::Buy);
                            self.log_event(e);
                        }
                    }
                } else {
                    self.status = format!("Enter a quantity. {}", self.buy_qty_prompt());
                    self.input_mode = Some(InputMode::Buy);
                }
            }
            InputMode::Sell => {
                if let Some(qty) = parsed {
                    match self.state.start_trade_session(
                        TradeKind::SellToSystem,
                        self.state.selected_ship,
                        self.state.selected_system,
                        self.state.selected_good,
                        qty,
                    ) {
                        Ok(sess) => {
                            self.pending_trade = Some(TradeContext::from_session(sess));
                            self.input_mode = Some(InputMode::TradePrice);
                            if let Some(ctx) = self.pending_trade.as_ref() {
                                self.status = self.trade_price_prompt(&ctx.session, "Ask");
                            }
                        }
                        Err(e) => {
                            self.status = format!("{e}. {}", self.sell_qty_prompt());
                            self.input_mode = Some(InputMode::Sell);
                            self.log_event(e);
                        }
                    }
                } else {
                    self.status = format!("Enter a quantity. {}", self.sell_qty_prompt());
                    self.input_mode = Some(InputMode::Sell);
                }
            }
            InputMode::Deposit => {
                if let Some(amount) = parsed {
                    match self.state.deposit(self.state.selected_ship, amount as i64) {
                        Ok(msg) => {
                            self.status = msg.clone();
                            self.log_event(msg);
                            info!("{}", self.status);
                        }
                        Err(e) => {
                            self.status = e.clone();
                            self.input_mode = Some(InputMode::Deposit);
                            self.log_event(e);
                            warn!("{}", self.status);
                        }
                    }
                } else {
                    self.status = self.deposit_prompt();
                    self.input_mode = Some(InputMode::Deposit);
                }
            }
            InputMode::Withdraw => {
                if let Some(amount) = parsed {
                    match self.state.withdraw(self.state.selected_ship, amount as i64) {
                        Ok(msg) => {
                            self.status = msg.clone();
                            self.log_event(msg);
                            info!("{}", self.status);
                        }
                        Err(e) => {
                            self.status = e.clone();
                            self.input_mode = Some(InputMode::Withdraw);
                            self.log_event(e);
                            warn!("{}", self.status);
                        }
                    }
                } else {
                    self.status = self.withdraw_prompt();
                    self.input_mode = Some(InputMode::Withdraw);
                }
            }
            InputMode::SetDestination => {
                if let Some(idx1) = parsed {
                    let dest_idx = (idx1 - 1).max(0) as usize;
                    if dest_idx < self.state.systems.len() {
                        self.state.selected_system = dest_idx;
                        let msg = self.state.route_selected_ship_to_selected_system();
                        self.status = msg.clone();
                        self.log_event(msg);
                    } else {
                        self.status = format!("No such system. {}", self.destination_prompt());
                        self.input_mode = Some(InputMode::SetDestination);
                    }
                } else {
                    self.status = self.destination_prompt();
                    self.input_mode = Some(InputMode::SetDestination);
                }
            }
            InputMode::PickDestination => {
                let msg = self.state.route_selected_ship_to_selected_system();
                self.status = msg.clone();
                self.log_event(msg);
            }
            InputMode::TradePrice => {
                if parsed.is_none() {
                    self.status = "Enter a price".to_string();
                    self.input_mode = Some(InputMode::TradePrice);
                    return;
                }

                let Some(ctx) = self.pending_trade.take() else {
                    self.status = "No pending trade".to_string();
                    return;
                };

                let total = parsed.unwrap();
                let outcome = self.state.evaluate_offer(ctx.session, total);
                match outcome {
                    HaggleOutcome::Accept(msg) => {
                        self.status = msg.clone();
                        self.log_event(msg);
                    }
                    HaggleOutcome::Reject(msg) => {
                        self.status = msg.clone();
                        self.log_event(msg);
                    }
                    HaggleOutcome::Counter(next_sess, msg) => {
                        self.status = msg.clone();
                        self.pending_trade = Some(TradeContext::from_session(next_sess));
                        self.input_mode = Some(InputMode::TradePrice);
                        self.log_event(msg);
                    }
                }
            }
        }
    }

    fn open_overlay(&mut self, overlay: Overlay, status: &str) {
        self.overlay = Some(overlay);
        self.status = status.to_string();
        self.log_event(status);
    }

    fn toggle_overlay(&mut self, overlay: Overlay, status: &str) {
        if self.overlay == Some(overlay) {
            self.overlay = None;
            self.status = "Closed overlay".to_string();
        } else {
            self.open_overlay(overlay, status);
        }
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn menu_items(&self) -> &'static [MenuItem] {
        &MENU_ITEMS
    }

    pub fn menu_index(&self) -> usize {
        self.menu_index.min(MENU_ITEMS.len().saturating_sub(1))
    }

    pub fn status_text(&self) -> &str {
        &self.status
    }

    pub fn ticks(&self) -> u64 {
        self.ticks
    }

    pub fn events(&self) -> &[String] {
        &self.events
    }

    pub fn bank_highlight(&self) -> Option<i64> {
        self.bank_hi
            .active(self.ticks)
            .then_some(self.bank_hi.delta)
    }

    pub fn ship_cash_highlight(&self, idx: usize) -> Option<i64> {
        self.ship_cash_hi
            .get(idx)
            .filter(|h| h.active(self.ticks))
            .map(|h| h.delta)
    }

    pub fn ship_tonnage_highlight(&self, idx: usize) -> Option<i64> {
        self.ship_ton_hi
            .get(idx)
            .filter(|h| h.active(self.ticks))
            .map(|h| h.delta)
    }

    pub fn cargo_highlight(&self, ship_idx: usize, good_idx: usize) -> Option<i64> {
        self.cargo_hi
            .get(ship_idx)
            .and_then(|g| g.get(good_idx))
            .filter(|h| h.active(self.ticks))
            .map(|h| h.delta)
    }

    pub fn stock_highlight(&self, system_idx: usize, good_idx: usize) -> Option<i64> {
        self.stock_hi
            .get(system_idx)
            .and_then(|g| g.get(good_idx))
            .filter(|h| h.active(self.ticks))
            .map(|h| h.delta)
    }

    pub fn price_highlight(&self, system_idx: usize, good_idx: usize) -> Option<i64> {
        self.price_hi
            .get(system_idx)
            .and_then(|g| g.get(good_idx))
            .filter(|h| h.active(self.ticks))
            .map(|h| h.delta)
    }

    pub fn overlay(&self) -> Option<Overlay> {
        self.overlay
    }

    pub fn key_hints(&self) -> Vec<(String, String)> {
        match self.screen {
            Screen::Launch => vec![
                ("Any key".to_string(), "skip".to_string()),
                ("Auto".to_string(), "after a moment".to_string()),
            ],
            Screen::Menu => vec![
                ("Up/Down".to_string(), "choose".to_string()),
                ("Enter".to_string(), "select".to_string()),
                ("Esc/q".to_string(), "quit".to_string()),
            ],
            Screen::Instructions | Screen::About => vec![
                ("Esc".to_string(), "back to menu".to_string()),
                ("Enter".to_string(), "back to menu".to_string()),
            ],
            Screen::Game => self.keybinds.footer_hints(),
        }
    }

    pub fn save_to_path(&self, path: impl AsRef<Path>) -> Result<String, String> {
        let path_ref = path.as_ref();
        let file = File::create(path_ref).map_err(|e| format!("Save failed: {e}"))?;
        self.save_to_writer_internal(file, Some(path_ref.to_owned()))
    }

    #[allow(dead_code)]
    pub fn save_to_writer<W: Write>(&self, writer: W) -> Result<String, String> {
        self.save_to_writer_internal(writer, None)
    }

    fn save_to_writer_internal<W: Write>(
        &self,
        mut writer: W,
        path: Option<std::path::PathBuf>,
    ) -> Result<String, String> {
        let payload = SaveFile {
            version: 1,
            app: SavedApp::from_app(self),
        };
        serde_json::to_writer_pretty(&mut writer, &payload)
            .map_err(|e| format!("Save failed: {e}"))?;
        writer.flush().map_err(|e| format!("Save failed: {e}"))?;
        let target = path
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "buffer".to_string());
        info!("Saved to {target}");
        Ok(format!("Saved to {target}"))
    }

    pub fn load_from_path(path: impl AsRef<Path>) -> Result<Self, String> {
        let mut buf = String::new();
        let mut file = File::open(path.as_ref()).map_err(|e| format!("Load failed: {e}"))?;
        file.read_to_string(&mut buf)
            .map_err(|e| format!("Load failed: {e}"))?;
        Self::load_from_str(&buf)
    }

    pub fn load_from_str(s: &str) -> Result<Self, String> {
        let parsed: SaveFile = serde_json::from_str(s).map_err(|e| format!("Load failed: {e}"))?;
        if parsed.version != 1 {
            return Err(format!("Unsupported save version {}", parsed.version));
        }
        Ok(parsed.app.into_app())
    }
}

#[cfg(test)]
mod tests {
    use super::App;
    use pretty_assertions::assert_eq;

    #[test]
    fn save_and_load_roundtrip() {
        let mut app = App::default();
        app.state.bank_balance = 1234;
        if let Some(ship) = app.state.ships.get_mut(0) {
            ship.cash = 777;
        }
        app.events.push("Test event".to_string());
        app.ticks = 42;

        let mut buf = Vec::new();
        let save_msg = app.save_to_writer(&mut buf).expect("save succeeds");
        assert!(save_msg.contains("Saved"));

        let json = String::from_utf8(buf).expect("valid utf8");
        let loaded = App::load_from_str(&json).expect("load succeeds");

        assert_eq!(app.state, loaded.state);
        assert_eq!(app.events, loaded.events);
        assert_eq!(app.ticks, loaded.ticks);
        assert_eq!(app.active_panel, loaded.active_panel);
    }
}
