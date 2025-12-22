; Requires AutoHotkey v2
; Controller Test Script using XInput
; Tests Xbox controller detection and button presses in background mode

#Requires AutoHotkey v2.0
#Include XInput.ahk

; Initialize XInput library
try {
    XInput_Init()
} catch Error as err {
    MsgBox "Failed to initialize XInput: " err.Message "`n`nMake sure XInput.ahk is in the same directory."
    ExitApp
}

; Global variables for controller state
global controllerConnected := false
global lastButtons := 0
global lastLT := 0
global lastRT := 0

; Start polling controller every 100ms
SetTimer(PollController, 100)

; Exit script with Insert key
Insert:: {
    XInput_Term()
    ToolTip
    MsgBox "Controller test stopped."
    ExitApp
}

; Main polling function
PollController() {
    global controllerConnected, lastButtons, lastLT, lastRT

    ; Get state of controller 0 (first controller)
    state := XInput_GetState(0)

    ; Check if controller is connected
    if (!state) {
        if (controllerConnected) {
            controllerConnected := false
            ToolTip "Controller disconnected! Connect Xbox controller and it will be detected automatically."
        } else {
            ToolTip "Waiting for controller... Press Insert to exit."
        }
        return
    }

    ; Controller is connected
    if (!controllerConnected) {
        controllerConnected := true
    }

    ; Build display string with current controller state
    display := "=== Controller Test (Press Insert to exit) ===`n`n"

    ; Check buttons using bitmask
    buttons := state.wButtons

    ; D-Pad
    display .= "D-Pad: "
    if (buttons & XINPUT_GAMEPAD_DPAD_UP)
        display .= "UP "
    if (buttons & XINPUT_GAMEPAD_DPAD_DOWN)
        display .= "DOWN "
    if (buttons & XINPUT_GAMEPAD_DPAD_LEFT)
        display .= "LEFT "
    if (buttons & XINPUT_GAMEPAD_DPAD_RIGHT)
        display .= "RIGHT "
    if (!(buttons & 0x000F))
        display .= "(none)"
    display .= "`n"

    ; Face Buttons
    display .= "Buttons: "
    if (buttons & XINPUT_GAMEPAD_A)
        display .= "A "
    if (buttons & XINPUT_GAMEPAD_B)
        display .= "B "
    if (buttons & XINPUT_GAMEPAD_X)
        display .= "X "
    if (buttons & XINPUT_GAMEPAD_Y)
        display .= "Y "
    if (!(buttons & 0xF000))
        display .= "(none)"
    display .= "`n"

    ; Shoulder Buttons
    display .= "Shoulders: "
    if (buttons & XINPUT_GAMEPAD_LEFT_SHOULDER)
        display .= "LB "
    if (buttons & XINPUT_GAMEPAD_RIGHT_SHOULDER)
        display .= "RB "
    if (!(buttons & 0x0300))
        display .= "(none)"
    display .= "`n"

    ; Thumbstick Clicks
    display .= "Thumbstick Click: "
    if (buttons & XINPUT_GAMEPAD_LEFT_THUMB)
        display .= "L3 "
    if (buttons & XINPUT_GAMEPAD_RIGHT_THUMB)
        display .= "R3 "
    if (!(buttons & 0x00C0))
        display .= "(none)"
    display .= "`n"

    ; Start/Back
    display .= "Menu: "
    if (buttons & XINPUT_GAMEPAD_START)
        display .= "START "
    if (buttons & XINPUT_GAMEPAD_BACK)
        display .= "BACK "
    if (buttons & XINPUT_GAMEPAD_GUIDE)
        display .= "GUIDE "
    if (!(buttons & 0x0430))
        display .= "(none)"
    display .= "`n`n"

    ; Triggers
    lt := state.bLeftTrigger
    rt := state.bRightTrigger
    display .= "Left Trigger: " FormatNumber(lt, 3) " (" FormatBar(lt, 255, 10) ")`n"
    display .= "Right Trigger: " FormatNumber(rt, 3) " (" FormatBar(rt, 255, 10) ")`n`n"

    ; Left Stick
    lx := state.sThumbLX
    ly := state.sThumbLY
    display .= "Left Stick X: " FormatNumber(lx, 6) " (" FormatBar(lx, 32767, 10) ")`n"
    display .= "Left Stick Y: " FormatNumber(ly, 6) " (" FormatBar(ly, 32767, 10) ")`n`n"

    ; Right Stick
    rx := state.sThumbRX
    ry := state.sThumbRY
    display .= "Right Stick X: " FormatNumber(rx, 6) " (" FormatBar(rx, 32767, 10) ")`n"
    display .= "Right Stick Y: " FormatNumber(ry, 6) " (" FormatBar(ry, 32767, 10) ")`n`n"

    display .= "Background Mode: " (WinActive("A") = WinExist("ahk_class AutoHotkeyGUI") ? "AHK Window Active" : "WORKING - Other window is active!")

    ; Update tooltip
    ToolTip display

    ; Detect button press/release and vibrate on A button press
    if ((buttons & XINPUT_GAMEPAD_A) && !(lastButtons & XINPUT_GAMEPAD_A)) {
        ; A button was just pressed - trigger a short vibration
        XInput_SetState(0, 32000, 0)  ; Left motor vibration
        SetTimer(() => XInput_SetState(0, 0, 0), -200)  ; Stop after 200ms
    }

    lastButtons := buttons
    lastLT := lt
    lastRT := rt
}

; Helper function to format numbers with padding
FormatNumber(num, width) {
    str := String(num)
    while (StrLen(str) < width)
        str := " " str
    return str
}

; Helper function to create a visual bar (like [===---])
FormatBar(value, maxValue, barWidth) {
    ; Handle negative values (center at 0)
    if (maxValue = 32767) {
        ; Analog stick mode (-32768 to 32767)
        normalizedValue := (value + 32768) / 65535.0
    } else {
        ; Trigger mode (0 to 255)
        normalizedValue := value / maxValue
    }

    filledWidth := Round(normalizedValue * barWidth)
    bar := "["

    Loop barWidth {
        if (A_Index <= filledWidth)
            bar .= "="
        else
            bar .= "-"
    }
    bar .= "]"

    return bar
}
