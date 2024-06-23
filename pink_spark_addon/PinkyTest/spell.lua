Spell = {}
Spell.__index = Spell

function Spell:new(name, abbreviation, max_cd, spell_id, mana_cost, chi_cost, energy_cost)
    local obj = {
        name = name,
        abbreviation = abbreviation,
        max_cd = max_cd,
        current_cd = 0,
        ready_in = 0,
        mana_cost = mana_cost,
        chi_cost = chi_cost,
        energy_cost = energy_cost,
        spell_id = spell_id,
        charge_number = 0,
        available = false,
        currently_channeled = false
    }
    setmetatable(obj, Spell)
    return obj
end

function Spell:update_status(character_status)
    local start, duration = GetSpellCooldown(self.spell_id)
    self.current_cd = start + duration - GetTime()
    self.ready_in = self.current_cd - character_status.next_gcd_end - 0.01
    self.available = (self.charge_number >= 1) or (self.current_cd < 0)
end
