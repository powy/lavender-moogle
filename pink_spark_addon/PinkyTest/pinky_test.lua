local frame = CreateFrame("Frame", "PinkyTestFrame", UIParent)
frame:SetSize(200, 200)
frame:SetPoint("CENTER")

local character_status = CharacterStatus:new(100780)
local mistweaver_behavior = MistweaverBehavior()

local function update_next_spell()
    character_status:update_status()
    mistweaver_behavior:update_spells(character_status)
    next_spell = mistweaver_behavior:next_spell()
end

frame:SetScript("OnEvent", function(self, event, ...)
    update_next_spell()
end)

frame:RegisterEvent("PLAYER_REGEN_ENABLED")
frame:RegisterEvent("PLAYER_REGEN_DISABLED")
frame:RegisterEvent("ZONE_CHANGED")
frame:RegisterEvent("ZONE_CHANGED_NEW_AREA")
frame:RegisterEvent("UNIT_POWER_UPDATE")
frame:RegisterEvent("SPELL_UPDATE_COOLDOWN")
frame:RegisterEvent("UNIT_SPELLCAST_SUCCEEDED")
frame:RegisterEvent("UNIT_SPELLCAST_INTERRUPTED")

local spell_frames = {}
local yOffset = -20

for spell_id, spell in pairs(mistweaver_behavior.Spells) do
    local spell_frame = CreateFrame("Frame", nil, frame)
    spell_frame:SetSize(200, 20)
    spell_frame:SetPoint("TOP", 0, yOffset)

    spell_frame.text = spell_frame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    spell_frame.text:SetPoint("LEFT", 5, 0)

    spell_frames[spell_id] = spell_frame
    yOffset = yOffset - 20
end

local next_spell_text = frame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
next_spell_text:SetPoint("TOP", 0, yOffset - 20)
next_spell_text:SetText("Next Spell: ")

local spec_text = frame:CreateFontString(nil, "OVERLAY", "GameFontNormal")
spec_text:SetPoint("TOP", 0, yOffset - 40)
spec_text:SetText("Specialization: Mistweaver")

frame:SetScript("OnUpdate", function()
    for spell_id, spell in pairs(mistweaver_behavior.Spells) do
        local spell_frame = spell_frames[spell_id]
        spell_frame.text:SetText(string.format("%s: CD: %.1f, Available: %s", spell.name, spell.current_cd, tostring(spell.available)))
    end
    next_spell_text:SetText(string.format("Next Spell: %d", next_spell))
end)
