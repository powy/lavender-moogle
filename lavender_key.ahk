#Persistent
#NoEnv

; Create a shared memory section
global hMapFile := DllCall("CreateFileMapping", "Int", 0, "Int", 0, "UInt", 0x04, "UInt", 0, "UInt", 8, "Str", "bluemoogle", "Ptr")
global pBuf := DllCall("MapViewOfFile", "Ptr", hMapFile, "UInt", 0x0004, "UInt", 0, "UInt", 0, "Ptr")

; Hotkey to stop the script
Insert::
ExitApp

; Hotkey to read from shared memory and simulate key press
Del::
If (pBuf)
{
    charArray := ""
    Loop, 8
    {
        char := Chr(NumGet(pBuf + A_Index - 1, "UChar"))
        charArray .= char
    }

    firstChar := SubStr(charArray, 1, 1)
    if firstChar
    {
        ; Simulate the pressing of the first character
        Send, %firstChar%
    }
    else
    {
        MsgBox, Error: First character is empty.
    }
}
else
{
    MsgBox, Error: Could not access shared memory.
}
return
