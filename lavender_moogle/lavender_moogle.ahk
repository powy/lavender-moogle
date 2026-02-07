; Requires AutoHotKey 2
#Requires AutoHotkey v2.0
#Include ..\XInput.ahk

; Configuration directives for robust spam functionality
#MaxThreadsPerHotkey 2  ; Allow hotkey to interrupt itself
SendMode "Event"        ; Explicit send mode
SetKeyDelay -1          ; Remove artificial delays

; Read values in the shared memory to execute a command modifier + keystroke
; There is 4 possible actions, depending on the modifiers used at the time of pressing the key
; The goal is to automate some action decision - the next action to do is written in the shared memory.
; On key pressing the asked keypress is simulated.

; Define constants describing the modifier "enum" as read in the shared memory
MODIFIER_NONE := "n"
MODIFIER_ALT := "a"
MODIFIER_SHIFT := "s"
MODIFIER_CTRL := "c"

; Controller polling configuration
global SPAM_INTERVAL := 150  ; milliseconds (100ms = 10 times/second)
global CONTROLLER_POLL_RATE := SPAM_INTERVAL  ; Milliseconds between polls (100ms = 10Hz)

; Used to block actions after stopping the script and before init
global stopScript := true

; Controller state tracking for edge detection
global lastControllerButtons := 0
global lastLeftTrigger := 0
global lastRightTrigger := 0
global controllerInitialized := false

; Wait 5 seconds in case it is started at the same time than the shared memory owner
; It allows time for the shared memory to be inbitialized
Sleep 5000
stopScript := false

; Shared Memory Mapping setup
; This script is the reader, the Shared Memory shall be created, Written and kept alive outside
mapping := FileMapping("Global\lavender_keys", 0xF001F, 0x4, 8)

; Controller polling variables (MUST be declared before use)
pollMs := 20        ; fast detection
repeatMs := 175     ; how often action may run
lastSend := 0
sending := false

; Initialize XInput for controller support
try {
    XInput_Init()
    controllerInitialized := true
    ; Start polling controller
    SetTimer(PollTrigger, pollMs)
    ; SetTimer(PollController, CONTROLLER_POLL_RATE)
    ; SetTimer(PollFunctions, CONTROLLER_POLL_RATE)
} catch Error as err {
    ; Controller not available - not a fatal error, script continues
    controllerInitialized := false
}

; Hotkey to stop the script
Insert:: {
    stopScript := true
    if (controllerInitialized) {
        SetTimer(PollTrigger, 0)    ; Stop the timer first
        Sleep(200)                      ; Wait for any in-flight timer calls
        XInput_Term()                   ; Then safely unload DLL
    }
    MsgBox "Your computer tastes Lavender. The color."
    ExitApp
}

; Hotkeys to read from shared memory and execute the order
; Browser_Search ko
; Browser_Favorites ok
; Browser_Refresh ok
; Browser_Stop ok
; Browser_Back ok
; Browser_Forward ok
; Browser_Home ko
Browser_Favorites:: {
    executeRaw(0, 1)
}

Browser_Refresh:: {
    executeRaw(2, 3)
}

Browser_Back:: {
    executeRaw(4, 5)
}

Browser_Forward:: {
    executeRaw(6, 7)
}

; Spam executeRaw() calls using robust While+GetKeyState pattern
; Hold each key to spam its corresponding shared memory action

; F13:: {
;     executeRaw(0, 1)  ; Execute immediately
;     while GetKeyState("F13", "P") {
;         Sleep(SPAM_INTERVAL)
;         executeRaw(0, 1)  ; Spam the executeRaw call
;     }
; }

; F14:: {
;     executeRaw(4, 5)
;     while GetKeyState("F14", "P") {
;         Sleep(SPAM_INTERVAL)
;         executeRaw(4, 5)
;     }
; }

; F15:: {
;     executeRaw(2, 3)
;     while GetKeyState("F15", "P") {
;         Sleep(SPAM_INTERVAL)
;         executeRaw(2, 3)
;     }
; }

; F16:: {
;     executeRaw(6, 7)
;     while GetKeyState("F16", "P") {
;         Sleep(SPAM_INTERVAL)
;         executeRaw(6, 7)
;     }
; }

; SetTimer(PollTrigger, pollMs)

PollTrigger() {
    global lastSend, repeatMs, sending, controllerInitialized, stopScript

    if sending
        return

    if (A_TickCount - lastSend < repeatMs)
        return

    lastSend := A_TickCount
    sending := true

    ; Actual call - could be reoragnized TODO
    if GetKeyState("F13") {
        executeRaw(0, 1)
    } else if GetKeyState("F14") {
        executeRaw(2, 3)
    } else if GetKeyState("F15") {
        executeRaw(4, 5)
    } else if GetKeyState("F16") {
        executeRaw(6, 7)
    } else {
        CheckController()
    }

    ; FIXME : add gamepad work maybe

    sending := false
}

