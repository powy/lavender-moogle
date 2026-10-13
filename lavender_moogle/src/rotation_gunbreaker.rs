// rotation_gunbreaker.rs
// Rotation logic for gunbreaker job

use log::{debug, error};

use crate::memory;
use crate::spell;
use crate::types::{CharacterDetails, JobDetails, SpellcastType};

pub fn update_next_spellcast(
    character_details: &mut CharacterDetails,
    spell_cast_type: SpellcastType,
    aoe_old: bool,
    alternate_old: bool,
) {
    if let Some(job_details) = character_details.get_mut("GNB") {
        // update status

        // check oGCD
        let mut next_spell_name = "";
        let burst_allowed = spell::burst_allowed(job_details);

        // Melee arrangement
        // 1 : normal
        // 2 : AOE
        // 3 : Only dist instant Mono
        // TBC : In serious play, Only Dist may be more useful than AOE.

        let mut distance_instant_cast_only = false;
        let mut aoe = false;

        match spell_cast_type {
            SpellcastType::First => (),
            SpellcastType::Second => aoe = true,
            SpellcastType::Third => distance_instant_cast_only = true,
            SpellcastType::Fourth => (),
        }

        // No clipping?

        // prepare all test for more clear code
        // Basic combo: Keen Edge -> Brutal Shell -> Solid Barrel
        let basic_combo_2nd_brutal_shell_ready: bool =
            spell::is_status_active_next_gcd(job_details, "Brutal Shell Ready")
                && !aoe
                && !distance_instant_cast_only;
        let basic_combo_3rd_solid_barrel_ready: bool =
            spell::is_status_active_next_gcd(job_details, "Solid Barrel Ready")
                && !aoe
                && !distance_instant_cast_only;

        // Gnash combo :
        let gnash_combo_2nd_savage_claw_ready: bool =
            spell::is_status_active_next_gcd(job_details, "Savage Claw Ready")
                && !aoe
                && !distance_instant_cast_only;
        let gnash_combo_3rd_wicked_talon_ready: bool =
            spell::is_status_active_next_gcd(job_details, "Wicked Talon Ready")
                && !aoe
                && !distance_instant_cast_only;

        // lion combo, during burst
        let lion_combo_2nd_noble_blood_ready: bool =
            spell::is_status_active_next_gcd(job_details, "Noble Blood Ready")
                && !distance_instant_cast_only;
        let lion_combo_3rd_lion_heart_ready: bool =
            spell::is_status_active_next_gcd(job_details, "Lion Heart Ready")
                && !distance_instant_cast_only;

        let aoe_combo_2nd_ready =
            spell::is_status_active_next_gcd(job_details, "Demon Slaughter Ready");

        // overcap gnash
        let gnash_overcap: bool = false; // FIXME not computed yet

        // Burst status
        // 13s before no_mercy (20s no mercy + 10s pou cd + 3s margin) // For oGCD ...
        let no_mercy_soon_ogcd: bool =
            spell::spell_will_be_available_soon(job_details, "No Mercy", 13000);
        let in_burst_gcd = spell::is_status_active_next_gcd(job_details, "No Mercy Buff");

        if job_details.opener && in_burst_gcd {
            job_details.opener = false;
        }

        // OGCD casting ///////////////////////////////

        // TODO - accept some small clipping for the GNB reactional skills
        if spell::ogcd_end_before_next_gcd(job_details) {
            // --- A Priority: no drifting raid buffs
            // FIXME OPENER

            if job_details.opener && burst_allowed {
                if spell::is_spell_available_now(job_details, "Bloodfest") {
                    next_spell_name = "Bloodfest";
                } else if basic_combo_3rd_solid_barrel_ready {
                    next_spell_name = "No Mercy";
                }
            } else if spell::is_spell_available_now(job_details, "No Mercy") {
                next_spell_name = "No Mercy";

                // Buff except out of buff Bloodfest are managed already, only melee oGCD left
                // If they are forbidden only bloodfest is an option
            } else if distance_instant_cast_only && should_bloodfest(job_details, burst_allowed) {
                next_spell_name = "Bloodfest"; // fixme bug{

            // --- B No wait for reaction oGCD (continuation)
            } else if spell::is_status_active_now(job_details, "Ready to Blast") {
                next_spell_name = "Hypervelocity";
            } else if spell::is_status_active_now(job_details, "Ready to Rip") {
                next_spell_name = "Jugular Rip";
            } else if spell::is_status_active_now(job_details, "Ready to Tear") {
                next_spell_name = "Abdomen Tear";
            } else if spell::is_status_active_now(job_details, "Ready to Gouge") {
                next_spell_name = "Eye Gouge";
            } else if spell::is_status_active_now(job_details, "Ready to Raze") {
                next_spell_name = "Fated Brand";

            // FIXME Systematic pot under buff

            // -- C on Cd oGCD - with small delay to fit in no mercy especially opener
            } else if spell::is_spell_available_now(job_details, "Bow Shock") && !no_mercy_soon_ogcd
            {
                next_spell_name = "Bow Shock";
            } else if spell::is_spell_available_now(job_details, "Blasting Zone")
                && !no_mercy_soon_ogcd
            {
                next_spell_name = "Blasting Zone";
            }
            // --- D cartbridges reloader on CD. Usually few GCD of margin so it runs after the "Bow Shock" "Blasting Zone"
            // In practice they may be delayed anyway to fit in the no mercy buff window
            else if should_bloodfest(job_details, burst_allowed) {
                next_spell_name = "Bloodfest";

                // WARNING Auto-pot
            }
        }

        // Reaction oGCD used even if GCD available - clipping allowed for GNB TBC
        if next_spell_name == "" && !distance_instant_cast_only {
            // --- B No wait for reaction oGCD
            if spell::is_status_active_now(job_details, "Ready to Blast") {
                next_spell_name = "Hypervelocity";
            } else if spell::is_status_active_now(job_details, "Ready to Rip") {
                next_spell_name = "Jugular Rip";
            } else if spell::is_status_active_now(job_details, "Ready to Tear") {
                next_spell_name = "Abdomen Tear";
            } else if spell::is_status_active_now(job_details, "Ready to Gouge") {
                next_spell_name = "Eye Gouge";
            } else if spell::is_status_active_now(job_details, "Ready to Raze") {
                next_spell_name = "Fated Brand";
            }
        }

        // No oGCD selected, prepare next GCD

        // no check on EXPIRING GCD because to many combo breakers with this job

        // GCD distance
        if distance_instant_cast_only {
            next_spell_name = "Lightning Shot";
        }

        // GCD under burst ///////////////////

        if next_spell_name == "" && in_burst_gcd {
            // prevent gnash overcap (also helpful for opener)
            if gnash_overcap && can_gnashing_fang(job_details, aoe) {
                next_spell_name = "Gnashing Fang";
                // Double Down asap to avoid any drift
            } else if can_double_down(job_details) {
                next_spell_name = "Double Down";

            // Gnash will overcap in burst, ! in_gnash_combo  -> start Gnash
            // FIXME solve after charges done. Maybe start Gnash systemtically if doesn't brake with lion
            // Follow up no mercy (flexible)
            } else if spell::is_status_active_next_gcd(job_details, "Ready to Break") && !aoe {
                next_spell_name = "Sonic Break";

            // in_gnash_combo -> Finish Combo
            } else if gnash_combo_2nd_savage_claw_ready {
                next_spell_name = "Savage Claw";
            } else if gnash_combo_3rd_wicked_talon_ready {
                next_spell_name = "Wicked Talon";

            // Lion combo (if available)
            // Note : lion combo is never triggerd auto if not in burst
            } else if can_lion_combo(job_details) {
                next_spell_name = "Reign of Beasts";
            } else if lion_combo_2nd_noble_blood_ready {
                next_spell_name = "Noble Blood";
            } else if lion_combo_3rd_lion_heart_ready {
                next_spell_name = "Lion Heart";

            // Start Gnash combo (if available) (charges > 0)
            // Fixme manage charges
            } else if can_gnashing_fang(job_details, aoe) {
                next_spell_name = "Gnashing Fang";

            // empty extra cartbridges
            } else if job_details.stacks_a >= 1 {
                if aoe {
                    next_spell_name = "Fated Circle"
                } else {
                    next_spell_name = "Burst Strike";
                }
            }
        }

        // Opener GCD
        if next_spell_name == "" && job_details.opener && !aoe {
            if basic_combo_2nd_brutal_shell_ready {
                next_spell_name = "Brutal Shell";
            } else {
                next_spell_name = "Keen Edge";
            }
        }

        // normal GCD /////////////////

        if next_spell_name == "" {
            // in_gnash_combo => Finish Gnash combo
            // Gnash up, Cartbridge > 0, gnash_will_overlap_before_next_burst-> Gnash
            // overcap_cartbridge ->  Boom
            // Finish basic combo
            // Start Basic combo

            // Gnashing will overcap - FIXME

            // Sonic break out of burst - edge case
            if spell::is_status_active_next_gcd(job_details, "Ready to Break") && !aoe {
                next_spell_name = "Sonic Break";

                // Finish Lion combo. Edge case
            } else if lion_combo_2nd_noble_blood_ready {
                next_spell_name = "Noble Blood";
            } else if lion_combo_3rd_lion_heart_ready {
                next_spell_name = "Lion Heart";

            // Finish Ganshing fang combo
            } else if spell::is_status_active_next_gcd(job_details, "Savage Claw Ready") {
                next_spell_name = "Savage Claw";
            } else if spell::is_status_active_next_gcd(job_details, "Wicked Talon Ready") {
                next_spell_name = "Wicked Talon";

            // Gnash up, Cartbridge > 0, gnash_will_overlap_before_next_burst-> Gnash
            // gnash_will_overlap_before_next_burst
            // FIXME, for now, simply use gnash on CD
            } else if can_gnashing_fang(job_details, aoe) {
                next_spell_name = "Gnashing Fang";
            // overcap_cartbridge ->  Boom
            } else if !aoe
                && cartbridges_will_overcap(job_details, basic_combo_3rd_solid_barrel_ready)
            {
                next_spell_name = "Burst Strike";
            } else if aoe && cartbridges_will_overcap(job_details, aoe_combo_2nd_ready) {
                next_spell_name = "Fated Circle"
            // Basic combo
            } else if aoe {
                if spell::is_status_active_next_gcd(job_details, "Demon Slaughter Ready") {
                    next_spell_name = "Demon Slaughter";
                } else {
                    // Start combo AOE
                    next_spell_name = "Demon Slice";
                }
            } else {
                if basic_combo_3rd_solid_barrel_ready {
                    // Note: a test on cartbridge used to sit there, but not needed anymore because we never sit at 3 cartbridge after patch 7.1
                    next_spell_name = "Solid Barrel";
                } else if basic_combo_2nd_brutal_shell_ready {
                    next_spell_name = "Brutal Shell";
                } else {
                    next_spell_name = "Keen Edge";
                }
            }
        }

        if next_spell_name != "" {
            memory::communicate_next_spellcast(job_details, spell_cast_type, next_spell_name, true);
        }
    } else {
        error!("JOB not found in spell book");
    }
}

