
-- local MyFrame = CreateFrame("Frame")
-- MyFrame:ClearAllPoints()
-- MyFrame:SetBackdrop(StaticPopup1:GetBackdrop())
-- MyFrame:SetHeight(300)
-- MyFrame:SetWidth(300)

-- MyFrame.text = MyFrame:CreateFontString(nil, "BACKGROUND", "GameFontNormal")
-- MyFrame.text:SetAllPoints()
-- MyFrame.text:SetText("YOUR HELP TEXT HERE")
-- MyFrame:SetPoint("CENTER", 0, 0)

-- FontString:SetFont("Fonts\\FRIZQT__.TTF", 20, "OUTLINE, MONOCHROME")

local backdropInfo =
{
	bgFile = "Interface\\Tooltips\\UI-Tooltip-Background",
 	edgeFile = "Interface\\Tooltips\\UI-Tooltip-Border",
 	tile = false,
 	tileEdge = false,
 	tileSize = 8,
 	edgeSize = 8,
 	insets = { left = 1, right = 1, top = 1, bottom = 1 },
}

local frame = CreateFrame("Frame", nil, nil, "BackdropTemplate")
frame:SetBackdrop(backdropInfo)

-- local frame = CreateFrame("Frame", "PinkSparkFrame", UIParent)
frame:SetSize(80, 50)
frame:SetPoint("TOPLEFT", 0, 0)
-- frame:SetBackdrop({
--     bgFile = "Interface\\DialogFrame\\UI-DialogBox-Background",
--     edgeFile = "Interface\\DialogFrame\\UI-DialogBox-Border",
--     tile = true, tileSize = 16, edgeSize = 16,
--     insets = { left = 4, right = 4, top = 4, bottom = 4 }
-- })

-- frame:SetBackdrop(StaticPopup1:GetBackdrop())
frame:SetBackdropColor(0, 0, 0, 1)
frame:SetMovable(true)
frame:EnableMouse(true)
frame:RegisterForDrag("LeftButton")
frame:SetScript("OnDragStart", frame.StartMoving)
frame:SetScript("OnDragStop", frame.StopMovingOrSizing)

local lines = {}
for i = 1, 6 do
    lines[i] = frame:CreateFontString(nil, "ARTWORK", "GameFontNormalSmall")
    lines[i]:SetJustifyH("LEFT")
    lines[i]:SetTextScale(0.5)
    lines[i]:SetTextColor(1, 1, 1, 1)
    if i == 1 then
        lines[i]:SetPoint("TOPLEFT", frame, "TOPLEFT", 5, -5)
    else
        lines[i]:SetPoint("TOPLEFT", lines[i-1], "BOTTOMLEFT", 0, -2)
    end
end

local lastSpellName = ""
local lastSpellTime = ""
local gcdStartTime, gcdDuration = 0, 0

local function updateInfo()
    -- Get player specialization
    local specIndex = GetSpecialization()
    local specName = specIndex and select(2, GetSpecializationInfo(specIndex)) or "None"
    
    -- Get current resource percentage
    local primaryResource = UnitPower("player")
    local primaryResourceMax = UnitPowerMax("player")
    local primaryResourcePercent = (primaryResource / primaryResourceMax) * 100

    -- Get GCD
    local gcd = gcdDuration - (GetTime() - gcdStartTime)
    if gcd < 0 then gcd = 0 end

    -- Get current Chi (specific for Monks)
    local chi = UnitPower("player", Enum.PowerType.Chi)
    local maxChi = UnitPowerMax("player", Enum.PowerType.Chi)

    local max_size = 100

    -- Helper function to truncate strings
    local function truncateString(str, maxSize)
        if #str > maxSize then
            return str:sub(1, maxSize) .. "..."
        else
            return str
        end
    end
    
    -- Update text lines with truncated strings
    lines[1]:SetText(truncateString("Spec: " .. specName, max_size))
    lines[2]:SetText(truncateString("Last Spell: " .. lastSpellName, max_size))
    lines[3]:SetText(truncateString("Primary Resource: " .. string.format("%.1f%%", primaryResourcePercent), max_size))
    lines[4]:SetText(truncateString("Current GCD: " .. string.format("%.2f", gcd), max_size))
    lines[5]:SetText(truncateString("Last Spell Time: " .. lastSpellTime, max_size))
    lines[6]:SetText(truncateString("Chi: " .. chi .. " / " .. maxChi, max_size))
    
end

frame:RegisterEvent("ADDON_LOADED")
frame:RegisterEvent("ZONE_CHANGED_NEW_AREA")
frame:RegisterEvent("PLAYER_ENTERING_WORLD")
frame:RegisterEvent("COMBAT_LOG_EVENT_UNFILTERED")
frame:RegisterEvent("UNIT_SPELLCAST_SUCCEEDED")
frame:RegisterEvent("UNIT_POWER_UPDATE")
frame:RegisterEvent("PLAYER_SPECIALIZATION_CHANGED")

frame:SetScript("OnEvent", function(self, event, arg1, arg2, arg3, arg4, arg5, arg6, arg7, arg8, arg9, arg10)
    if event == "ADDON_LOADED" and arg1 == "PinkSpark" then
        print("PinkSpark addon loaded!")
        PinkSparkFrame:Show()
        updateInfo()
    elseif event == "ZONE_CHANGED_NEW_AREA" or event == "PLAYER_ENTERING_WORLD" or event == "PLAYER_SPECIALIZATION_CHANGED" then
        updateInfo()
    elseif event == "COMBAT_LOG_EVENT_UNFILTERED" then
        local _, subevent, _, sourceGUID, _, _, _, _, destName, _, _, spellId, spellName = CombatLogGetCurrentEventInfo()
        if subevent == "SPELL_CAST_SUCCESS" and sourceGUID == UnitGUID("player") then
            lastSpellName = spellName or "Unknown"
            lastSpellTime = date("%H:%M:%S")
            gcdStartTime, gcdDuration = GetSpellCooldown(61304) -- GCD spell ID
            updateInfo()
        end
    elseif event == "UNIT_SPELLCAST_SUCCEEDED" and arg1 == "player" then
        updateInfo()
    elseif event == "UNIT_POWER_UPDATE" and arg1 == "player" then
        updateInfo()
    end
end)

-- Update time and GCD every second
local timeFrame = CreateFrame("Frame")
timeFrame:SetScript("OnUpdate", function(self, elapsed)
    self.timeSinceLastUpdate = (self.timeSinceLastUpdate or 0) + elapsed
    if self.timeSinceLastUpdate >= 1 then
        updateInfo()
        self.timeSinceLastUpdate = 0
    end
end)
