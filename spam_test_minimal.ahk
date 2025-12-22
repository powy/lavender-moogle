; Requires AutoHotKey 2
#Requires AutoHotkey v2.0

; CONFIGURATION DIRECTIVES - Critical for reliability
#MaxThreadsPerHotkey 2  ; Allow hotkey to interrupt itself
SendMode "Event"        ; Explicit send mode
SetKeyDelay -1          ; Remove artificial delays

; MINIMAL spam test - Just the essentials
; Hold X to spam F13 key

global INTERVAL := 100  ; milliseconds (100ms = 10 times/second)

; Hold X to spam F13 key
; This is a BLOCKING approach - simpler and more reliable
X:: {
    ; Send immediately - fixes initial delay issue
    SendEvent("{F13}")

    ; Continue spamming while X is physically held down
    While GetKeyState("X", "P") {
        Sleep(INTERVAL)
        SendEvent("{F13}")
    }
    ; Automatically stops when X is released - no separate Up hotkey needed
}

; Press Insert to exit
Insert:: {
    MsgBox("Script stopped!")
    ExitApp
}

; Startup message
ToolTip("Hold X to spam F13 | Insert to exit")
SetTimer(() => ToolTip(), -3000)