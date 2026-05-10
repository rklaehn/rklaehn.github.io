// Starship + LunOX architecture model.
// All parameters live in a `Config` struct; calculation functions take `&Config`.

mod no_tether;

// ---------- Config ----------

#[derive(Debug, Clone)]
struct CargoStarship {
    dry: f64,           // t
    prop_cap: f64,      // t (full main-tank capacity, 78/22 mix in standard tank)
    bay: f64,           // t (cargo bay capacity)
    leo_residual: f64,  // t (main-tank prop residual when arriving at LEO from Earth)
}

#[derive(Debug, Clone)]
struct LunarStarship {
    dry: f64,
    prop_cap: f64,
}

#[derive(Debug, Clone)]
struct Engine {
    isp_s: f64,         // vacuum Isp, seconds
    g0_m_s2: f64,       // standard gravity
    lox_frac: f64,      // LOX mass fraction of methalox
    fuel_frac: f64,      // CH4 mass fraction of methalox
}

impl Engine {
    fn ve_km_s(&self) -> f64 {
        self.isp_s * self.g0_m_s2 / 1000.0
    }
}

#[derive(Debug, Clone)]
struct DvBudget {
    leo_eml1: f64,      // km/s
    eml1_moon: f64,
    moon_eml1: f64,
    eml1_leo: f64,      // (+aerobraking handles the rest)
    leo_landing: f64,   // deorbit + landing
}

#[derive(Debug, Clone)]
struct Config {
    cargo: CargoStarship,
    lunar: LunarStarship,
    tanker_payload: f64, // t CH4 cargo per tanker flight
    engine: Engine,
    dv: DvBudget,
    n_cargo: usize,      // cargo Starship flights per cycle
}

impl Config {
    fn mass_ratio(&self, dv_km_s: f64) -> f64 {
        (dv_km_s / self.engine.ve_km_s()).exp()
    }
}

// Canonical Starship V3 parameters with the cislunar Δv budget assumed in the
// blog post. Everything in the Config is a plain variable — dry mass, prop tank
// capacity, mixture fractions, Isp, Δv legs, ship counts — so swap any field to
// explore variants.
const STARSHIP_V3: Config = Config {
    cargo: CargoStarship {
        dry: 130.0,
        prop_cap: 1600.0,
        bay: 150.0,
        leo_residual: 30.0,
    },
    lunar: LunarStarship {
        dry: 90.0,
        prop_cap: 1600.0,
    },
    tanker_payload: 200.0,
    engine: Engine {
        isp_s: 380.0,
        g0_m_s2: 9.81,
        lox_frac: 0.78,
        fuel_frac: 0.22,
    },
    dv: DvBudget {
        leo_eml1: 3.80,
        eml1_moon: 2.50,
        moon_eml1: 2.50,
        eml1_leo: 0.77,
        leo_landing: 0.10,
    },
    n_cargo: 3,
};

// Hypothetical Stoke-like LOX/LH2 architecture, Starship-class vehicle scale
// for fair comparison (volumetrically unrealistic — LH2 is ~6× less dense than
// CH4 — but isolates the propellant chemistry effect). Mixture ratio 6:1 LOX:LH2.
#[allow(dead_code)]
const STOKE_LIKE: Config = Config {
    cargo: CargoStarship {
        dry: 130.0,
        prop_cap: 1600.0,
        bay: 150.0,
        leo_residual: 30.0,
    },
    lunar: LunarStarship {
        dry: 90.0,
        prop_cap: 1600.0,
    },
    tanker_payload: 200.0,
    engine: Engine {
        isp_s: 462.0,
        g0_m_s2: 9.81,
        lox_frac: 0.857,
        fuel_frac: 0.143,
    },
    dv: DvBudget {
        leo_eml1: 3.80,
        eml1_moon: 2.50,
        moon_eml1: 2.50,
        eml1_leo: 0.77,
        leo_landing: 0.10,
    },
    n_cargo: 3,
};

// ---------- Cargo Starship leg analysis ----------

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct CargoOutbound {
    extra_ch4: f64,
    ch4_loaded_at_leo: f64,
    lox_loaded_at_leo: f64,
    prop_burned: f64,
    lox_burned: f64,
    ch4_burned: f64,
    lox_residual_eml1: f64,
    ch4_residual_eml1: f64,
}

