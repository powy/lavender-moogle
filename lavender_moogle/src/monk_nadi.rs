use crate::spell;
use crate::types::JobDetails;
use log::debug;

#[derive(Debug, Clone, Copy)]
struct MonkNadiStats {
    opo_charges: u8,    // 0-3
    raptor_charges: u8, // 0-3
    coeurl_charges: u8, // 0-3
    lunar_nadi: bool,
    solar_nadi: bool,
    perfect_balance_charges: u8, // 0-3
}

impl MonkNadiStats {
    fn from_i32(value: i32) -> Self {
        let opo_charges = ((value / 1) % 10) as u8;
        let raptor_charges = ((value / 10) % 10) as u8;
        let coeurl_charges = ((value / 100) % 10) as u8;
        let lunar_nadi = ((value / 1000) % 10) != 0;
        let solar_nadi = ((value / 10_000) % 10) != 0;
        let perfect_balance_charges = ((value / 100_000) % 10) as u8;

        Self {
            opo_charges,
            raptor_charges,
            coeurl_charges,
            lunar_nadi,
            solar_nadi,
            perfect_balance_charges,
        }
    }

    fn to_i32(&self) -> i32 {
        (self.opo_charges as i32)
            + (self.raptor_charges as i32) * 10
            + (self.coeurl_charges as i32) * 100
            + (if self.lunar_nadi { 1 } else { 0 }) * 1_000
            + (if self.solar_nadi { 1 } else { 0 }) * 10_000
            + (self.perfect_balance_charges as i32) * 100_000
    }

    fn total_animal_charges(&self) -> u8 {
        self.opo_charges + self.raptor_charges + self.coeurl_charges
    }
}

pub fn add_opo_charge(job_details: &mut JobDetails) {
    debug!("add_opo_charge");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    if stats.total_animal_charges() < 3 {
        stats.opo_charges += 1;
        job_details.secondary_bar = stats.to_i32();
        reduce_perfect_balance_charges(job_details);
    }
}

pub fn add_raptor_charge(job_details: &mut JobDetails) {
    debug!("add_raptor_charge");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    if stats.total_animal_charges() < 3 {
        stats.raptor_charges += 1;
        job_details.secondary_bar = stats.to_i32();
        reduce_perfect_balance_charges(job_details);
    }
}

pub fn add_coeurl_charge(job_details: &mut JobDetails) {
    debug!("add_coeurl_charge");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    if stats.total_animal_charges() < 3 {
        stats.coeurl_charges += 1;
        job_details.secondary_bar = stats.to_i32();
        reduce_perfect_balance_charges(job_details);
    }
}

pub fn activate_lunar_nadi(job_details: &mut JobDetails) {
    debug!("activate_lunar_nadi");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    stats.lunar_nadi = true;
    job_details.secondary_bar = stats.to_i32();
}

pub fn activate_solar_nadi(job_details: &mut JobDetails) {
    debug!("activate_solar_nadi");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    stats.solar_nadi = true;
    job_details.secondary_bar = stats.to_i32();
}

pub fn reset_animal_charges(job_details: &mut JobDetails) {
    debug!("reset_animal_charges");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    stats.opo_charges = 0;
    stats.raptor_charges = 0;
    stats.coeurl_charges = 0;
    job_details.secondary_bar = stats.to_i32();
}

pub fn reset_nadi(job_details: &mut JobDetails) {
    debug!("reset_nadi");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    stats.lunar_nadi = false;
    stats.solar_nadi = false;
    job_details.secondary_bar = stats.to_i32();
}

pub fn reduce_perfect_balance_charges(job_details: &mut JobDetails) {
    debug!("reduce_perfect_balance_charges");
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    if stats.perfect_balance_charges > 0 {
        stats.perfect_balance_charges -= 1;
    }
    job_details.secondary_bar = stats.to_i32();
    if stats.perfect_balance_charges == 0 {
        spell::deactivate_status(job_details, "Balance Buff");
        spell::activate_status_default_time(job_details, "Finisher Buff", 0);
    }
    // When using Perfect Balance charges, the spells don't give a form
    spell::deactivate_status(job_details, "Opo-opo Form");
    spell::deactivate_status(job_details, "Raptor Form");
    spell::deactivate_status(job_details, "Coeurl Form");
}

pub fn add_three_perfect_balance_charges(job_details: &mut JobDetails) {
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    stats.perfect_balance_charges = 3;
    job_details.secondary_bar = stats.to_i32();
}

pub fn has_lunar_nadi(job_details: &JobDetails) -> bool {
    MonkNadiStats::from_i32(job_details.secondary_bar).lunar_nadi
}

pub fn has_solar_nadi(job_details: &JobDetails) -> bool {
    MonkNadiStats::from_i32(job_details.secondary_bar).solar_nadi
}

pub fn get_perfect_balance_charges(job_details: &JobDetails) -> u8 {
    let stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    stats.perfect_balance_charges
}

fn _debug_monk_nadi(job_details: &JobDetails) {
    let stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    debug!("=== Monk Nadi Debug ===");
    debug!("Opo-opo Charges     : {}", stats.opo_charges);
    debug!("Raptor Charges      : {}", stats.raptor_charges);
    debug!("Coeurl Charges      : {}", stats.coeurl_charges);
    debug!("Lunar Nadi Active   : {}", stats.lunar_nadi);
    debug!("Solar Nadi Active   : {}", stats.solar_nadi);
    debug!("Perfect Balance     : {}", stats.perfect_balance_charges);
    debug!("Total Animal Charges: {}", stats.total_animal_charges());
    debug!("Raw secondary_bar   : {}", job_details.secondary_bar);
    debug!("======================");
}

