// rotation_gunbreaker.rs
// Rotation logic for gunbreaker job

use log::error;

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails};

pub fn update_next_spellcast(character_details: &mut CharacterDetails, aoe: bool) {
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

        // OGCD casting ///////////////////////////////

        // TODO - accept some small clipping for the GNB reactional skills
        if spell::ogcd_end_before_next_gcd(job_details) {
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

            // --- C cartbridges reloader on CD
            } else if should_bloodfest(job_details) {
                next_spell_name = "Bloodfest";

            // --- D ON-CD OGCD
            } else if spell::is_spell_available_now(job_details, "Bow Shock") {
                next_spell_name = "Bow Shock";
            } else if spell::is_spell_available_now(job_details, "Blasting Zone") {
                next_spell_name = "Blasting Zone";
            }
        }

        // Reaction oGCD used even if GCD available - clipping allowed
        if next_spell_name == "" {
            // --- B No wait for reaction oGCD
            if spell::is_status_active_now(job_details, "Ready to Blast") {
                next_spell_name = "Hypervelocity";
            } else if spell::is_status_active_now(job_details, "Ready to Rip") {
                next_spell_name = "Jugular Rip";
            } else if spell::is_status_active_now(job_details, "Ready to Tear") {
                next_spell_name = "Abdomen Tear";
            } else if spell::is_status_active_now(job_details, "Ready to Gouge") {
                next_spell_name = "Eye Gouge";
            }
        }

        // No oGCD selected, prepare next GCD

        // no check on EXPIRING GCD because to many combo breakers with this job

        // GCD under burst ///////////////////

        // normal GCD /////////////////

        // check normal GCD
        if next_spell_name == "" {
            // --- CARTBRIDGE usage
            // Note: Gnashing fang before double down - evolution of patch 7.1
            // "should conditions include delaying if burst is coming"
            if should_gnashing_fang(job_details) {
                next_spell_name = "Gnashing Fang";
            } else if should_double_down(job_details) {
                // FIXME DOUBLE DOWN AVAILABLE FAIL
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
            // Lion Heart during burst after Gnashing fang combo
            // If the burst is not started, don't trigger it. The spell is available BEFORE the burst starts
            } else if spell::is_status_active_next_gcd(job_details, "Ready to Reign")
                && !spell::spell_will_be_available_soon(job_details, "No Mercy", 10000)
            {
                next_spell_name = "Reign of Beasts";

            // --- Sonic break in the no mercy burst - no need to test it because it can't be otherwise
            } else if spell::is_status_active_next_gcd(job_details, "Ready to Break") {
                next_spell_name = "Sonic Break";

            // -- STANDART ROTATION and Special burst strike cases
            // // -- Fit Burst strike in the no mercy burst
            } else if should_add_a_boom_in_the_burst(job_details) {
                // Still in burst and some GCD + cartbridge left, let's spend them
                next_spell_name = "Burst Strike";
            } else if job_details.stacks_a >= 3 {
                // Never sit at 3 cartbridge after patch 7.1
                next_spell_name = "Burst Strike";
            } else if spell::is_status_active_next_gcd(job_details, "Solid Barrel Ready") {
                // Note: a test on cartbridge used to sit there, but not needed anymore because we never sit at 3 cartbridge after patch 7.1
                next_spell_name = "Solid Barrel";
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

fn should_bloodfest(job_details: &mut JobDetails) -> bool {
    let available = spell::is_spell_available_now(job_details, "Bloodfest");

    // If available and no overcap - let's use it
    // expected : first use at pull
    // later used when we reach 0 cartbridge in burst
    // Should happen after Double down and gnashing fang in a 2 m combo
    if available && job_details.stacks_a == 0 {
        return true;
    }

    // Error case - starting No mercy with 3 cartbridge
    // Double down just been used
    // Gnashing fang has been used
    // Still one cartbridge left
    // Ideally, burst strike in combo befor using this

    // If everything goes well, we stay at 2 cartbridge all the time
    // And this should be casted only right after double down.
    // Top ensure no drifting, just call after double down...

    let double_down_used = !spell::spell_will_be_available_soon(job_details, "Double Down", 30000);
    let in_burst = spell::is_status_active_next_gcd(job_details, "No Mercy Buff");
    if available && double_down_used && in_burst && job_details.stacks_a <= 2 {
        return true;
    }

    false
}

// FIXME need to see that Double down has been used
fn should_double_down(job_details: &mut JobDetails) -> bool {
    // Default behavior - 1 cartbridge = 1 double
    let enough_cartbridge = job_details.stacks_a >= 1;
    let available = spell::is_spell_available_next_gcd(job_details, "Double Down");
    // let available = spell::is_status_active_next_gcd(job_details, "Ready to Reign");
    let no_mercy_soon = spell::spell_will_be_available_soon(job_details, "No Mercy", 30000);
    return enough_cartbridge && available && !no_mercy_soon;
}

fn should_gnashing_fang(job_details: &mut JobDetails) -> bool {
    // Default behavior - 1 cartbridge = 1 gnashing
    // Delay if no mercy is very close and no overcap? To be seen after longer usage
    // With perfect usage, never need to delay gnashing. Otherwise, the logic would be to delay it if we can't fit a full combo in no mercy
    // 7 s of combo - 20s of no mercy - 13s left on CD at start of no mercy
    // 30s of gnashing CD => 17s max wait
    let enough_cartbridge = job_details.stacks_a >= 1;
    let available = spell::is_spell_available_next_gcd(job_details, "Gnashing Fang");
    // let no_mercy_soon = spell::spell_will_be_available_soon(job_details, "No Mercy", 17000);
    let no_mercy_soon = spell::spell_will_be_available_soon(job_details, "No Mercy", 10000);
    return enough_cartbridge && available && !no_mercy_soon;
    // return enough_cartbridge && available;
}

// Opti suggested by guide, but makes the wound slide
fn should_add_a_boom_in_the_burst(job_details: &mut JobDetails) -> bool {
    return spell::is_status_active_next_gcd(job_details, "No Mercy Buff")
        && job_details.stacks_a >= 1;
}

// note: may opti get at 0 cartbridge when Bloodfest is coming

pub fn apply_spell_additional_effects(_spell_name: &str, _job_details: &mut JobDetails) {}

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    // debug!("Job gauge update: {}", data);
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    // for (i, chunk) in chunks.iter().enumerate() {
    //     let chunk_int: i32 = i32::from_str_radix(chunk, 16).expect("Failed to convert chunk");
    //     println!("Chunk {}: {}", i + 1, chunk_int);
    // }

    let cartridges: i32 = i32::from_str_radix(&chunks[1], 16).expect("Failed to convert chunk");

    job_details.stacks_a = cartridges;
}
