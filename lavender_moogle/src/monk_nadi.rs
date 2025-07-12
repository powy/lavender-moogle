use crate::spell;
use crate::types::JobDetails;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Animal {
    Nothing = 0,
    Opo = 1,
    Raptor = 2,
    Coeurl = 3,
}

impl Animal {
    pub fn from_i32(value: i32) -> Animal {
        match value {
            0 => Animal::Nothing,
            1 => Animal::Opo,
            2 => Animal::Raptor,
            3 => Animal::Coeurl,
            _ => panic!("Invalid Animal value: {}", value),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct MonkNadiStats {
    animal_slots: [Animal; 3],
    lunar_nadi: bool,
    solar_nadi: bool,
    perfect_balance_charges: u8, // 0-3
}

impl MonkNadiStats {
    fn from_i32(value: i32) -> Self {
        let slot1 = ((value / 1) % 10) as u8;
        let slot2 = ((value / 10) % 10) as u8;
        let slot3 = ((value / 100) % 10) as u8;
        let lunar_nadi = ((value / 1000) % 10) != 0;
        let solar_nadi = ((value / 10_000) % 10) != 0;
        let perfect_balance_charges = ((value / 100_000) % 10) as u8;

        Self {
            animal_slots: [
                Self::decode_animal(slot1),
                Self::decode_animal(slot2),
                Self::decode_animal(slot3),
            ],
            lunar_nadi,
            solar_nadi,
            perfect_balance_charges,
        }
    }

    fn to_i32(&self) -> i32 {
        let s1 = self.animal_slots[0] as i32;
        let s2 = self.animal_slots[1] as i32;
        let s3 = self.animal_slots[2] as i32;
        let l = if self.lunar_nadi { 1 } else { 0 };
        let s = if self.solar_nadi { 1 } else { 0 };
        let pb = self.perfect_balance_charges as i32;

        s1 + s2 * 10 + s3 * 100 + l * 1000 + s * 10_000 + pb * 100_000
    }

    fn decode_animal(value: u8) -> Animal {
        match value {
            1 => Animal::Opo,
            2 => Animal::Raptor,
            3 => Animal::Coeurl,
            _ => Animal::Nothing,
        }
    }

    fn total_animal_charges(&self) -> u8 {
        self.animal_slots
            .iter()
            .filter(|a| **a != Animal::Nothing)
            .count() as u8
    }

    pub fn count_animals(&self) -> (u8, u8, u8) {
        let mut nb_opo = 0;
        let mut nb_raptor = 0;
        let mut nb_coeurl = 0;

        for slot in self.animal_slots.iter() {
            match slot {
                Animal::Opo => nb_opo += 1,
                Animal::Raptor => nb_raptor += 1,
                Animal::Coeurl => nb_coeurl += 1,
                Animal::Nothing => {} // ignore
            }
        }

        (nb_opo, nb_raptor, nb_coeurl)
    }
}

pub fn set_nadi_state(
    job_details: &mut JobDetails,
    slots: [Animal; 3],
    lunar_nadi: bool,
    solar_nadi: bool,
) {
    let mut stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    stats.animal_slots = slots;
    stats.lunar_nadi = lunar_nadi;
    stats.solar_nadi = solar_nadi;
    job_details.secondary_bar = stats.to_i32();
}

pub fn has_lunar_nadi(job_details: &JobDetails) -> bool {
    MonkNadiStats::from_i32(job_details.secondary_bar).lunar_nadi
}

pub fn has_solar_nadi(job_details: &JobDetails) -> bool {
    MonkNadiStats::from_i32(job_details.secondary_bar).solar_nadi
}

pub fn get_nb_beast_chakra(job_details: &JobDetails) -> u8 {
    return MonkNadiStats::from_i32(job_details.secondary_bar).total_animal_charges();
}

pub fn do_we_want_lunar_nadi(job_details: &mut JobDetails) -> bool {
    let stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    if spell::is_status_active_now(job_details, "Force Lunar Action") {
        return true;
    }
    if spell::is_status_active_now(job_details, "Force Solar Action") {
        return false;
    }

    if job_details.opener {
        return true;
    } else {
        if stats.solar_nadi {
            return true;
        } else if stats.lunar_nadi {
            return false;
        }
        return true;
    }
}

pub fn next_animal_in_balance_phase(job_details: &mut JobDetails) -> &'static str {
    let want_solar_nadi = !do_we_want_lunar_nadi(job_details);
    let stats = MonkNadiStats::from_i32(job_details.secondary_bar);
    let (nb_opo_chakra, nb_raptor_chakra, nb_coeurl_chakra) = stats.count_animals();
    let total = nb_opo_chakra + nb_raptor_chakra + nb_coeurl_chakra;

    let next = if want_solar_nadi {
        match total {
            0 => "Raptor",
            1 => {
                if nb_raptor_chakra == 1 {
                    "Coeurl"
                } else {
                    "Raptor"
                }
            }
            2 => {
                if nb_coeurl_chakra == 2 || nb_raptor_chakra == 2 {
                    "Opo-opo"
                } else if nb_opo_chakra == 2 {
                    "Coeurl"
                } else if nb_opo_chakra == 1 && nb_raptor_chakra == 1 {
                    "Coeurl"
                } else if nb_raptor_chakra == 1 && nb_coeurl_chakra == 1 {
                    "Opo-opo"
                } else if nb_opo_chakra == 1 && nb_coeurl_chakra == 1 {
                    "Raptor"
                } else {
                    "Raptor"
                }
            }
            _ => "Opo-opo",
        }
    } else {
        match total {
            0 => "Opo-opo",
            1 => {
                if nb_opo_chakra == 1 {
                    "Opo-opo"
                } else if nb_raptor_chakra == 1 {
                    "Raptor"
                } else if nb_coeurl_chakra == 1 {
                    "Coeurl"
                } else {
                    "Opo-opo"
                }
            }
            2 => {
                if nb_opo_chakra == 2 {
                    "Opo-opo"
                } else if nb_raptor_chakra == 2 {
                    "Raptor"
                } else if nb_coeurl_chakra == 2 {
                    "Coeurl"
                } else if nb_opo_chakra == 1 && nb_coeurl_chakra == 1 {
                    "Opo-opo"
                } else if nb_coeurl_chakra == 1 && nb_raptor_chakra == 1 {
                    "Raptor"
                } else if nb_opo_chakra == 1 && nb_raptor_chakra == 1 {
                    "Opo-opo"
                } else {
                    "Opo-opo"
                }
            }
            _ => "Opo-opo",
        }
    };

    return next;
}
