# Amiga System Symbols (LVO Tables)

## Overview

Amiga OS libraries are accessed through base pointers and negative offsets. When a program calls a library function, it uses a pattern like:

```asm
move.l  4.w,a6          ; Load ExecBase
jsr     -552(a6)        ; Call OpenLibrary
```

The negative offset `-552` corresponds to `_LVOOpenLibrary`. These Library Vector Offsets (LVOs) are standardized and can be resolved to human-readable names during disassembly.

## Common Pattern

```asm
; Open a library
move.l  4.w,a6              ; a6 = ExecBase (always at address 4)
lea     dosName(pc),a1      ; a1 = "dos.library"
moveq   #0,d0               ; d0 = any version
jsr     _LVOOpenLibrary(a6) ; -552(a6)
move.l  d0,_DOSBase         ; save library base

; Use the library
move.l  _DOSBase,a6
jsr     _LVOOpen(a6)        ; -30(a6) in dos.library
```

## Library Base Tracking Heuristics

For disassembly annotation:
- `a6` is typically `SysBase` (ExecBase) at program start
- `move.l $4.w,a6` loads ExecBase
- After `jsr _LVOOpenLibrary(a6)` + `move.l d0,aX`, register `aX` holds the new library base
- Track register assignments to resolve subsequent library calls

---

## exec.library (SysBase - always at address $4)

