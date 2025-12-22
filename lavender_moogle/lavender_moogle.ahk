; Requires AutoHotKey 2
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
global CONTROLLER_POLL_RATE := 100  ; Milliseconds between polls (100ms = 10Hz)

; Used to block actions after stopping the script and before init
global stopScript := true

; Controller state tracking for edge detection
global lastControllerButtons := 0
global controllerInitialized := false

; Wait 5 seconds in case it is started at the same time than the shared memory owner
; It allows time for the shared memory to be inbitialized
Sleep 5000
stopScript := false

; Shared Memory Mapping setup
; This script is the reader, the Shared Memory shall be created, Written and kept alive outside
; TEMPORARY: Commented out for testing without admin rights - UNCOMMENT after testing
; mapping := FileMapping("Global\lavender_keys", 0xF001F, 0x4, 8)

; Initialize XInput for controller support
try {
    XInput_Init()
    controllerInitialized := true
    ; Start polling controller
    SetTimer(PollController, CONTROLLER_POLL_RATE)
} catch Error as err {
    ; Controller not available - not a fatal error, script continues
    controllerInitialized := false
}

; Hotkey to stop the script
Insert:: {
    stopScript := true
    if (controllerInitialized)
        XInput_Term()
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

F13:: {
    executeRaw(0, 1)  ; Execute immediately
    While GetKeyState("F13", "P") {
        Sleep(100)
        executeRaw(0, 1)  ; Spam the executeRaw call
    }
}

F14:: {
    executeRaw(4, 5)
    While GetKeyState("F14", "P") {
        Sleep(100)
        executeRaw(4, 5)
    }
}

F15:: {
    executeRaw(2, 3)
    While GetKeyState("F15", "P") {
        Sleep(100)
        executeRaw(2, 3)
    }
}

F16:: {
    executeRaw(6, 7)
    While GetKeyState("F16", "P") {
        Sleep(100)
        executeRaw(6, 7)
    }
}

; 2025-12-19: Controller test hotkeys using legacy Joy1-Joy32
; WARNING: These Joy hotkeys only work when this AHK window is in FOCUS
; For modern Xbox controllers (2013+), they DON'T work in background mode
; This is a Windows limitation with XInput controllers and DirectInput API
; For background controller support, use XInput.ahk library instead
; See: controller_test.ahk for a working XInput example
;
; These are kept for basic connectivity testing only - not for production use

Joy1:: {
    MsgBox "Joy1 pressed (A/Cross/B)`n`nWARNING: Only works when AHK window is focused!"
}

Joy2:: {
    MsgBox "Joy2 pressed (B/Circle/A)`n`nWARNING: Only works when AHK window is focused!"
}

Joy3:: {
    MsgBox "Joy3 pressed (X/Square/Y)`n`nWARNING: Only works when AHK window is focused!"
}

Joy4:: {
    MsgBox "Joy4 pressed (Y/Triangle/X)`n`nWARNING: Only works when AHK window is focused!"
}

Joy5:: {
    MsgBox "Joy5 pressed (LB/L1)`n`nWARNING: Only works when AHK window is focused!"
}

Joy6:: {
    MsgBox "Joy6 pressed (RB/R1)`n`nWARNING: Only works when AHK window is focused!"
}

Joy7:: {
    MsgBox "Joy7 pressed (Back/Select/Minus)`n`nWARNING: Only works when AHK window is focused!"
}

Joy8:: {
    MsgBox "Joy8 pressed (Start/Plus)`n`nWARNING: Only works when AHK window is focused!"
}

; Execute the order using raw memory reading (byte by byte)
; Positions are zero-based (0-7) as in Rust
executeRaw(modifier_pos, key_pos) {
    if (!stopScript) {
        ; Get a raw buffer of all 8 bytes
        buf := Buffer(8)
        mapping.ReadToBuffer(buf)

        ; Extract the specific bytes we need
        modifier_byte := NumGet(buf, modifier_pos, "UChar")
        key_byte := NumGet(buf, key_pos, "UChar")

        ; Convert to character
        modifier_char := Chr(modifier_byte)
        key_char := Chr(key_byte)

        ; Process the keypress
        target_modifier := GetModifierString(modifier_char)

        if (target_modifier != "invalid") {
            existing_modifiers := ""
            if GetKeyState("Ctrl", "P")
                existing_modifiers .= "{Ctrl down}"
            if GetKeyState("Shift", "P")
                existing_modifiers .= "{Shift down}"
            if GetKeyState("Alt", "P")
                existing_modifiers .= "{Alt down}"

            SendEvent("{Blind}{Ctrl up}{Shift up}{Alt up}")
            Sleep(1)

            if target_modifier {
                SendEvent("{" target_modifier " down}")
                Sleep(1)
            }
            SendEvent(key_char)
            if target_modifier {
                Sleep(1)
                SendEvent("{" target_modifier " up}")
                Sleep(1)
            }

            SendEvent(existing_modifiers)
            Sleep(1)
        }
    }
}

; Helper function to check if a button was just pressed (rising edge)
; Returns true if button is currently pressed but wasn't pressed last poll
CheckButtonPressed(currentButtons, lastButtons, buttonMask) {
    return (currentButtons & buttonMask) && !(lastButtons & buttonMask)
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
    global lastControllerButtons, controllerInitialized, stopScript

    if (stopScript || !controllerInitialized)
        return

    ; Get controller state (controller 0)
    state := XInput_GetState(0)

    ; Controller disconnected
    if (!state) {
        lastControllerButtons := 0
        return
    }

    currentButtons := state.wButtons

    ; Check if any face button was pressed (A, B, X, or Y)
    if (CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_A)
        || CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_B)
        || CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_X)
        || CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_Y)) {

        ; Any of A/B/X/Y pressed - all trigger same action
        ; executeRaw(0, 1)  ; Same as Browser_Favorites/F13 - COMMENTED for testing
        ; TODO: Uncomment when ready to integrate with rotation engine
    }

    ; Check if L1 (Left Shoulder) was pressed
    if (CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_LEFT_SHOULDER)) {
        ; L1 pressed - trigger specific action
        ; executeRaw(2, 3)  ; Same as Browser_Refresh/F15 - COMMENTED for testing
        ; TODO: Uncomment and customize action when ready
    }

    ; Check if R1 (Right Shoulder) was pressed
    if (CheckButtonPressed(currentButtons, lastControllerButtons, XINPUT_GAMEPAD_RIGHT_SHOULDER)) {
        ; R1 pressed - trigger different action
        ; executeRaw(4, 5)  ; Same as Browser_Back/F14 - COMMENTED for testing
        ; TODO: Uncomment and customize action when ready
    }

    ; Save current state for next poll (edge detection)
    lastControllerButtons := currentButtons
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