executeRaw(modifier_pos, key_pos) {
    if stopScript
        return

    buf := Buffer(8)
    mapping.ReadToBuffer(buf)

    modifier_char := Chr(NumGet(buf, modifier_pos, "UChar"))
    key_char := Chr(NumGet(buf, key_pos, "UChar"))

    target_modifier := GetModifierString(modifier_char)
    if (target_modifier = "invalid")
        return

    ; ---- HARD logical cleanup (intentional) ----
    SendEvent "{Ctrl up}{Shift up}{Alt up}"
    Sleep 10

    ; ---- Latch modifier ----
    if target_modifier {
        SendEvent "{" target_modifier " down}"
        Sleep 90      ; CRITICAL
    }

    ; ---- Send key ----
    SendEvent key_char
    Sleep 30

    ; ---- Release modifier ----
    if target_modifier
        SendEvent "{" target_modifier " up}"
}

CheckController() {
    global lastControllerButtons, lastLeftTrigger, lastRightTrigger, controllerInitialized, stopScript

    if (stopScript || !controllerInitialized)
        return

    ; Get controller state (controller 0)
    state := XInput_GetState(0)

    ; Controller disconnected
    if (!state) {
        lastControllerButtons := 0
        lastLeftTrigger := 0
        lastRightTrigger := 0
        return
    }

    currentButtons := state.wButtons
    currentLeftTrigger := state.bLeftTrigger
    currentRightTrigger := state.bRightTrigger

    l1Pressed := CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_LEFT_SHOULDER)
    ; l2Pressed := CheckTriggerPressed(currentLeftTrigger, lastLeftTrigger)
    r1Pressed := CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_RIGHT_SHOULDER)
    ; r2Pressed := CheckTriggerPressed(currentRightTrigger, lastRightTrigger)
    trianglePressed := CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_Y)

    if (l1Pressed && trianglePressed) {
        executeRaw(0, 1)
    } else if (r1Pressed && trianglePressed) {
        executeRaw(2, 3)
    }

    ; Save current state for next poll (edge detection)
    lastControllerButtons := currentButtons
    lastLeftTrigger := currentLeftTrigger
    lastRightTrigger := currentRightTrigger
}

; Helper function to check if a button was just pressed (rising edge)
; Returns true if button is currently pressed but wasn't pressed last poll
CheckButtonPressed(currentButtons, lastButtons, buttonMask) {
    ; return (currentButtons & buttonMask) && !(lastButtons & buttonMask)
    return (currentButtons & buttonMask)
}

; Helper function to check if a trigger was just pressed (rising edge)
; Returns true if trigger crossed the threshold from below to above
CheckTriggerPressed(currentValue, lastValue, threshold := 30) {
    return (currentValue >= threshold) && (lastValue < threshold)
}

; 2025-12-20: Poll Xbox controller buttons A, B, X, Y and shoulder buttons L1, R1
; All face buttons trigger the same action (executeRaw 0,1 like F13/Browser_Favorites)
; L1 and R1 shoulder buttons can trigger different actions
PollController() {
    global lastControllerButtons, lastLeftTrigger, lastRightTrigger, controllerInitialized, stopScript

    if (stopScript || !controllerInitialized)
        return

    ; Get controller state (controller 0)
    state := XInput_GetState(0)

    ; Controller disconnected
    if (!state) {
        lastControllerButtons := 0
        lastLeftTrigger := 0
        lastRightTrigger := 0
        return
    }

    currentButtons := state.wButtons
    currentLeftTrigger := state.bLeftTrigger
    currentRightTrigger := state.bRightTrigger

    ; DEBUG: Check if L1 (Left Shoulder) was pressed
    if (CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_LEFT_SHOULDER)) {
        ToolTip "DEBUG: L1 (Left Shoulder) pressed!", 10, 10
        SetTimer(() => ToolTip(), -2000)  ; Clear tooltip after 2 seconds
        ; L1 pressed - trigger specific action
        ; executeRaw(0, 1)  ; COMMENTED for testing
    }

    ; DEBUG: Check if R1 (Right Shoulder) was pressed
    if (CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_RIGHT_SHOULDER)) {
        ToolTip "DEBUG: R1 (Right Shoulder) pressed!", 10, 30
        SetTimer(() => ToolTip(), -2000)  ; Clear tooltip after 2 seconds
        ; R1 pressed - trigger different action
        ; executeRaw(4, 5)  ; COMMENTED for testing
    }

    ; DEBUG: Check if L2 (Left Trigger) was pressed
    if (CheckTriggerPressed(currentLeftTrigger, lastLeftTrigger)) {
        ToolTip "DEBUG: L2 (Left Trigger) pressed!", 10, 50
        SetTimer(() => ToolTip(), -2000)  ; Clear tooltip after 2 seconds
        ; L2 pressed - trigger specific action
        ; executeRaw(2, 3)  ; COMMENTED for testing
    }

    ; DEBUG: Check if R2 (Right Trigger) was pressed
    if (CheckTriggerPressed(currentRightTrigger, lastRightTrigger)) {
        ToolTip "DEBUG: R2 (Right Trigger) pressed!", 10, 70
        SetTimer(() => ToolTip(), -2000)  ; Clear tooltip after 2 seconds
        ; R2 pressed - trigger different action
        ; executeRaw(6, 7)  ; COMMENTED for testing
    }

    ; Save current state for next poll (edge detection)
    lastControllerButtons := currentButtons
    lastLeftTrigger := currentLeftTrigger
    lastRightTrigger := currentRightTrigger
}

