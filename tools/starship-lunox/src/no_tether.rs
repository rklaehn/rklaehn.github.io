// Comparison architecture: same 2 depots (LEO + EML1), but no LunOX tether.
// All propellant comes from Earth via tankers. Cargo Starships still cycle
// Earth → LEO → EML1 → LEO → Earth, but return with empty bay (no LunOX cargo).
// EML1 depot is supplied by cargo Starship outbound residuals plus, if needed,
// dedicated LEO → EML1 propellant tankers (Starship-variant with prop cargo in bay).

use crate::{cargo_outbound, Config};

// Lunar round trip without LunOX top-up on the Moon.
// The lunar Starship must carry both descent and ascent propellant from EML1,
// so descent loading carries ascent reserves as dead weight through landing.
#[allow(dead_code)]
struct LunarRoundTripNoLunox {
    eml1_lox_needed: f64,    // LOX loaded at EML1 (descent burn + ascent reserve)
    eml1_fuel_needed: f64,
    descent_lox_burn: f64,
    descent_fuel_burn: f64,
    descent_lox_preserved: f64, // ascent reserve carried through landing
    descent_fuel_preserved: f64,
    ascent_lox_burn: f64,
    ascent_fuel_burn: f64,
}

fn lunar_round_trip_no_lunox(cfg: &Config, payload: f64) -> LunarRoundTripNoLunox {
    let r_desc = cfg.mass_ratio(cfg.dv.eml1_moon);
    let r_asc = cfg.mass_ratio(cfg.dv.moon_eml1);
    let dry = cfg.lunar.dry;

    let prop_asc = dry * (r_asc - 1.0);
    let lox_asc = cfg.engine.lox_frac * prop_asc;
    let fuel_asc = cfg.engine.fuel_frac * prop_asc;

    // m_final after descent = dry + payload + ascent reserves (both LOX and fuel
    // carried through landing because the Moon has no propellant production).
    let m_final_desc = dry + payload + lox_asc + fuel_asc;
    let prop_desc = m_final_desc * (r_desc - 1.0);
    let lox_desc = cfg.engine.lox_frac * prop_desc;
    let fuel_desc = cfg.engine.fuel_frac * prop_desc;

    LunarRoundTripNoLunox {
        eml1_lox_needed: lox_desc + lox_asc,
        eml1_fuel_needed: fuel_desc + fuel_asc,
        descent_lox_burn: lox_desc,
        descent_fuel_burn: fuel_desc,
        descent_lox_preserved: lox_asc,
        descent_fuel_preserved: fuel_asc,
        ascent_lox_burn: lox_asc,
        ascent_fuel_burn: fuel_asc,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NoTetherCycle {
    pub lunar_payload: f64,
    pub n_eml1_tankers: usize,
    pub n_leo_tankers: usize,
    pub total_launches: usize,
    pub leverage: f64,
    pub leo_prop_demand: f64,
    pub eml1_lox_balance: f64,
    pub eml1_fuel_balance: f64,
    pub cargo_return_prop_per_ship: f64,
    pub eml1_tanker_net_per_flight: f64,
}

pub fn solve(cfg: &Config) -> NoTetherCycle {
    let n_c = cfg.n_cargo as f64;
    let lunar_payload = n_c * cfg.cargo.bay;
    let lunar = lunar_round_trip_no_lunox(cfg, lunar_payload);

    // No extra CH4 ferrying: EML1 fuel comes from cargo residuals (or dedicated tankers),
    // not from main-tank residual surplus engineered for that purpose.
    let out = cargo_outbound(cfg, 0.0);

    // Cargo Starship return: load only prop for return burn + landing — no LOX cargo.
    //   m_final after EML1 → LEO burn = dry + landing_reserve
    //   m_initial at EML1 = m_final × r_ret
    let r_ret = cfg.mass_ratio(cfg.dv.eml1_leo);
    let r_land = cfg.mass_ratio(cfg.dv.leo_landing);
    let landing_prop = cfg.cargo.dry * (r_land - 1.0);
    let m_final_at_leo = cfg.cargo.dry + landing_prop;
    let return_burn_prop = m_final_at_leo * (r_ret - 1.0);
    let cargo_eml1_total_load = return_burn_prop + landing_prop;
    let cargo_eml1_lox_load = cfg.engine.lox_frac * cargo_eml1_total_load;
    let cargo_eml1_fuel_load = cfg.engine.fuel_frac * cargo_eml1_total_load;

    // EML1 depot pre-tanker balance:
    //   IN  = n_cargo × outbound residuals
    //   OUT = lunar Starship round trip + n_cargo × return loading
    let eml1_lox_balance =
        n_c * out.lox_residual_eml1 - lunar.eml1_lox_needed - n_c * cargo_eml1_lox_load;
    let eml1_fuel_balance =
        n_c * out.ch4_residual_eml1 - lunar.eml1_fuel_needed - n_c * cargo_eml1_fuel_load;

    // LEO → EML1 propellant tanker: Starship variant carrying full main tank +
    // bay full of prop cargo (78/22 mix). Burns 78/22 from main tank en route,
    // delivers main residual + bay cargo to EML1, returns with min prop.
    let r_out = cfg.mass_ratio(cfg.dv.leo_eml1);
    let bay_prop = cfg.cargo.bay;
    let m_initial_out = cfg.cargo.dry + bay_prop + cfg.cargo.prop_cap;
    let prop_burned_out = m_initial_out * (r_out - 1.0) / r_out;
    let main_residual = cfg.cargo.prop_cap - prop_burned_out;
    let tanker_net_delivered = bay_prop + main_residual - cargo_eml1_total_load;
    let tanker_lox_per_flight = cfg.engine.lox_frac * tanker_net_delivered;
    let tanker_fuel_per_flight = cfg.engine.fuel_frac * tanker_net_delivered;

    // Tanker count sized to cover whichever propellant has the larger deficit.
    let lox_deficit = (-eml1_lox_balance).max(0.0);
    let fuel_deficit = (-eml1_fuel_balance).max(0.0);
    let n_for_lox = (lox_deficit / tanker_lox_per_flight).ceil();
    let n_for_fuel = (fuel_deficit / tanker_fuel_per_flight).ceil();
    let n_eml1_tankers = n_for_lox.max(n_for_fuel) as usize;

    // LEO depot demand: cargo Starships + EML1-bound tankers (full refill + bay cargo).
    let cargo_leo_refill = cfg.cargo.prop_cap - cfg.cargo.leo_residual;
    let eml1_tanker_leo_take = cargo_leo_refill + bay_prop;
    let leo_total_demand = n_c * cargo_leo_refill + n_eml1_tankers as f64 * eml1_tanker_leo_take;

    // Earth → LEO tankers (delivering prop in 78/22 mix as cargo).
    let n_leo_tankers = (leo_total_demand / cfg.tanker_payload).ceil() as usize;

    let total_launches = cfg.n_cargo + n_eml1_tankers + n_leo_tankers;
    let leverage = lunar_payload / total_launches as f64;

    NoTetherCycle {
        lunar_payload,
        n_eml1_tankers,
        n_leo_tankers,
        total_launches,
        leverage,
        leo_prop_demand: leo_total_demand,
        eml1_lox_balance,
        eml1_fuel_balance,
        cargo_return_prop_per_ship: cargo_eml1_total_load,
        eml1_tanker_net_per_flight: tanker_net_delivered,
    }
}

pub fn print(cfg: &Config, c: &NoTetherCycle) {
    println!("================================================================");
    println!("Comparison: no-tether architecture (full Earth refueling)");
    println!("================================================================");
    println!();
    println!("Per-cycle  (n_cargo = {}, lunar payload = {:.0} t):",
             cfg.n_cargo, c.lunar_payload);
    println!();
    println!("EML1 depot pre-tanker balance:");
    println!("  LOX:                              {:>+7.1} t   (cargo residuals − lunar − cargo return loading)",
             c.eml1_lox_balance);
    println!("  fuel:                             {:>+7.1} t",
             c.eml1_fuel_balance);
    if c.eml1_lox_balance >= -0.5 && c.eml1_fuel_balance >= -0.5 {
        println!("  → cargo Starship residuals alone cover EML1 demand; no EML1-bound tankers needed.");
    } else {
        println!("  → deficit; LEO → EML1 tankers required ({:.0} t prop net delivered per tanker).",
                 c.eml1_tanker_net_per_flight);
    }
    println!();
    println!("Cargo Starship return load (no LOX cargo on return):");
    println!("  Prop loaded at EML1:              {:>7.1} t   (return burn + landing reserve)",
             c.cargo_return_prop_per_ship);
    println!();
    println!("Earth launches per cycle:");
    println!("  Cargo Starships:                  {}", cfg.n_cargo);
    println!("  LEO → EML1 propellant tankers:    {}", c.n_eml1_tankers);
    println!("  Earth → LEO tankers:              {}", c.n_leo_tankers);
    println!("  Total:                            {}", c.total_launches);
    println!();
    println!("LEO depot demand:                   {:.0} t prop (78/22 mix from Earth)",
             c.leo_prop_demand);
    println!();
    println!("Per-launch leverage:                {:.1} t lunar / launch", c.leverage);
}
