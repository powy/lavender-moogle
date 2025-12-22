; Requires AutoHotKey 2
; Simple keyboard spam test script
; Press and hold a key to spam another key repeatedly

; =============================================================================
; CONFIGURATION
; =============================================================================

; Spam interval in milliseconds (100ms = 10 times per second)
global SPAM_INTERVAL := 100

; Target key to spam (can be changed to any key: "1", "2", "4", "Space", "e", etc.)
global TARGET_KEY := "4"

; =============================================================================
; STATE TRACKING
; =============================================================================

; Track whether script is running
global scriptActive := true

; Track spam state for each trigger key
global spamming_X := false
global spamming_Z := false
global spamming_C := false

; =============================================================================
; HOTKEYS - Press and hold to spam
; =============================================================================

; X key - Spam the target key while held
X:: {
    global spamming_X
    spamming_X := true
    SetTimer(SpamTargetKey_X, SPAM_INTERVAL)
}

X Up:: {
    global spamming_X
    spamming_X := false
    SetTimer(SpamTargetKey_X, 0)  ; Stop timer
}

; Z key - Alternative spam trigger (spams "1" key)
Z:: {
    global spamming_Z
    spamming_Z := true
    SetTimer(SpamKey1_Z, SPAM_INTERVAL)
}

Z Up:: {
    global spamming_Z
    spamming_Z := false
    SetTimer(SpamKey1_Z, 0)
}

; C key - Another alternative (spams "2" key)
C:: {
    global spamming_C
    spamming_C := true
    SetTimer(SpamKey2_C, SPAM_INTERVAL)
}

C Up:: {
    global spamming_C
    spamming_C := false
    SetTimer(SpamKey2_C, 0)
}

; =============================================================================
; TEST HOTKEYS - Single press actions
; =============================================================================

; F1 - Test single keypress
F1:: {
    SendEvent(TARGET_KEY)
    ToolTip("Sent: " TARGET_KEY)
    SetTimer(() => ToolTip(), -1000)  ; Clear tooltip after 1 second
}

; F2 - Change spam interval (slower)
F2:: {
    global SPAM_INTERVAL
    SPAM_INTERVAL := SPAM_INTERVAL + 50
    if (SPAM_INTERVAL > 1000)
        SPAM_INTERVAL := 1000
    ToolTip("Spam Interval: " SPAM_INTERVAL "ms")
    SetTimer(() => ToolTip(), -2000)
}

; F3 - Change spam interval (faster)
F3:: {
    global SPAM_INTERVAL
    SPAM_INTERVAL := SPAM_INTERVAL - 50
    if (SPAM_INTERVAL < 50)
        SPAM_INTERVAL := 50
    ToolTip("Spam Interval: " SPAM_INTERVAL "ms")
    SetTimer(() => ToolTip(), -2000)
}

; F4 - Display current configuration
F4:: {
    global SPAM_INTERVAL, TARGET_KEY
    MsgBox(
        "=== Spam Test Configuration ===`n`n"
        "Target Key: " TARGET_KEY "`n"
        "Spam Interval: " SPAM_INTERVAL "ms (" Round(1000/SPAM_INTERVAL, 1) " keys/sec)`n`n"
        "=== Controls ===`n"
        "Hold X: Spam key " TARGET_KEY "`n"
        "Hold Z: Spam key 1`n"
        "Hold C: Spam key 2`n`n"
        "F1: Test single keypress`n"
        "F2: Slower (-50ms)`n"
        "F3: Faster (+50ms)`n"
        "F4: Show this help`n"
        "Insert: Exit script"
    )
}

; =============================================================================
; SPAM FUNCTIONS
; =============================================================================

; Spam target key when X is held
SpamTargetKey_X() {
    global scriptActive, spamming_X, TARGET_KEY
    if (scriptActive && spamming_X) {
        SendEvent(TARGET_KEY)
    }
}

; Spam key "1" when Z is held
SpamKey1_Z() {
    global scriptActive, spamming_Z
    if (scriptActive && spamming_Z) {
        SendEvent("1")
    }
}

; Spam key "2" when C is held
SpamKey2_C() {
    global scriptActive, spamming_C
    if (scriptActive && spamming_C) {
        SendEvent("2")
    }
}

; =============================================================================
; UTILITY FUNCTIONS
; =============================================================================

; Advanced spam function with modifier support
; Example: SpamKeyWithModifier("4", "Ctrl")
SpamKeyWithModifier(key, modifier := "") {
    if (modifier != "") {
        SendEvent("{" modifier " down}")
        Sleep(1)
    }

    SendEvent(key)

    if (modifier != "") {
        Sleep(1)
        SendEvent("{" modifier " up}")
    }
}

; Burst spam function - sends key N times rapidly
; Example: BurstSpam("4", 5) sends "4" five times
BurstSpam(key, count := 5) {
    Loop count {
        SendEvent(key)
        Sleep(10)
    }
}

; =============================================================================
; EXIT HOTKEY
; =============================================================================

; Insert key - Stop and exit script
Insert:: {
    global scriptActive
    scriptActive := false

    ; Stop all timers
    SetTimer(SpamTargetKey_X, 0)
    SetTimer(SpamKey1_Z, 0)
    SetTimer(SpamKey2_C, 0)

    MsgBox("Spam test script stopped!")
    ExitApp
}

; =============================================================================
; STARTUP MESSAGE
; =============================================================================

; Show help on startup
ToolTip(
    "Spam Test Script Active!`n`n"
    "Hold X to spam key " TARGET_KEY "`n"
    "Press F4 for help`n"
    "Press Insert to exit"
)
SetTimer(() => ToolTip(), -3000)  ; Clear after 3 seconds