; Display full memory contents for debugging
DumpMemory() {
    buf := Buffer(8)
    mapping.ReadToBuffer(buf)
    hexStr := ""
    charStr := ""

    ; Fixed loop syntax for AutoHotkey 2
    loop 8 {
        i := A_Index - 1  ; Zero-based index
        byteVal := NumGet(buf, i, "UChar")
        hexStr .= Format("{:02X} ", byteVal)
        ; For printable characters, show them; otherwise show a dot
        charStr .= (byteVal >= 32 && byteVal <= 126) ? Chr(byteVal) : "."
    }

    MsgBox "Shared memory contents (8 bytes):`nHex: " hexStr "`nChar: " charStr
}

; Functions to get the string identifier understood by AHK based on the API value
GetModifierString(modifier) {
    switch modifier {
        case MODIFIER_NONE:
            return ""
        case MODIFIER_ALT:
            return "Alt"
        case MODIFIER_SHIFT:
            return "Shift"
        case MODIFIER_CTRL:
            return "Ctrl"
        default:
            return "invalid"
    }
}

; Management of the Shared memory reading
; This class is used almost as is from the tutoriel on the official forum
; based on https://www.autohotkey.com/boards/viewtopic.php?f=96&t=124720
class FileMapping {
    ; http://msdn.microsoft.com/en-us/library/windows/desktop/aa366556(v=vs.85).aspx
    ; http://www.autohotkey.com/board/topic/86771-i-want-to-share-var-between-2-processes-how-to-copy-memory-do-it/#entry552031
    ; Source: https://www.autohotkey.com/board/topic/93305-filemapping-class/

    __New(szName?, dwDesiredAccess := 0xF001F, flProtect := 0x4, dwSize := 8) {    ; Opens existing or creates new file mapping object with FILE_MAP_ALL_ACCESS, PAGE_READ_WRITE
        static INVALID_HANDLE_VALUE := -1
        this.BUF_SIZE := dwSize, this.szName := szName ?? ""
        if !(this.hMapFile := DllCall("OpenFileMapping", "Ptr", dwDesiredAccess, "Int", 0, "Ptr", IsSet(szName) ?
            StrPtr(szName) : 0)) {
            ; OpenFileMapping Failed - file mapping object doesn't exist
            throw Error("Unable to open the file mapping", -1)
        }
        if !(this.pBuf := DllCall("MapViewOfFile", "Ptr", this.hMapFile, "Int", dwDesiredAccess, "Int", 0, "Int", 0,
            "Int", dwSize))    ; MapViewOfFile Failed
            throw Error("Unable to map view of file")
    }

    Write(data, offset := 0) {
        if (this.pBuf) {
            if data is String
                StrPut(data, this.pBuf + offset, this.BUF_SIZE - offset, "CP0")
            else if data is Buffer
                DllCall("RtlCopyMemory", "ptr", this.pBuf + offset, "ptr", data, "int", Min(data.Size, this.BUF_SIZE -
                    offset))
            else
                throw TypeError("The data type can be a string or a Buffer object")
        } else
            throw Error("File already closed!")
    }

    ; The original Read method that uses StrGet (has issues with null bytes)
    Read(buffer?, offset := 0, size?) => IsSet(buffer) ? DllCall("RtlCopyMemory", "ptr", buffer, "ptr", this.pBuf +
        offset, "int", Min(buffer.size, this.BUF_SIZE - offset, size ?? this.BUF_SIZE - offset)) : StrGet(this.pBuf +
            offset, size ?? this.BUF_SIZE - offset, "CP0")

    ; reads directly into a buffer without string interpretation
    ReadToBuffer(buf, offset := 0) {
        if (this.pBuf) {
            DllCall("RtlCopyMemory", "ptr", buf, "ptr", this.pBuf + offset, "int", Min(buf.Size, this.BUF_SIZE - offset
            ))
            return buf.Size
        } else
            throw Error("File already closed!")
    }

    Close() {
        DllCall("UnmapViewOfFile", "Ptr", this.pBuf), DllCall("CloseHandle", "Ptr", this.hMapFile)
        this.szName := "", this.BUF_SIZE := "", this.hMapFile := "", this.pBuf := ""
    }
    __Delete() => this.Close()
}