// Bloodfest is used on CD. It is delayed if we are expecting to delay the next burst. Note the danger that it will get popped several GCD before burst
fn should_bloodfest(job_details: &mut JobDetails, burst_allowed: bool) -> bool {
    let available = spell::is_spell_available_now(job_details, "Bloodfest");
    return available && burst_allowed;
}

// FIXME need to see that Double down has been used
// Note - with 7.4 Double down is used as soon as no mercy is started.
fn can_double_down(job_details: &mut JobDetails) -> bool {
    // Default behavior - 1 cartbridge = 1 double
    let enough_cartbridge = job_details.stacks_a >= 2;
    let available = spell::is_spell_available_next_gcd(job_details, "Double Down");
    return enough_cartbridge && available;
}

// FIXME, should be redone following rework 7.4
fn can_gnashing_fang(job_details: &mut JobDetails, aoe: bool) -> bool {
    let lion_combo_2nd_noble_blood_ready: bool =
        spell::is_status_active_next_gcd(job_details, "Noble Blood Ready");
    let lion_combo_3rd_lion_heart_ready: bool =
        spell::is_status_active_next_gcd(job_details, "Lion Heart Ready");
    let in_lion_combo = lion_combo_2nd_noble_blood_ready || lion_combo_3rd_lion_heart_ready;

    let gnash_combo_2nd_savage_claw_ready: bool =
        spell::is_status_active_next_gcd(job_details, "Savage Claw Ready");
    let gnash_combo_3rd_wicked_talon_ready: bool =
        spell::is_status_active_next_gcd(job_details, "Wicked Talon Ready");
    let in_gnash_combo = gnash_combo_2nd_savage_claw_ready || gnash_combo_3rd_wicked_talon_ready;

    let enough_cartbridge = job_details.stacks_a >= 1;
    let available = spell::is_spell_available_next_gcd(job_details, "Gnashing Fang");

    return !aoe && !in_gnash_combo && !in_lion_combo && enough_cartbridge && available;
}