// Cargo Starship leaves LEO with full LOX + CH4 (standard 78/22 prop tank) plus
// `extra_ch4` of additional CH4 carried in slightly enlarged CH4 tankage. The
// extra CH4 is a "passenger" — not consumed in the burn (which stays 78/22 of
// burnable propellant), just delivered to EML1 as residual.
fn cargo_outbound(cfg: &Config, extra_ch4: f64) -> CargoOutbound {
    let lox_loaded = cfg.cargo.prop_cap * cfg.engine.lox_frac;
    let ch4_loaded = cfg.cargo.prop_cap * cfg.engine.fuel_frac + extra_ch4;
    let m_initial = cfg.cargo.dry + cfg.cargo.bay + lox_loaded + ch4_loaded;
    let m_final = m_initial / cfg.mass_ratio(cfg.dv.leo_eml1);
    let prop = m_initial - m_final;
    CargoOutbound {
        extra_ch4,
        ch4_loaded_at_leo: ch4_loaded,
        lox_loaded_at_leo: lox_loaded,
        prop_burned: prop,
        lox_burned: cfg.engine.lox_frac * prop,
        ch4_burned: cfg.engine.fuel_frac * prop,
        lox_residual_eml1: lox_loaded - cfg.engine.lox_frac * prop,
        ch4_residual_eml1: ch4_loaded - cfg.engine.fuel_frac * prop,
    }
}

#[derive(Debug, Clone, Copy)]
struct CargoReturn {
    prop_burned: f64,
    lox_burned: f64,
    ch4_burned: f64,
    lox_loaded_at_eml1: f64,
    ch4_loaded_at_eml1: f64,
    landing_prop: f64,
    landing_lox: f64,
    landing_ch4: f64,
    lox_delivered_to_leo: f64,
    ch4_excess_at_eml1: f64,
    ch4_topup_from_eml1: f64,
}

// Cargo Starship return EML1 → LEO → Earth.
// Tank policy: top up LOX to full at EML1 (LunOX is free at the depot); load
// only enough CH4 for the return burn + landing reserve.
fn cargo_return(cfg: &Config, out: &CargoOutbound) -> CargoReturn {
    let r = cfg.mass_ratio(cfg.dv.eml1_leo);
    let r_landing = cfg.mass_ratio(cfg.dv.leo_landing);

    let landing_prop = cfg.cargo.dry * (r_landing - 1.0);
    let landing_lox = cfg.engine.lox_frac * landing_prop;
    let landing_ch4 = cfg.engine.fuel_frac * landing_prop;

    let alpha = (r - 1.0) / r;
    let lox_loaded = cfg.cargo.prop_cap * cfg.engine.lox_frac;
    let ch4_loaded = (cfg.engine.fuel_frac * (cfg.cargo.dry + lox_loaded) * alpha + landing_ch4)
                   / (1.0 - cfg.engine.fuel_frac * alpha);

    let m_initial = cfg.cargo.dry + lox_loaded + ch4_loaded;
    let prop_burned = m_initial * alpha;
    let lox_burned = cfg.engine.lox_frac * prop_burned;
    let ch4_burned = cfg.engine.fuel_frac * prop_burned;

    let lox_in_tank_at_leo = lox_loaded - lox_burned;
    let lox_delivered_to_leo = lox_in_tank_at_leo - landing_lox;

    let outbound_residual_ch4 = out.ch4_residual_eml1;
    let ch4_excess_at_eml1 = (outbound_residual_ch4 - ch4_loaded).max(0.0);
    let ch4_topup_from_eml1 = (ch4_loaded - outbound_residual_ch4).max(0.0);

    CargoReturn {
        prop_burned,
        lox_burned,
        ch4_burned,
        lox_loaded_at_eml1: lox_loaded,
        ch4_loaded_at_eml1: ch4_loaded,
        landing_prop,
        landing_lox,
        landing_ch4,
        lox_delivered_to_leo,
        ch4_excess_at_eml1,
        ch4_topup_from_eml1,
    }
}

