// rotation_gunbreaker.rs
// Rotation logic for gunbreaker job

use log::error;

use crate::types::{CharacterDetails, JobDetails};
use crate::spell;
use crate::memory;


pub fn update_next_spellcast(character_details: &mut CharacterDetails) {
    if let Some(job_details) = character_details.get_mut("GNB") {
        // update status

        // update stack min / max

        if job_details.stacks_a > 3 {
            job_details.stacks_a = 3;
        } else if job_details.stacks_a < 0 {
            job_details.stacks_a = 0;
        }

        // check oGCD
        let mut next_spell_name = "";


        // No clipping?

        // TODO - accept some small clipping for the GNB reactional skills
        if spell::ogcd_end_before_next_gcd(job_details)
        {
            // --- A Priority: no drifting raid buffs
            if spell::is_spell_available_now(job_details, "No Mercy") {
                next_spell_name = "No Mercy";

            // --- B No wait for reaction oGCD
            } else if spell::is_status_active_now(job_details, "Ready to Blast") {
                next_spell_name = "Hypervelocity";
            } else if spell::is_status_active_now(job_details, "Ready to Rip") {
                next_spell_name = "Jugular Rip";
            } else if spell::is_status_active_now(job_details, "Ready to Tear") {
                next_spell_name = "Abdomen Tear";
            } else if spell::is_status_active_now(job_details, "Ready to Gouge") {
                next_spell_name = "Eye Gouge";

            // --- C cartbridges
            } else if should_bloodfest(job_details) {
                next_spell_name = "Bloodfest";

            // --- D ON-CD OGCD
            } else if spell::is_spell_available_now(job_details, "Bow Shock") {
                next_spell_name = "Bow Shock";
            } else if spell::is_spell_available_now(job_details, "Blasting Zone") {
                next_spell_name = "Blasting Zone";
            }
        }

        // No oGCD selected, prepare next GCD            

        // no check on EXPIRING GCD because to many combo breakers with this job

        // check normal GCD
        if next_spell_name == "" {

        // --- CARTBRIDGE usage
            // Note: Double down or gnashing first ? Chose double down to secure cardbridge usage
            if should_double_down(job_details) {
                next_spell_name = "Double Down";

            // -- Finish started combo

            //--- GNASHING FANG FOLLOW UP
            } else if spell::is_status_active_next_gcd(job_details, "Savage Claw Ready") {
                next_spell_name = "Savage Claw";
            } else if spell::is_status_active_next_gcd(job_details, "Wicked Talon Ready") {
                next_spell_name = "Wicked Talon";

            // --- LION HEART COMBO
            } else if spell::is_status_active_next_gcd(job_details, "Noble Blood Ready") {
                next_spell_name = "Noble Blood";
            } else if spell::is_status_active_next_gcd(job_details, "Lion Heart Ready") {
                next_spell_name = "Lion Heart";

            // FIXME - start of No Mercy burst depending on the cartbridge usage


            // --- CARTBRIDGE usage
            // Wait for burst if needed
            } else if should_gnashing_fang(job_details) {
                next_spell_name = "Gnashing Fang";

            // --- Sonic break in the no mercy burst
            } else if spell::is_status_active_next_gcd(job_details, "Ready to Break") {
                next_spell_name = "Sonic Break";
                
            // Wait for burst to Lion Heart
            } else if spell::is_status_active_next_gcd(job_details, "Ready to Reign")
                && !spell::spell_will_be_available_soon(job_details, "No Mercy", 10000) {
                next_spell_name = "Reign of Beasts";

            // -- STANDART ROTATION and Special burst strike cases
            // // -- Fit Burst strike in the no mercy burst
            } else if should_add_a_boom_in_the_burst(job_details) {
                next_spell_name = "Burst Strike";
            } else if should_burst_strike_to_prevent_cartbridge_overcap(job_details) {
                next_spell_name = "Burst Strike";
            } else if spell::is_status_active_next_gcd(job_details,"Solid Barrel Ready") {
                    if would_overcap_cartbridges(job_details, 1) {
                        next_spell_name = "Burst Strike";
                    } else {
                        next_spell_name = "Solid Barrel";
                    }
            } else if spell::is_status_active_next_gcd(job_details, "Brutal Shell Ready") {                
                next_spell_name = "Brutal Shell";
            } else {
                next_spell_name = "Keen Edge";
            }
        }
        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, next_spell_name);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

fn would_overcap_cartbridges(job_details: &mut JobDetails, nb_new_cartbridges: i32) -> bool {
    return job_details.stacks_a + nb_new_cartbridges > 3; // Max 3 cartbridge
}

fn should_bloodfest(job_details: &mut JobDetails) -> bool {
    // Default is if no cartbridge
    // Opti - see edge case where it should be used top not delay burst
    // FIXME - call on CD ? no drifting
    let available = spell::is_spell_available_now(job_details, "Bloodfest");
    let would_overcap = job_details.stacks_a >= 2;
    //  would_overcap_cartbridges(job_details, 3);
    // let no_mercy_soon = spell::spell_will_be_available_soon(job_details, "No Mercy", 20000);
    return available && !would_overcap;
}

fn should_double_down(job_details: &mut JobDetails) -> bool {
    // Default behavior - 2 cartbridge = 1 double
    let enough_cartbridge = job_details.stacks_a >= 2;
    let available = spell::is_spell_available_next_gcd(job_details, "Double Down");
    let no_mercy_soon = spell::spell_will_be_available_soon(job_details, "No Mercy", 20000);
    return enough_cartbridge && available && !no_mercy_soon;
}

fn should_gnashing_fang(job_details: &mut JobDetails) -> bool {
    // Default behavior - 1 cartbridge = 1 gnashing
    // Delay if no mercy is very close and no overcap? To be seen after longer usage
    let enough_cartbridge = job_details.stacks_a >= 1;
    let available = spell::is_spell_available_next_gcd(job_details, "Gnashing Fang");
    let no_mercy_soon = spell::spell_will_be_available_soon(job_details, "No Mercy", 20000);
    return enough_cartbridge && available && !no_mercy_soon;
}

fn should_burst_strike_to_prevent_cartbridge_overcap(job_details: &mut JobDetails) -> bool {
    // 1) Cartbridge reset available next GCD, already 3 cartbridge
    let bloodfest_available_soon = spell::is_spell_available_next_gcd(job_details, "Bloodfest");
    let bloodfest_would_overcap = would_overcap_cartbridges(job_details, 1);
    if bloodfest_available_soon && bloodfest_would_overcap {
        return true
    }

    // 2) Double down is too far, Cartbridge reset available next GCD, already 2 cartbridge
    let double_down_available_soon = spell::spell_will_be_available_soon(job_details, "Double Down", 5000);
    if bloodfest_available_soon && !double_down_available_soon && (job_details.stacks_a == 2) {
        return true
    }

    // 3) bloodfest already available and still some cartbridges
    if bloodfest_available_soon && (job_details.stacks_a >= 1) {
        return true
    }

    false
}

// Opti suggested by guide, but makes the wound slide
fn should_add_a_boom_in_the_burst(job_details: &mut JobDetails) -> bool {
    return spell::is_status_active_next_gcd(job_details, "No Mercy")
        && job_details.stacks_a >= 1;
    // let enough_cartbridge = job_details.stacks_a >= 1;
    // let under_no_mercy = spell::is_status_active_next_gcd(job_details, "No Mercy");
    // let end_of_no_mercy = !spell::is_status_still_active_in_two_gcd(job_details, "No Mercy");
    // return enough_cartbridge && under_no_mercy && end_of_no_mercy;
}

// note: may opti get at 0 cartbridge when Bloodfest is coming

pub fn apply_spell_additional_effects(_spell_name: &str, _job_details: &mut JobDetails) {

}