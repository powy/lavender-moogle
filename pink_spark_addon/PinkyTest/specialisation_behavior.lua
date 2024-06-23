SpecializationBehavior = {}
SpecializationBehavior.__index = SpecializationBehavior

function SpecializationBehavior:new(name, spells)
    local obj = {
        name = name,
        Spells = spells or {}
    }
    setmetatable(obj, SpecializationBehavior)
    return obj
end

function SpecializationBehavior:update_spells(character_status)
    for _, spell in pairs(self.Spells) do
        spell:update_status(character_status)
    end
end

function SpecializationBehavior:next_spell()
    -- Abstract method, to be implemented by derived classes
end
