CharacterStatus = {}
CharacterStatus.__index = CharacterStatus

function CharacterStatus:new(reference_gcd_spell_id)
    local obj = {
        mana = 0,
        energy = 0,
        chi = 0,
        expected_energy = 0,
        reference_gcd_spell_id = reference_gcd_spell_id,
        next_gcd_end = 0
    }
    setmetatable(obj, CharacterStatus)
    return obj
end

function CharacterStatus:update_status()
    self:update_mana()
    self:update_chi()
    self:update_energy()
    self:update_next_gcd_end()
end

function CharacterStatus:update_mana()
    self.mana = UnitPower("player", SPELL_POWER_MANA)
end

function CharacterStatus:update_chi()
    self.chi = UnitPower("player", SPELL_POWER_CHI)
end

function CharacterStatus:update_energy()
    self.energy = UnitPower("player", SPELL_POWER_ENERGY)
end

function CharacterStatus:update_next_gcd_end()
    local start, duration = GetSpellCooldown(self.reference_gcd_spell_id)
    self.next_gcd_end = start + duration
end