// ---------- Lunar Starship round trip ----------

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
struct LunarRoundTrip {
    descent_prop: f64,
    descent_lox: f64,
    descent_ch4: f64,
    ascent_prop: f64,
    ascent_lox: f64,
    ascent_ch4: f64,
    descent_ch4_loaded: f64,
    descent_lox_loaded: f64,
    eml1_lox_needed: f64,
    eml1_ch4_needed: f64,
    moon_lox_topup: f64,
}

// Tank policy: load only enough LOX for the descent burn (LOX is plentiful at
// both endpoints), and CH4 = descent burn + ascent reserve.
fn lunar_round_trip(cfg: &Config, payload: f64) -> LunarRoundTrip {
    let r_desc = cfg.mass_ratio(cfg.dv.eml1_moon);
    let r_asc = cfg.mass_ratio(cfg.dv.moon_eml1);
    let dry = cfg.lunar.dry;
    let lox_frac = cfg.engine.lox_frac;
    let fuel_frac = cfg.engine.fuel_frac;

    // Ascent Moon → EML1, no payload, zero residual at EML1.
    let prop_asc = dry * (r_asc - 1.0);
    let lox_asc = lox_frac * prop_asc;
    let ch4_asc = fuel_frac * prop_asc;

    // Descent EML1 → Moon. m_final = dry + payload + ch4_asc (CH4 reserve carried
    // through landing to feed ascent; LOX is fully consumed on descent).
    let m_final_desc = dry + payload + ch4_asc;
    let prop_desc = m_final_desc * (r_desc - 1.0);
    let lox_desc = lox_frac * prop_desc;
    let ch4_desc = fuel_frac * prop_desc;

    let descent_lox_loaded = lox_desc;
    let descent_ch4_loaded = ch4_desc + ch4_asc;

    LunarRoundTrip {
        descent_prop: prop_desc,
        descent_lox: lox_desc,
        descent_ch4: ch4_desc,
        ascent_prop: prop_asc,
        ascent_lox: lox_asc,
        ascent_ch4: ch4_asc,
        descent_ch4_loaded,
        descent_lox_loaded,
        eml1_lox_needed: descent_lox_loaded,
        eml1_ch4_needed: descent_ch4_loaded,
        moon_lox_topup: lox_asc,
    }
}

// ---------- Per-cycle balance ----------

#[derive(Debug, Clone, Copy)]
struct DepotBalance {
    lox_in: f64,
    lox_out: f64,
    ch4_in: f64,
    ch4_out: f64,
}

impl DepotBalance {
    fn lox_net(&self) -> f64 { self.lox_in - self.lox_out }
    fn ch4_net(&self) -> f64 { self.ch4_in - self.ch4_out }
}

#[derive(Debug, Clone, Copy)]
struct Cycle {
    lunar_payload: f64,
    cargo_payload_per_ship: f64,
    bay_unused_per_ship: f64,
    cargo_lox_to_leo_per_ship: f64,
    extra_ch4_per_ship: f64,
    ch4_tank_total_per_ship: f64,
    n_tankers: usize,
    leo: DepotBalance,
    eml1: DepotBalance,
    tether_lox_per_cycle: f64,
    binding_constraint: &'static str,
}