pub fn do_we_want_lunar_nadi(job_details: &mut JobDetails) -> bool {
    let stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    // FIXME Forced even outside of opener

    if spell::is_status_active_now(job_details, "Force Lunar Action") {
        return true;
    }
    if spell::is_status_active_now(job_details, "Force Solar Action") {
        return false;
    }

    if job_details.opener {
        // In opener, default to double lunar
        return true;
    } else {
        // Default is lunar if both are active or inactive
        // If we already have a solar nadi, we want lunar.
        if stats.solar_nadi {
            return true;
        // If we already have a lunar nadi but no solar, we want solar
        } else if stats.lunar_nadi {
            return false;
        }
        // default is lunar nadi
        return true;
    }
}

// pub fn next_animal_in_balance_phase(job_details: &mut JobDetails) -> &'static str {
//     let stats = MonkNadiStats::from_i32(job_details.secondary_bar);
//     let we_want_lunar_nadi = do_we_want_lunar_nadi(job_details);

//     // TODO - the actual optimal way is to put Opo last in Solar windows

//     // NO charge, always start with Opo
//     if stats.total_animal_charges() == 0 {
//         return "Opo-opo";
//     } else if stats.total_animal_charges() == 1 {
//         if we_want_lunar_nadi {
//             // If we want lunar, we keep a different animal
//             return existing_animal_chakra(stats);
//         } else {
//             // If we want solar, we need a different animal
//             return missing_animal_chakra(stats);
//         }
//         //
//     } else if stats.total_animal_charges() == 2 {
//         // We prepare phantom rush -> just do Opo
//         if stats.lunar_nadi && stats.solar_nadi {
//             return "Opo-opo";
//         }
//         // Note - uselss test - just existing or missing depending what we want
//         // 2 same charges - Lunar way
//         if stats.opo_charges == 2 || stats.raptor_charges == 2 || stats.raptor_charges == 2 {
//             if we_want_lunar_nadi {
//                 // normal lunar
//                 return existing_animal_chakra(stats);
//             } else {
//                 // We want solar but we started lunar. Go to celestial to save phantom rush
//                 return missing_animal_chakra(stats);
//             }
//         } else {
//             // 2 different charges - Solar way
//             // stats.opo_charges == 1 || stats.raptor_charges == 1 || stats.raptor_charges == 1
//             if we_want_lunar_nadi {
//                 // We want lunar but we started solar. Go to celestial to save phantom rush
//                 return existing_animal_chakra(stats);
//             } else {
//                 // normal solar
//                 return missing_animal_chakra(stats);
//             }
//         }
//     }

//     // Default // should not reach here
//     return "Opo-opo";
// }

pub fn next_animal_in_balance_phase(job_details: &mut JobDetails) -> &'static str {
    let stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    let we_want_lunar_nadi = do_we_want_lunar_nadi(job_details);

    // TODO - the actual optimal way is to put Opo last in Solar windows

    // NO charge, always start with Opo
    if stats.total_animal_charges() == 0 {
        return "Opo-opo";
    } else if stats.total_animal_charges() == 1 {
        if we_want_lunar_nadi {
            // If we want lunar, we keep a different animal
            return existing_animal_chakra(stats);
        } else {
            // If we want solar, we need a different animal
            return missing_animal_chakra(stats);
        }
        //
    } else if stats.total_animal_charges() == 2 {
        // We prepare phantom rush -> just do Opo
        if stats.lunar_nadi && stats.solar_nadi {
            return "Opo-opo";
        }
        // Note - uselss test - just existing or missing depending what we want
        // 2 same charges - Lunar way
        if stats.opo_charges == 2 || stats.raptor_charges == 2 || stats.raptor_charges == 2 {
            if we_want_lunar_nadi {
                // normal lunar
                return existing_animal_chakra(stats);
            } else {
                // We want solar but we started lunar. Go to celestial to save phantom rush
                return missing_animal_chakra(stats);
            }
        } else {
            // 2 different charges - Solar way
            // stats.opo_charges == 1 || stats.raptor_charges == 1 || stats.raptor_charges == 1
            if we_want_lunar_nadi {
                // We want lunar but we started solar. Go to celestial to save phantom rush
                return existing_animal_chakra(stats);
            } else {
                // normal solar
                return missing_animal_chakra(stats);
            }
        }
    }

    // Default // should not reach here
    return "Opo-opo";
}

fn missing_animal_chakra(stats: MonkNadiStats) -> &'static str {
    if stats.opo_charges == 0 {
        return "Opo-opo";
    } else if stats.raptor_charges == 0 {
        return "Raptor";
    } else if stats.coeurl_charges == 0 {
        return "Coeurl";
    }
    // Default // should not reach here
    return "Opo-opo";
}

fn existing_animal_chakra(stats: MonkNadiStats) -> &'static str {
    if stats.opo_charges > 0 {
        return "Opo-opo";
    } else if stats.raptor_charges > 0 {
        return "Raptor";
    } else if stats.coeurl_charges > 0 {
        return "Coeurl";
    }
    // Default // should not reach here
    return "Opo-opo";
}

// Similar to missing - but optimisation, we should finish with opo so it has more chnace to be inside buffs
// Not active for now as to be less of a robot player
fn _next_lunar_animal_chakra(stats: MonkNadiStats) -> &'static str {
    if stats.raptor_charges == 0 {
        return "Raptor";
    } else if stats.coeurl_charges == 0 {
        return "Coeurl";
    } else if stats.opo_charges == 0 {
        return "Opo-opo";
    }
    // Default // should not reach here
    return "Opo-opo";
}