| Offset | Decimal | LVO Name | Description |
|--------|---------|----------|-------------|
| -0x1E | -30 | _LVOSupervisor | Run code in supervisor mode |
| -0x24 | -36 | _LVOExitIntr | Exit from interrupt |
| -0x2A | -42 | _LVOSchedule | Schedule next task |
| -0x30 | -48 | _LVOReschedule | Reschedule processor |
| -0x36 | -54 | _LVOSwitch | Switch to waiting task |
| -0x3C | -60 | _LVODispatch | Dispatch a task |
| -0x42 | -66 | _LVOException | Process task exception |
| -0x48 | -72 | _LVOInitCode | Initialize resident code |
| -0x4E | -78 | _LVOInitStruct | Initialize memory from table |
| -0x54 | -84 | _LVOMakeLibrary | Create a library |
| -0x5A | -90 | _LVOMakeFunctions | Create jump table |
| -0x60 | -96 | _LVOFindResident | Find a resident module |
| -0x66 | -102 | _LVOInitResident | Initialize a resident module |
| -0x6C | -108 | _LVOAlert | Alert (system failure) |
| -0x72 | -114 | _LVODebug | Enter debugger |
| -0x78 | -120 | _LVODisable | Disable interrupts |
| -0x7E | -126 | _LVOEnable | Enable interrupts |
| -0x84 | -132 | _LVOForbid | Forbid task switching |
| -0x8A | -138 | _LVOPermit | Permit task switching |
| -0x90 | -144 | _LVOSetSR | Set status register |
| -0x96 | -150 | _LVOSuperState | Enter supervisor state |
| -0x9C | -156 | _LVOUserState | Return to user state |
| -0xA2 | -162 | _LVOSetIntVector | Set interrupt vector |
| -0xA8 | -168 | _LVOAddIntServer | Add interrupt server |
| -0xAE | -174 | _LVORemIntServer | Remove interrupt server |
| -0xB4 | -180 | _LVOCause | Cause a software interrupt |
| -0xBA | -186 | _LVOAllocate | Allocate from memory region |
| -0xC0 | -192 | _LVODeallocate | Deallocate to memory region |
| -0xC6 | -198 | _LVOAllocMem | Allocate memory |
| -0xCC | -204 | _LVOAllocAbs | Allocate absolute memory |
| -0xD2 | -210 | _LVOFreeMem | Free memory |
| -0xD8 | -216 | _LVOAvailMem | Query available memory |
| -0xDE | -222 | _LVOAllocEntry | Allocate memory entries |
| -0xE4 | -228 | _LVOFreeEntry | Free memory entries |
| -0xEA | -234 | _LVOInsert | Insert node into list |
| -0xF0 | -240 | _LVOAddHead | Add node to head of list |
| -0xF6 | -246 | _LVOAddTail | Add node to tail of list |
| -0xFC | -252 | _LVORemove | Remove node from list |
| -0x102 | -258 | _LVORemHead | Remove head of list |
| -0x108 | -264 | _LVORemTail | Remove tail of list |
| -0x10E | -270 | _LVOEnqueue | Insert node by priority |
| -0x114 | -276 | _LVOFindName | Find named node in list |
| -0x11A | -282 | _LVOAddTask | Add task to system |
| -0x120 | -288 | _LVORemTask | Remove task |
| -0x126 | -294 | _LVOFindTask | Find task by name |
| -0x12C | -300 | _LVOSetTaskPri | Set task priority |
| -0x132 | -306 | _LVOSetSignal | Set signal bits |
| -0x138 | -312 | _LVOSetExcept | Set exception bits |
| -0x13E | -318 | _LVOFindPort | Find message port |
| -0x144 | -324 | _LVOSignal | Signal a task |
| -0x14A | -330 | _LVOWait | Wait for signal |
| -0x150 | -336 | _LVOSubTime | Subtract time values |
| -0x156 | -342 | _LVOAddTime | Add time values |
| -0x15C | -348 | _LVOCmpTime | Compare time values |
| -0x162 | -354 | _LVOAddResource | Add resource to system |
| -0x168 | -360 | _LVORemResource | Remove resource |
| -0x16E | -366 | _LVOOpenResource | Open a resource |
| -0x174 | -372 | _LVOAddPort | Add message port |
| -0x17A | -378 | _LVORemPort | Remove message port |
| -0x180 | -384 | _LVOWaitPort | Wait for message at port |
| -0x186 | -390 | _LVOFindPort | Find message port by name |
| -0x18C | -396 | _LVOGetMsg | Get message from port |
| -0x192 | -402 | _LVOPutMsg | Send message to port |
| -0x198 | -408 | _LVOReplyMsg | Reply to a message |
| -0x19E | -414 | _LVOWaitIO | Wait for I/O completion |
| -0x1A4 | -420 | _LVODoIO | Perform I/O synchronously |
| -0x1AA | -426 | _LVOSendIO | Send I/O request async |
| -0x1B0 | -432 | _LVOCheckIO | Check I/O status |
| -0x1B6 | -438 | _LVOAbortIO | Abort I/O request |
| -0x1BC | -444 | _LVOAddDevice | Add device to system |
| -0x1C2 | -450 | _LVORemDevice | Remove device |
| -0x1C8 | -456 | _LVOOpenDevice | Open a device |
| -0x1CE | -462 | _LVOCloseDevice | Close a device |
| -0x1D4 | -468 | _LVOFindConfigDev | Find configured device |
| -0x1DA | -474 | _LVOSetFunction | Replace library function |
| -0x1E0 | -480 | _LVOSumLibrary | Checksum a library |
| -0x1E6 | -486 | _LVOAddLibrary | Add library to system |
| -0x1EC | -492 | _LVORemLibrary | Remove library |
| -0x1F2 | -498 | _LVOOldOpenLibrary | Open library (old, no version check) |
| -0x1F8 | -504 | _LVOCloseLibrary | Close library |
| -0x1FE | -510 | _LVOExpungeLibrary | Remove and free library |
| -0x204 | -516 | _LVOGetCurrentBinding | Get current binding |
| -0x20A | -522 | _LVOSetCurrentBinding | Set current binding |
| -0x210 | -528 | _LVORawDoFmt | Format string (printf-like) |
| -0x216 | -534 | _LVOGetCC | Get condition codes |
| -0x21C | -540 | _LVOTypeOfMem | Query memory type |
| -0x222 | -546 | _LVOProcure | Obtain semaphore |
| -0x228 | -552 | _LVOOpenLibrary | Open library with version check |
| -0x22E | -558 | _LVOInitSemaphore | Initialize semaphore |
| -0x234 | -564 | _LVOObtainSemaphore | Obtain semaphore |
| -0x23A | -570 | _LVOReleaseSemaphore | Release semaphore |
| -0x240 | -576 | _LVOAttemptSemaphore | Try to obtain semaphore |
| -0x246 | -582 | _LVOObtainSemaphoreList | Obtain semaphore list |
| -0x24C | -588 | _LVOReleaseSemaphoreList | Release semaphore list |
| -0x252 | -594 | _LVOFindSemaphore | Find semaphore by name |
| -0x258 | -600 | _LVOAddSemaphore | Add semaphore to system |
| -0x25E | -606 | _LVORemSemaphore | Remove semaphore |
| -0x264 | -612 | _LVOSumKickData | Checksum kick data |
| -0x26A | -618 | _LVOAddMemList | Add memory to free list |
| -0x270 | -624 | _LVOCopyMem | Copy memory |
| -0x276 | -630 | _LVOCopyMemQuick | Copy aligned memory (fast) |
| -0x27C | -636 | _LVOCacheClearU | Clear all caches |
| -0x282 | -642 | _LVOCacheClearE | Clear specific cache lines |
| -0x288 | -648 | _LVOCacheControl | Control cache behavior |
| -0x28E | -654 | _LVOCreateIORequest | Create I/O request |
| -0x294 | -660 | _LVODeleteIORequest | Delete I/O request |
| -0x29A | -666 | _LVOCreateMsgPort | Create message port |
| -0x2A0 | -672 | _LVODeleteMsgPort | Delete message port |
| -0x2A6 | -678 | _LVOObtainSemaphoreShared | Obtain shared semaphore |
| -0x2AC | -684 | _LVOAllocVec | Allocate tracked memory |
| -0x2B2 | -690 | _LVOFreeVec | Free tracked memory |
| -0x2B8 | -696 | _LVOCreatePool | Create memory pool |
| -0x2BE | -702 | _LVODeletePool | Delete memory pool |
| -0x2C4 | -708 | _LVOAllocPooled | Allocate from pool |
| -0x2CA | -714 | _LVOFreePooled | Free to pool |