// Given an extra-CH4 load per cargo Starship, return the maximum lunar payload
// the EML1 CH4 supply can support. Monotonic increasing in `extra_ch4`.
fn max_payload_for_extra(cfg: &Config, extra_ch4: f64) -> f64 {
    let n_c = cfg.n_cargo as f64;
    let out = cargo_outbound(cfg, extra_ch4);
    let ret = cargo_return(cfg, &out);
    let cargo_ch4_excess_per_ship = ret.ch4_excess_at_eml1 - ret.ch4_topup_from_eml1;
    let eml1_ch4_supply = n_c * cargo_ch4_excess_per_ship;
    if eml1_ch4_supply <= 0.0 || lunar_round_trip(cfg, 0.0).eml1_ch4_needed > eml1_ch4_supply {
        return 0.0;
    }
    let mut lo = 0.0f64;
    let mut hi = 10_000.0f64;
    for _ in 0..60 {
        let mid = 0.5 * (lo + hi);
        if lunar_round_trip(cfg, mid).eml1_ch4_needed <= eml1_ch4_supply {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

fn solve_cycle(cfg: &Config) -> Option<Cycle> {
    let n_c = cfg.n_cargo as f64;
    let bay_max = n_c * cfg.cargo.bay;

    // Find the smallest extra CH4 (per ship) such that the EML1 CH4 supply allows
    // P = bay_max (i.e., bay binds). If extra=0 already does, no enlargement needed.
    let extra_ch4 = if max_payload_for_extra(cfg, 0.0) >= bay_max {
        0.0
    } else {
        let mut lo = 0.0f64;
        let mut hi = 1_000.0f64;
        for _ in 0..80 {
            let mid = 0.5 * (lo + hi);
            if max_payload_for_extra(cfg, mid) >= bay_max {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        hi
    };

    let out = cargo_outbound(cfg, extra_ch4);
    let ret = cargo_return(cfg, &out);

    let cargo_ch4_excess_per_ship = ret.ch4_excess_at_eml1 - ret.ch4_topup_from_eml1;
    let eml1_ch4_supply = n_c * cargo_ch4_excess_per_ship;
    let p_ch4_max = max_payload_for_extra(cfg, extra_ch4);
    let p = p_ch4_max.min(bay_max);
    let binding = if (bay_max - p).abs() < 1.0 { "bay" } else { "ch4" };

    let lunar = lunar_round_trip(cfg, p);
    let p_per_ship = p / n_c;
    let bay_unused_per_ship = (cfg.cargo.bay - p_per_ship).max(0.0);

    // LEO CH4 balance: tanker count sized to cover refill demand. Each cargo
    // Starship loads `out.ch4_loaded_at_leo` (incl. extra) at LEO.
    let arrival_ch4_per_ship = cfg.cargo.leo_residual * cfg.engine.fuel_frac;
    let refill_ch4_per_ship = out.ch4_loaded_at_leo - arrival_ch4_per_ship;
    let leo_ch4_demand = n_c * refill_ch4_per_ship;
    let n_tankers = (leo_ch4_demand / cfg.tanker_payload).ceil() as usize;
    let leo_ch4_supply = n_tankers as f64 * cfg.tanker_payload;

    // LEO LOX balance: cargo Starship returns vs refuel demand.
    let arrival_lox_per_ship = cfg.cargo.leo_residual * cfg.engine.lox_frac;
    let refill_lox_per_ship = cfg.cargo.prop_cap * cfg.engine.lox_frac - arrival_lox_per_ship;
    let leo_lox_demand = n_c * refill_lox_per_ship;
    let leo_lox_supply = n_c * ret.lox_delivered_to_leo;

    Some(Cycle {
        lunar_payload: p,
        cargo_payload_per_ship: p_per_ship,
        bay_unused_per_ship,
        cargo_lox_to_leo_per_ship: ret.lox_delivered_to_leo,
        extra_ch4_per_ship: extra_ch4,
        ch4_tank_total_per_ship: out.ch4_loaded_at_leo,
        n_tankers,
        leo: DepotBalance {
            lox_in: leo_lox_supply,
            lox_out: leo_lox_demand,
            ch4_in: leo_ch4_supply,
            ch4_out: leo_ch4_demand,
        },
        eml1: DepotBalance {
            lox_in: 0.0, // tether supplies whatever is needed; reported separately
            lox_out: lunar.eml1_lox_needed + n_c * ret.lox_burned,
            ch4_in: eml1_ch4_supply,
            ch4_out: lunar.eml1_ch4_needed,
        },
        tether_lox_per_cycle:
            // Net new LOX needed at EML1 from the tether per cycle.
            // Cargo Starship outbound residual is recycled into the EML1 depot,
            // so the tether only supplies the difference between depot withdrawals
            // and residual contributions.
            n_c * (ret.lox_delivered_to_leo + ret.lox_burned - out.lox_residual_eml1)
            + lunar.eml1_lox_needed,
        binding_constraint: binding,
    })
}

// ---------- Output ----------

fn main() {
    run(&STARSHIP_V3, "STARSHIP V3 (LOX/CH4, Isp 380 s)");
    println!("\n\n\n");
    run(&STOKE_LIKE, "STOKE-LIKE (LOX/LH2, Isp 462 s)");
}

fn run(cfg: &Config, label: &str) {
    println!("################################################################");
    println!("# {}", label);
    println!("################################################################\n");

    print_config(cfg);

    let cycle = solve_cycle(cfg);
    let preview_extra = cycle.map(|c| c.extra_ch4_per_ship).unwrap_or(0.0);

    let out = cargo_outbound(cfg, preview_extra);
    print_outbound(&out);

    let ret = cargo_return(cfg, &out);
    print_return(cfg, &ret);

    println!("Architecture: {} cargo Starship(s) (bay = 100% lunar payload) + auto-sized fuel tankers",
             cfg.n_cargo);
    println!("              fully separated payload / propellant Earth launches");
    println!();

    match cycle {
        Some(c) => print_cycle(cfg, &c),
        None => {
            println!("No feasible solution: not enough EML1 fuel supply for even an empty lunar round trip.");
            println!("Increase n_cargo so that cargo Starship fuel excess can fuel the lunar Starship.");
        }
    }

    println!();
    let nt = no_tether::solve(cfg);
    no_tether::print(cfg, &nt);
}

fn print_config(cfg: &Config) {
    println!("Starship + LunOX architecture model");
    println!("====================================\n");

    println!("Vehicle parameters:");
    println!("  Cargo Starship:     dry={} t  prop_cap={} t  bay={} t",
             cfg.cargo.dry, cfg.cargo.prop_cap, cfg.cargo.bay);
    println!("  Tanker variant:     payload={} t (CH4 to LEO depot)", cfg.tanker_payload);
    println!("  Lunar Starship:     dry={} t  prop_cap={} t",
             cfg.lunar.dry, cfg.lunar.prop_cap);
    println!("  Methalox mix:       {:.0}% LOX / {:.0}% CH4",
             100.0 * cfg.engine.lox_frac, 100.0 * cfg.engine.fuel_frac);
    println!("  Engine:             Isp={} s  ve={:.3} km/s",
             cfg.engine.isp_s, cfg.engine.ve_km_s());
    println!();

    println!("Δv budget (km/s):");
    println!("  LEO  → EML1: {:.2}  (mass ratio {:.3})",
             cfg.dv.leo_eml1, cfg.mass_ratio(cfg.dv.leo_eml1));
    println!("  EML1 → Moon: {:.2}  (mass ratio {:.3})",
             cfg.dv.eml1_moon, cfg.mass_ratio(cfg.dv.eml1_moon));
    println!("  Moon → EML1: {:.2}  (mass ratio {:.3})",
             cfg.dv.moon_eml1, cfg.mass_ratio(cfg.dv.moon_eml1));
    println!("  EML1 → LEO:  {:.2}  (mass ratio {:.3}) + aerobraking",
             cfg.dv.eml1_leo, cfg.mass_ratio(cfg.dv.eml1_leo));
    println!();
}

fn print_outbound(out: &CargoOutbound) {
    println!("Cargo Starship outbound (LEO → EML1, full LOX + CH4 tank loaded with {:.1} t extra CH4):",
             out.extra_ch4);
    println!("  CH4 loaded at LEO:  {:>7.1} t  (extra {:.1})",
             out.ch4_loaded_at_leo, out.extra_ch4);
    println!("  Prop burned:        {:>7.1} t  (LOX {:>7.1} + CH4 {:>6.1})",
             out.prop_burned, out.lox_burned, out.ch4_burned);
    println!("  Residual at EML1:   {:>7.1} t  (LOX {:>7.1} + CH4 {:>6.1})",
             out.lox_residual_eml1 + out.ch4_residual_eml1,
             out.lox_residual_eml1, out.ch4_residual_eml1);
    println!();
}

fn print_return(cfg: &Config, ret: &CargoReturn) {
    println!("Cargo Starship return  (EML1 → LEO, top up LOX to full, CH4 sized for return + landing):");
    println!("  EML1 loading:       LOX {:>7.1} t  CH4 {:>6.1} t",
             ret.lox_loaded_at_eml1, ret.ch4_loaded_at_eml1);
    println!("  EML1 → LEO burn:    {:>7.1} t  (LOX {:>7.1} + CH4 {:>6.1})",
             ret.prop_burned, ret.lox_burned, ret.ch4_burned);
    println!("  Landing reserve:    {:>7.1} t  (LOX {:>7.1} + CH4 {:>6.2})  for Δv={:.2} km/s",
             ret.landing_prop, ret.landing_lox, ret.landing_ch4, cfg.dv.leo_landing);
    println!("  LOX cargo to LEO:   {:>7.1} t", ret.lox_delivered_to_leo);
    if ret.ch4_topup_from_eml1 > 0.0 {
        println!("  CH4 topup at EML1:  {:>7.1} t  (drawn from depot — outbound residual was short)",
                 ret.ch4_topup_from_eml1);
    } else {
        println!("  CH4 excess at EML1: {:>7.1} t  (transferable to depot)",
                 ret.ch4_excess_at_eml1);
    }
    println!();
}

fn print_cycle(cfg: &Config, c: &Cycle) {
    let standard_ch4 = cfg.cargo.prop_cap * cfg.engine.fuel_frac;
    let tank_increase_pct = 100.0 * c.extra_ch4_per_ship / standard_ch4;
    println!("Per-cycle solution:                  binding constraint: {}", c.binding_constraint);
    println!("  Total lunar surface payload:    {:>7.1} t", c.lunar_payload);
    println!("  Per cargo Starship:");
    println!("    Lunar payload (in bay):       {:>7.1} t", c.cargo_payload_per_ship);
    println!("    Bay unused:                   {:>7.1} / {:.0} t",
             c.bay_unused_per_ship, cfg.cargo.bay);
    println!("    LOX cargo back to LEO:        {:>7.1} t", c.cargo_lox_to_leo_per_ship);
    println!("    Extra CH4 carried (in tank):  {:>7.1} t  → CH4 tank {:.1} t (+{:.1}% over std {:.0} t)",
             c.extra_ch4_per_ship, c.ch4_tank_total_per_ship, tank_increase_pct, standard_ch4);
    println!();

    println!("LEO depot balance per cycle ({} tanker(s) auto-sized to balance CH4):", c.n_tankers);
    println!("  LOX in (cargo Starship returns):   {:>7.1} t", c.leo.lox_in);
    println!("  LOX out (cargo Starship refuels):  {:>7.1} t", c.leo.lox_out);
    println!("  LOX net:                           {:>+7.1} t", c.leo.lox_net());
    if c.leo.lox_net() < -0.5 {
        println!("  ⚠  LEO LOX deficit — LunOX returns alone can't cover refuel demand;");
        println!("     consider filling cargo Starship return bays with LunOX (up to {:.0} t/ship).",
                 cfg.cargo.bay);
    }
    println!("  CH4 in (tankers):                  {:>7.1} t", c.leo.ch4_in);
    println!("  CH4 out (cargo Starship refuels):  {:>7.1} t", c.leo.ch4_out);
    println!("  CH4 net:                           {:>+7.1} t", c.leo.ch4_net());
    println!();

    println!("EML1 depot balance per cycle:");
    println!("  LOX in (tether):                              {:>7.1} t", c.tether_lox_per_cycle);
    println!("  LOX out (lunar descent + cargo return burns): {:>7.1} t", c.eml1.lox_out);
    println!("  CH4 in (cargo Starship outbound excess):      {:>7.1} t", c.eml1.ch4_in);
    println!("  CH4 out (lunar Starship round trip):          {:>7.1} t", c.eml1.ch4_out);
    println!("  CH4 net:                                       {:>+7.1} t", c.eml1.ch4_net());
    println!();

    println!("Tether throughput per cycle:        {:>7.1} t LunOX → EML1", c.tether_lox_per_cycle);
    println!();

    let total_launches = cfg.n_cargo + c.n_tankers;
    let leverage = c.lunar_payload / total_launches as f64;
    println!("Launch summary:");
    println!("  Cargo Starships:                 {}", cfg.n_cargo);
    println!("  Tankers:                         {}", c.n_tankers);
    println!("  Total Earth launches:            {}", total_launches);
    println!("  Per-launch leverage:             {:.1} t lunar / launch", leverage);
}