// WARNING - does not check if we are in burst, this in done at a upper level
fn can_lion_combo(job_details: &mut JobDetails) -> bool {
    let lion_combo_1st_reign_of_beasts_ready: bool =
        spell::is_status_active_next_gcd(job_details, "Ready to Reign");
    let gnash_combo_2nd_savage_claw_ready: bool =
        spell::is_status_active_next_gcd(job_details, "Savage Claw Ready");
    let gnash_combo_3rd_wicked_talon_ready: bool =
        spell::is_status_active_next_gcd(job_details, "Wicked Talon Ready");
    let in_gnash_combo = gnash_combo_2nd_savage_claw_ready || gnash_combo_3rd_wicked_talon_ready;

    return !in_gnash_combo && lion_combo_1st_reign_of_beasts_ready;
}

// overcap_cartbridge = 3 cartbridge (6 Bloodfest Buff) and basic_combo_3rd
// may_lose_cartbridge ->
// 1) Bloodfest Buff gone in X and X + 3 cartbridge left (3 cas)
// 2) Bloodfest Buff gone in X and in basic combo (3 cas au moins)

// FIXME
fn cartbridges_will_overcap(job_details: &mut JobDetails, will_generate: bool) -> bool {
    if spell::is_status_active_next_gcd(job_details, "Bloodfest Buff") {
        // 6 cartbridges max
        let too_many_cartbridges: bool = job_details.stacks_a >= 5;
        let will_get_too_many_cartbridges: bool = will_generate && job_details.stacks_a >= 4;
        return too_many_cartbridges || will_get_too_many_cartbridges;
    } else {
        // 3 cartbridges max
        return will_generate && job_details.stacks_a >= 3;
    }
}

// note: may opti get at 0 cartbridge when Bloodfest is coming

pub fn apply_spell_additional_effects(_spell_name: &str, _job_details: &mut JobDetails) {}

pub fn update_job_gauge(job_details: &mut JobDetails, data: &str) {
    let chunks: Vec<String> = data
        .as_bytes() // Convert the string to a byte slice
        .chunks(2) // Split into chunks of 2 bytes
        .map(|chunk| String::from_utf8_lossy(chunk).to_string()) // Convert each chunk back to a String
        .collect();

    let cartridges: i32 = i32::from_str_radix(&chunks[2], 16).expect("Failed to convert chunk");
    debug!("cartridges: {}", cartridges);

    job_details.stacks_a = cartridges;
}
