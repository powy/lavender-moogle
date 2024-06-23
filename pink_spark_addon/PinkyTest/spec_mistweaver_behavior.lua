MistweaverBehavior = SpecializationBehavior:new("Mistweaver", {
    [100780] = Spell:new("Tiger Palm", "TP", 0, 100780, 0, 0, 50),
    [100784] = Spell:new("Blackout Kick", "BK", 0, 100784, 0, 1, 30),
    [107428] = Spell:new("Rising Sun Kick", "RSK", 0, 107428, 0, 2, 40)
})
MistweaverBehavior.__index = MistweaverBehavior

setmetatable(MistweaverBehavior, {
    __index = SpecializationBehavior,
    __call = function(cls, ...)
        return cls:new(...)
    end,
})

function MistweaverBehavior:next_spell()
    if self.Spells[107428].available then
        return self.Spells[107428].spell_id
    elseif self.Spells[100784].available then
        return self.Spells[100784].spell_id
    elseif self.Spells[100780].available then
        return self.Spells[100780].spell_id
    else
        return 0
    end
end