---

## dos.library

| Offset | Decimal | LVO Name | Description |
|--------|---------|----------|-------------|
| -0x1E | -30 | _LVOOpen | Open a file |
| -0x24 | -36 | _LVOClose | Close a file |
| -0x2A | -42 | _LVORead | Read from file |
| -0x30 | -48 | _LVOWrite | Write to file |
| -0x36 | -54 | _LVOInput | Get input file handle |
| -0x3C | -60 | _LVOOutput | Get output file handle |
| -0x42 | -66 | _LVOSeek | Seek in file |
| -0x48 | -72 | _LVODeleteFile | Delete a file |
| -0x4E | -78 | _LVORename | Rename a file |
| -0x54 | -84 | _LVOLock | Lock a file/directory |
| -0x5A | -90 | _LVOUnLock | Unlock a file/directory |
| -0x60 | -96 | _LVODupLock | Duplicate a lock |
| -0x66 | -102 | _LVOExamine | Examine file info |
| -0x6C | -108 | _LVOExNext | Examine next entry |
| -0x72 | -114 | _LVOInfo | Get filesystem info |
| -0x78 | -120 | _LVOCreateDir | Create directory |
| -0x7E | -126 | _LVOCurrentDir | Set current directory |
| -0x84 | -132 | _LVOIoErr | Get last I/O error |
| -0x8A | -138 | _LVOCreateProc | Create a process |
| -0x90 | -144 | _LVOExit | Exit program |
| -0xA8 | -168 | _LVOLoadSeg | Load an executable |
| -0xAE | -174 | _LVOUnLoadSeg | Unload a segment |
| -0xB4 | -180 | _LVODeviceProc | Get device process |
| -0xBA | -186 | _LVOSetComment | Set file comment |
| -0xC0 | -192 | _LVOSetProtection | Set file protection |
| -0xC6 | -198 | _LVODateStamp | Get current date/time |
| -0xCC | -204 | _LVODelay | Delay for ticks |
| -0xDE | -222 | _LVOExecute | Execute a command |
| -0xE4 | -228 | _LVOAllocDosObject | Allocate DOS object |
| -0xEA | -234 | _LVOFreeDosObject | Free DOS object |
| -0xF0 | -240 | _LVODoPkt | Send DOS packet |
| -0xF6 | -246 | _LVOSendPkt | Send packet async |
| -0xFC | -252 | _LVOWaitPkt | Wait for packet |
| -0x102 | -258 | _LVOReplyPkt | Reply to packet |
| -0x108 | -264 | _LVOAbortPkt | Abort packet |
| -0x10E | -270 | _LVOLockRecord | Lock file record |
| -0x114 | -276 | _LVOLockRecords | Lock multiple records |
| -0x11A | -282 | _LVOUnLockRecord | Unlock file record |
| -0x120 | -288 | _LVOUnLockRecords | Unlock multiple records |
| -0x126 | -294 | _LVOSelectInput | Select input stream |
| -0x12C | -300 | _LVOSelectOutput | Select output stream |
| -0x132 | -306 | _LVOFGetC | Get character from file |
| -0x138 | -312 | _LVOFPutC | Put character to file |
| -0x13E | -318 | _LVOFRead | Read block from file |
| -0x144 | -324 | _LVOFWrite | Write block to file |
| -0x14A | -330 | _LVOFGets | Get string from file |
| -0x150 | -336 | _LVOFPuts | Put string to file |
| -0x156 | -342 | _LVOVFWritef | Formatted write |
| -0x15C | -348 | _LVOVFPrintf | Formatted print |
| -0x168 | -360 | _LVOSetVBuf | Set buffer mode |
| -0x174 | -372 | _LVOParsePatternNoCase | Parse pattern (case insensitive) |
| -0x17A | -378 | _LVOMatchPatternNoCase | Match pattern (case insensitive) |
| -0x192 | -402 | _LVOFilePart | Get filename from path |
| -0x198 | -408 | _LVOPathPart | Get path from full path |
| -0x19E | -414 | _LVOAddPart | Combine path components |
| -0x1A4 | -420 | _LVOReadArgs | Parse command line arguments |
| -0x1AA | -426 | _LVOFreeArgs | Free parsed arguments |
| -0x1B0 | -432 | _LVOPrintFault | Print error message |
| -0x1B6 | -438 | _LVOErrorReport | Report error |
| -0x1BC | -444 | _LVOSystemTagList | Execute command with tags |
| -0x1C2 | -450 | _LVOAssignLock | Create assignment |
| -0x1C8 | -456 | _LVOAssignLate | Create late assignment |

---

## intuition.library

| Offset | Decimal | LVO Name | Description |
|--------|---------|----------|-------------|
| -0x1E | -30 | _LVOOpenIntuition | Open Intuition (private) |
| -0x24 | -36 | _LVOIntuition | Process IntuiMessage (private) |
| -0x2A | -42 | _LVOAddGadget | Add gadget to window |
| -0x30 | -48 | _LVOClearDMRequest | Clear DM requester |
| -0x36 | -54 | _LVOClearMenuStrip | Remove menu from window |
| -0x3C | -60 | _LVOClearPointer | Reset window pointer |
| -0x42 | -66 | _LVOCloseScreen | Close a screen |
| -0x48 | -72 | _LVOCloseWindow | Close a window |
| -0x4E | -78 | _LVOCloseWorkBench | Close Workbench screen |
| -0x54 | -84 | _LVOCurrentTime | Get current time |
| -0x5A | -90 | _LVODisplayAlert | Display alert |
| -0x60 | -96 | _LVODisplayBeep | Flash screen |
| -0x66 | -102 | _LVODoubleClick | Test double-click timing |
| -0x6C | -108 | _LVODrawBorder | Draw border |
| -0x72 | -114 | _LVODrawImage | Draw image |
| -0x78 | -120 | _LVOEndRequest | End requester |
| -0x7E | -126 | _LVOGetDefPrefs | Get default preferences |
| -0x84 | -132 | _LVOGetPrefs | Get preferences |
| -0x8A | -138 | _LVOInitRequester | Initialize requester |
| -0x90 | -144 | _LVOItemAddress | Get menu item address |
| -0x96 | -150 | _LVOModifyIDCMP | Change window IDCMP flags |
| -0x9C | -156 | _LVOModifyProp | Modify proportional gadget |
| -0xA2 | -162 | _LVOMoveScreen | Move screen |
| -0xA8 | -168 | _LVOMoveWindow | Move window |
| -0xAE | -174 | _LVOOffGadget | Disable gadget |
| -0xB4 | -180 | _LVOOffMenu | Disable menu item |
| -0xBA | -186 | _LVOOnGadget | Enable gadget |
| -0xC0 | -192 | _LVOOnMenu | Enable menu item |
| -0xC6 | -198 | _LVOOpenScreen | Open a screen |
| -0xCC | -204 | _LVOOpenWindow | Open a window |
| -0xD2 | -210 | _LVOOpenWorkBench | Open Workbench screen |
| -0xDE | -222 | _LVORefreshGadgets | Refresh gadget display |
| -0xE4 | -228 | _LVORemoveGadget | Remove gadget |
| -0xFC | -252 | _LVOReportMouse | Enable mouse reporting |
| -0x102 | -258 | _LVORequest | Display requester |
| -0x108 | -264 | _LVOScreenToBack | Send screen to back |
| -0x10E | -270 | _LVOScreenToFront | Bring screen to front |
| -0x114 | -276 | _LVOSetDMRequest | Set DM requester |
| -0x11A | -282 | _LVOSetMenuStrip | Attach menu to window |
| -0x120 | -288 | _LVOSetPointer | Set window pointer |
| -0x126 | -294 | _LVOSetWindowTitles | Set window titles |
| -0x12C | -300 | _LVOShowTitle | Show/hide screen title |
| -0x132 | -306 | _LVOSizeWindow | Resize window |
| -0x138 | -312 | _LVOViewAddress | Get View address |
| -0x13E | -318 | _LVOViewPortAddress | Get ViewPort address |
| -0x144 | -324 | _LVOWindowToBack | Send window to back |
| -0x14A | -330 | _LVOWindowToFront | Bring window to front |
| -0x150 | -336 | _LVOWindowLimits | Set window size limits |
| -0x186 | -390 | _LVOAutoRequest | Auto requester |
| -0x18C | -396 | _LVOBeginRefresh | Begin optimized refresh |
| -0x192 | -402 | _LVOBuildSysRequest | Build system requester |
| -0x198 | -408 | _LVOEndRefresh | End optimized refresh |
| -0x19E | -414 | _LVOFreeSysRequest | Free system requester |
| -0x1AA | -426 | _LVOMakeScreen | Remake screen display |
| -0x1B0 | -432 | _LVORemakeDisplay | Remake entire display |
| -0x1B6 | -438 | _LVORethinkDisplay | Rethink display layout |
| -0x264 | -612 | _LVOEasyRequestArgs | Easy requester |
| -0x26A | -618 | _LVOBuildEasyRequestArgs | Build easy requester |

---

## graphics.library

| Offset | Decimal | LVO Name | Description |
|--------|---------|----------|-------------|
| -0x1E | -30 | _LVOBltBitMap | Blit bitmap |
| -0x24 | -36 | _LVOBltTemplate | Blit template |
| -0x2A | -42 | _LVOClearEOL | Clear to end of line |
| -0x30 | -48 | _LVOClearScreen | Clear screen |
| -0x36 | -54 | _LVOTextLength | Get text width |
| -0x3C | -60 | _LVOText | Render text |
| -0x42 | -66 | _LVOSetFont | Set rastport font |
| -0x48 | -72 | _LVOOpenFont | Open font |
| -0x4E | -78 | _LVOCloseFont | Close font |
| -0x54 | -84 | _LVOAskSoftStyle | Query soft style |
| -0x5A | -90 | _LVOSetSoftStyle | Set soft style |
| -0xC6 | -198 | _LVOMove | Move draw position |
| -0xCC | -204 | _LVODraw | Draw line |
| -0xD2 | -210 | _LVOAreaMove | Area move |
| -0xD8 | -216 | _LVOAreaDraw | Area draw |
| -0xDE | -222 | _LVOAreaEnd | End area fill |
| -0xEA | -234 | _LVOInitRastPort | Initialize RastPort |
| -0xF0 | -240 | _LVOInitVPort | Initialize ViewPort |
| -0xFC | -252 | _LVOSetRGB4 | Set color (4-bit) |
| -0x102 | -258 | _LVOQBSBlit | Queue blitter operation |
| -0x108 | -264 | _LVOBltClear | Clear with blitter |
| -0x10E | -270 | _LVORectFill | Fill rectangle |
| -0x114 | -276 | _LVOBltPattern | Blit pattern |
| -0x11A | -282 | _LVOReadPixel | Read pixel |
| -0x120 | -288 | _LVOWritePixel | Write pixel |
| -0x12C | -300 | _LVOFlood | Flood fill |
| -0x132 | -306 | _LVOPolyDraw | Draw polygon |
| -0x138 | -312 | _LVOSetAPen | Set area pen (foreground) |
| -0x13E | -318 | _LVOSetBPen | Set background pen |
| -0x144 | -324 | _LVOSetDrMd | Set draw mode |
| -0x150 | -336 | _LVOInitView | Initialize View |
| -0x168 | -360 | _LVOLoadView | Load View into display |
| -0x16E | -366 | _LVOWaitBlit | Wait for blitter |
| -0x174 | -372 | _LVOSetRast | Fill entire rastport |
| -0x17A | -378 | _LVOOwnBlitter | Own blitter exclusively |
| -0x180 | -384 | _LVODisownBlitter | Release blitter |
| -0x18C | -396 | _LVOInitBitMap | Initialize BitMap |
| -0x192 | -402 | _LVOScrollRaster | Scroll raster area |
| -0x198 | -408 | _LVOWaitBOVP | Wait for ViewPort BOV |
| -0x19E | -414 | _LVOAllocRaster | Allocate raster memory |
| -0x1A4 | -420 | _LVOFreeRaster | Free raster memory |
| -0x1B0 | -432 | _LVOMakeVPort | Construct ViewPort |
| -0x1B6 | -438 | _LVOMrgCop | Merge copper lists |
| -0x1BC | -444 | _LVOLoadRGB4 | Load color table (4-bit) |
| -0x1F8 | -504 | _LVOBltBitMapRastPort | Blit bitmap to rastport |
| -0x264 | -612 | _LVOAllocBitMap | Allocate BitMap |
| -0x26A | -618 | _LVOFreeBitMap | Free BitMap |

---

## Notes for Implementation

- LVO offsets are always **negative** and multiples of 6 (each jump table entry is 6 bytes: JMP instruction)
- The tables above cover the most commonly used functions. Full tables can be generated from the Amiga NDK `.fd` (function definition) files
- Libraries not listed here but commonly encountered: `diskfont.library`, `layers.library`, `mathffp.library`, `commodities.library`, `gadtools.library`, `asl.library`, `iffparse.library`, `utility.library`, `workbench.library`, `icon.library`, `expansion.library`
- The initial implementation should include exec, dos, intuition, and graphics. Others can be added incrementally.

---

## References

- Amiga NDK (Native Development Kit) `.fd` files
- [resrc4 LVO tables](https://github.com/rolsen74/resrc4)
- [Amiga Developer Docs](http://amigadev.elowar.com/)
