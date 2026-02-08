//! Static Amiga OS Library Vector Offset (LVO) tables.
//!
//! Each Amiga library is accessed through a base pointer with negative
//! offsets. These tables map the well-known offsets to human-readable
//! function names for the four core libraries: exec, dos, intuition,
//! and graphics.

/// A single library vector offset entry.
#[derive(Debug, Clone, Copy)]
pub struct LvoEntry {
    pub offset: i16,
    pub name: &'static str,
}

/// A named library with its LVO table.
#[derive(Debug, Clone, Copy)]
pub struct Library {
    pub name: &'static str,
    pub entries: &'static [LvoEntry],
}

// Tables are sorted by offset (descending / most negative first) so we
// can binary-search by offset.

static EXEC_ENTRIES: &[LvoEntry] = &[
    LvoEntry { offset: -714, name: "_LVOFreePooled" },
    LvoEntry { offset: -708, name: "_LVOAllocPooled" },
    LvoEntry { offset: -702, name: "_LVODeletePool" },
    LvoEntry { offset: -696, name: "_LVOCreatePool" },
    LvoEntry { offset: -690, name: "_LVOFreeVec" },
    LvoEntry { offset: -684, name: "_LVOAllocVec" },
    LvoEntry { offset: -678, name: "_LVOObtainSemaphoreShared" },
    LvoEntry { offset: -672, name: "_LVODeleteMsgPort" },
    LvoEntry { offset: -666, name: "_LVOCreateMsgPort" },
    LvoEntry { offset: -660, name: "_LVODeleteIORequest" },
    LvoEntry { offset: -654, name: "_LVOCreateIORequest" },
    LvoEntry { offset: -648, name: "_LVOCacheControl" },
    LvoEntry { offset: -642, name: "_LVOCacheClearE" },
    LvoEntry { offset: -636, name: "_LVOCacheClearU" },
    LvoEntry { offset: -630, name: "_LVOCopyMemQuick" },
    LvoEntry { offset: -624, name: "_LVOCopyMem" },
    LvoEntry { offset: -618, name: "_LVOAddMemList" },
    LvoEntry { offset: -612, name: "_LVOSumKickData" },
    LvoEntry { offset: -606, name: "_LVORemSemaphore" },
    LvoEntry { offset: -600, name: "_LVOAddSemaphore" },
    LvoEntry { offset: -594, name: "_LVOFindSemaphore" },
    LvoEntry { offset: -588, name: "_LVOReleaseSemaphoreList" },
    LvoEntry { offset: -582, name: "_LVOObtainSemaphoreList" },
    LvoEntry { offset: -576, name: "_LVOAttemptSemaphore" },
    LvoEntry { offset: -570, name: "_LVOReleaseSemaphore" },
    LvoEntry { offset: -564, name: "_LVOObtainSemaphore" },
    LvoEntry { offset: -558, name: "_LVOInitSemaphore" },
    LvoEntry { offset: -552, name: "_LVOOpenLibrary" },
    LvoEntry { offset: -546, name: "_LVOProcure" },
    LvoEntry { offset: -540, name: "_LVOTypeOfMem" },
    LvoEntry { offset: -534, name: "_LVOGetCC" },
    LvoEntry { offset: -528, name: "_LVORawDoFmt" },
    LvoEntry { offset: -522, name: "_LVOSetCurrentBinding" },
    LvoEntry { offset: -516, name: "_LVOGetCurrentBinding" },
    LvoEntry { offset: -510, name: "_LVOExpungeLibrary" },
    LvoEntry { offset: -504, name: "_LVOCloseLibrary" },
    LvoEntry { offset: -498, name: "_LVOOldOpenLibrary" },
    LvoEntry { offset: -492, name: "_LVORemLibrary" },
    LvoEntry { offset: -486, name: "_LVOAddLibrary" },
    LvoEntry { offset: -480, name: "_LVOSumLibrary" },
    LvoEntry { offset: -474, name: "_LVOSetFunction" },
    LvoEntry { offset: -468, name: "_LVOFindConfigDev" },
    LvoEntry { offset: -462, name: "_LVOCloseDevice" },
    LvoEntry { offset: -456, name: "_LVOOpenDevice" },
    LvoEntry { offset: -450, name: "_LVORemDevice" },
    LvoEntry { offset: -444, name: "_LVOAddDevice" },
    LvoEntry { offset: -438, name: "_LVOAbortIO" },
    LvoEntry { offset: -432, name: "_LVOCheckIO" },
    LvoEntry { offset: -426, name: "_LVOSendIO" },
    LvoEntry { offset: -420, name: "_LVODoIO" },
    LvoEntry { offset: -414, name: "_LVOWaitIO" },
    LvoEntry { offset: -408, name: "_LVOReplyMsg" },
    LvoEntry { offset: -402, name: "_LVOPutMsg" },
    LvoEntry { offset: -396, name: "_LVOGetMsg" },
    LvoEntry { offset: -390, name: "_LVOFindPort" },
    LvoEntry { offset: -384, name: "_LVOWaitPort" },
    LvoEntry { offset: -378, name: "_LVORemPort" },
    LvoEntry { offset: -372, name: "_LVOAddPort" },
    LvoEntry { offset: -366, name: "_LVOOpenResource" },
    LvoEntry { offset: -360, name: "_LVORemResource" },
    LvoEntry { offset: -354, name: "_LVOAddResource" },
    LvoEntry { offset: -348, name: "_LVOCmpTime" },
    LvoEntry { offset: -342, name: "_LVOAddTime" },
    LvoEntry { offset: -336, name: "_LVOSubTime" },
    LvoEntry { offset: -330, name: "_LVOWait" },
    LvoEntry { offset: -324, name: "_LVOSignal" },
    LvoEntry { offset: -318, name: "_LVOFindPort" },
    LvoEntry { offset: -312, name: "_LVOSetExcept" },
    LvoEntry { offset: -306, name: "_LVOSetSignal" },
    LvoEntry { offset: -300, name: "_LVOSetTaskPri" },
    LvoEntry { offset: -294, name: "_LVOFindTask" },
    LvoEntry { offset: -288, name: "_LVORemTask" },
    LvoEntry { offset: -282, name: "_LVOAddTask" },
    LvoEntry { offset: -276, name: "_LVOFindName" },
    LvoEntry { offset: -270, name: "_LVOEnqueue" },
    LvoEntry { offset: -264, name: "_LVORemTail" },
    LvoEntry { offset: -258, name: "_LVORemHead" },
    LvoEntry { offset: -252, name: "_LVORemove" },
    LvoEntry { offset: -246, name: "_LVOAddTail" },
    LvoEntry { offset: -240, name: "_LVOAddHead" },
    LvoEntry { offset: -234, name: "_LVOInsert" },
    LvoEntry { offset: -228, name: "_LVOFreeEntry" },
    LvoEntry { offset: -222, name: "_LVOAllocEntry" },
    LvoEntry { offset: -216, name: "_LVOAvailMem" },
    LvoEntry { offset: -210, name: "_LVOFreeMem" },
    LvoEntry { offset: -204, name: "_LVOAllocAbs" },
    LvoEntry { offset: -198, name: "_LVOAllocMem" },
    LvoEntry { offset: -192, name: "_LVODeallocate" },
    LvoEntry { offset: -186, name: "_LVOAllocate" },
    LvoEntry { offset: -180, name: "_LVOCause" },
    LvoEntry { offset: -174, name: "_LVORemIntServer" },
    LvoEntry { offset: -168, name: "_LVOAddIntServer" },
    LvoEntry { offset: -162, name: "_LVOSetIntVector" },
    LvoEntry { offset: -156, name: "_LVOUserState" },
    LvoEntry { offset: -150, name: "_LVOSuperState" },
    LvoEntry { offset: -144, name: "_LVOSetSR" },
    LvoEntry { offset: -138, name: "_LVOPermit" },
    LvoEntry { offset: -132, name: "_LVOForbid" },
    LvoEntry { offset: -126, name: "_LVOEnable" },
    LvoEntry { offset: -120, name: "_LVODisable" },
    LvoEntry { offset: -114, name: "_LVODebug" },
    LvoEntry { offset: -108, name: "_LVOAlert" },
    LvoEntry { offset: -102, name: "_LVOInitResident" },
    LvoEntry { offset: -96, name: "_LVOFindResident" },
    LvoEntry { offset: -90, name: "_LVOMakeFunctions" },
    LvoEntry { offset: -84, name: "_LVOMakeLibrary" },
    LvoEntry { offset: -78, name: "_LVOInitStruct" },
    LvoEntry { offset: -72, name: "_LVOInitCode" },
    LvoEntry { offset: -66, name: "_LVOException" },
    LvoEntry { offset: -60, name: "_LVODispatch" },
    LvoEntry { offset: -54, name: "_LVOSwitch" },
    LvoEntry { offset: -48, name: "_LVOReschedule" },
    LvoEntry { offset: -42, name: "_LVOSchedule" },
    LvoEntry { offset: -36, name: "_LVOExitIntr" },
    LvoEntry { offset: -30, name: "_LVOSupervisor" },
];

static DOS_ENTRIES: &[LvoEntry] = &[
    LvoEntry { offset: -456, name: "_LVOAssignLate" },
    LvoEntry { offset: -450, name: "_LVOAssignLock" },
    LvoEntry { offset: -444, name: "_LVOSystemTagList" },
    LvoEntry { offset: -438, name: "_LVOErrorReport" },
    LvoEntry { offset: -432, name: "_LVOPrintFault" },
    LvoEntry { offset: -426, name: "_LVOFreeArgs" },
    LvoEntry { offset: -420, name: "_LVOReadArgs" },
    LvoEntry { offset: -414, name: "_LVOAddPart" },
    LvoEntry { offset: -408, name: "_LVOPathPart" },
    LvoEntry { offset: -402, name: "_LVOFilePart" },
    LvoEntry { offset: -378, name: "_LVOMatchPatternNoCase" },
    LvoEntry { offset: -372, name: "_LVOParsePatternNoCase" },
    LvoEntry { offset: -360, name: "_LVOSetVBuf" },
    LvoEntry { offset: -348, name: "_LVOVFPrintf" },
    LvoEntry { offset: -342, name: "_LVOVFWritef" },
    LvoEntry { offset: -336, name: "_LVOFPuts" },
    LvoEntry { offset: -330, name: "_LVOFGets" },
    LvoEntry { offset: -324, name: "_LVOFWrite" },
    LvoEntry { offset: -318, name: "_LVOFRead" },
    LvoEntry { offset: -312, name: "_LVOFPutC" },
    LvoEntry { offset: -306, name: "_LVOFGetC" },
    LvoEntry { offset: -300, name: "_LVOSelectOutput" },
    LvoEntry { offset: -294, name: "_LVOSelectInput" },
    LvoEntry { offset: -288, name: "_LVOUnLockRecords" },
    LvoEntry { offset: -282, name: "_LVOUnLockRecord" },
    LvoEntry { offset: -276, name: "_LVOLockRecords" },
    LvoEntry { offset: -270, name: "_LVOLockRecord" },
    LvoEntry { offset: -264, name: "_LVOAbortPkt" },
    LvoEntry { offset: -258, name: "_LVOReplyPkt" },
    LvoEntry { offset: -252, name: "_LVOWaitPkt" },
    LvoEntry { offset: -246, name: "_LVOSendPkt" },
    LvoEntry { offset: -240, name: "_LVODoPkt" },
    LvoEntry { offset: -234, name: "_LVOFreeDosObject" },
    LvoEntry { offset: -228, name: "_LVOAllocDosObject" },
    LvoEntry { offset: -222, name: "_LVOExecute" },
    LvoEntry { offset: -204, name: "_LVODelay" },
    LvoEntry { offset: -198, name: "_LVODateStamp" },
    LvoEntry { offset: -192, name: "_LVOSetProtection" },
    LvoEntry { offset: -186, name: "_LVOSetComment" },
    LvoEntry { offset: -180, name: "_LVODeviceProc" },
    LvoEntry { offset: -174, name: "_LVOUnLoadSeg" },
    LvoEntry { offset: -168, name: "_LVOLoadSeg" },
    LvoEntry { offset: -144, name: "_LVOExit" },
    LvoEntry { offset: -138, name: "_LVOCreateProc" },
    LvoEntry { offset: -132, name: "_LVOIoErr" },
    LvoEntry { offset: -126, name: "_LVOCurrentDir" },
    LvoEntry { offset: -120, name: "_LVOCreateDir" },
    LvoEntry { offset: -114, name: "_LVOInfo" },
    LvoEntry { offset: -108, name: "_LVOExNext" },
    LvoEntry { offset: -102, name: "_LVOExamine" },
    LvoEntry { offset: -96, name: "_LVODupLock" },
    LvoEntry { offset: -90, name: "_LVOUnLock" },
    LvoEntry { offset: -84, name: "_LVOLock" },
    LvoEntry { offset: -78, name: "_LVORename" },
    LvoEntry { offset: -72, name: "_LVODeleteFile" },
    LvoEntry { offset: -66, name: "_LVOSeek" },
    LvoEntry { offset: -60, name: "_LVOOutput" },
    LvoEntry { offset: -54, name: "_LVOInput" },
    LvoEntry { offset: -48, name: "_LVOWrite" },
    LvoEntry { offset: -42, name: "_LVORead" },
    LvoEntry { offset: -36, name: "_LVOClose" },
    LvoEntry { offset: -30, name: "_LVOOpen" },
];

static INTUITION_ENTRIES: &[LvoEntry] = &[
    LvoEntry { offset: -618, name: "_LVOBuildEasyRequestArgs" },
    LvoEntry { offset: -612, name: "_LVOEasyRequestArgs" },
    LvoEntry { offset: -438, name: "_LVORethinkDisplay" },
    LvoEntry { offset: -432, name: "_LVORemakeDisplay" },
    LvoEntry { offset: -426, name: "_LVOMakeScreen" },
    LvoEntry { offset: -414, name: "_LVOFreeSysRequest" },
    LvoEntry { offset: -408, name: "_LVOEndRefresh" },
    LvoEntry { offset: -402, name: "_LVOBuildSysRequest" },
    LvoEntry { offset: -396, name: "_LVOBeginRefresh" },
    LvoEntry { offset: -390, name: "_LVOAutoRequest" },
    LvoEntry { offset: -336, name: "_LVOWindowLimits" },
    LvoEntry { offset: -330, name: "_LVOWindowToFront" },
    LvoEntry { offset: -324, name: "_LVOWindowToBack" },
    LvoEntry { offset: -318, name: "_LVOViewPortAddress" },
    LvoEntry { offset: -312, name: "_LVOViewAddress" },
    LvoEntry { offset: -306, name: "_LVOSizeWindow" },
    LvoEntry { offset: -300, name: "_LVOShowTitle" },
    LvoEntry { offset: -294, name: "_LVOSetWindowTitles" },
    LvoEntry { offset: -288, name: "_LVOSetPointer" },
    LvoEntry { offset: -282, name: "_LVOSetMenuStrip" },
    LvoEntry { offset: -276, name: "_LVOSetDMRequest" },
    LvoEntry { offset: -270, name: "_LVOScreenToFront" },
    LvoEntry { offset: -264, name: "_LVOScreenToBack" },
    LvoEntry { offset: -258, name: "_LVORequest" },
    LvoEntry { offset: -252, name: "_LVOReportMouse" },
    LvoEntry { offset: -228, name: "_LVORemoveGadget" },
    LvoEntry { offset: -222, name: "_LVORefreshGadgets" },
    LvoEntry { offset: -210, name: "_LVOOpenWorkBench" },
    LvoEntry { offset: -204, name: "_LVOOpenWindow" },
    LvoEntry { offset: -198, name: "_LVOOpenScreen" },
    LvoEntry { offset: -192, name: "_LVOOnMenu" },
    LvoEntry { offset: -186, name: "_LVOOnGadget" },
    LvoEntry { offset: -180, name: "_LVOOffMenu" },
    LvoEntry { offset: -174, name: "_LVOOffGadget" },
    LvoEntry { offset: -168, name: "_LVOMoveWindow" },
    LvoEntry { offset: -162, name: "_LVOMoveScreen" },
    LvoEntry { offset: -156, name: "_LVOModifyProp" },
    LvoEntry { offset: -150, name: "_LVOModifyIDCMP" },
    LvoEntry { offset: -144, name: "_LVOItemAddress" },
    LvoEntry { offset: -138, name: "_LVOInitRequester" },
    LvoEntry { offset: -132, name: "_LVOGetPrefs" },
    LvoEntry { offset: -126, name: "_LVOGetDefPrefs" },
    LvoEntry { offset: -120, name: "_LVOEndRequest" },
    LvoEntry { offset: -114, name: "_LVODrawImage" },
    LvoEntry { offset: -108, name: "_LVODrawBorder" },
    LvoEntry { offset: -102, name: "_LVODoubleClick" },
    LvoEntry { offset: -96, name: "_LVODisplayBeep" },
    LvoEntry { offset: -90, name: "_LVODisplayAlert" },
    LvoEntry { offset: -84, name: "_LVOCurrentTime" },
    LvoEntry { offset: -78, name: "_LVOCloseWorkBench" },
    LvoEntry { offset: -72, name: "_LVOCloseWindow" },
    LvoEntry { offset: -66, name: "_LVOCloseScreen" },
    LvoEntry { offset: -60, name: "_LVOClearPointer" },
    LvoEntry { offset: -54, name: "_LVOClearMenuStrip" },
    LvoEntry { offset: -48, name: "_LVOClearDMRequest" },
    LvoEntry { offset: -42, name: "_LVOAddGadget" },
    LvoEntry { offset: -36, name: "_LVOIntuition" },
    LvoEntry { offset: -30, name: "_LVOOpenIntuition" },
];

static GRAPHICS_ENTRIES: &[LvoEntry] = &[
    LvoEntry { offset: -618, name: "_LVOFreeBitMap" },
    LvoEntry { offset: -612, name: "_LVOAllocBitMap" },
    LvoEntry { offset: -504, name: "_LVOBltBitMapRastPort" },
    LvoEntry { offset: -444, name: "_LVOLoadRGB4" },
    LvoEntry { offset: -438, name: "_LVOMrgCop" },
    LvoEntry { offset: -432, name: "_LVOMakeVPort" },
    LvoEntry { offset: -420, name: "_LVOFreeRaster" },
    LvoEntry { offset: -414, name: "_LVOAllocRaster" },
    LvoEntry { offset: -408, name: "_LVOWaitBOVP" },
    LvoEntry { offset: -402, name: "_LVOScrollRaster" },
    LvoEntry { offset: -396, name: "_LVOInitBitMap" },
    LvoEntry { offset: -384, name: "_LVODisownBlitter" },
    LvoEntry { offset: -378, name: "_LVOOwnBlitter" },
    LvoEntry { offset: -372, name: "_LVOSetRast" },
    LvoEntry { offset: -366, name: "_LVOWaitBlit" },
    LvoEntry { offset: -360, name: "_LVOLoadView" },
    LvoEntry { offset: -336, name: "_LVOInitView" },
    LvoEntry { offset: -324, name: "_LVOSetDrMd" },
    LvoEntry { offset: -318, name: "_LVOSetBPen" },
    LvoEntry { offset: -312, name: "_LVOSetAPen" },
    LvoEntry { offset: -306, name: "_LVOPolyDraw" },
    LvoEntry { offset: -300, name: "_LVOFlood" },
    LvoEntry { offset: -288, name: "_LVOWritePixel" },
    LvoEntry { offset: -282, name: "_LVOReadPixel" },
    LvoEntry { offset: -276, name: "_LVOBltPattern" },
    LvoEntry { offset: -270, name: "_LVORectFill" },
    LvoEntry { offset: -264, name: "_LVOBltClear" },
    LvoEntry { offset: -258, name: "_LVOQBSBlit" },
    LvoEntry { offset: -252, name: "_LVOSetRGB4" },
    LvoEntry { offset: -240, name: "_LVOInitVPort" },
    LvoEntry { offset: -234, name: "_LVOInitRastPort" },
    LvoEntry { offset: -222, name: "_LVOAreaEnd" },
    LvoEntry { offset: -216, name: "_LVOAreaDraw" },
    LvoEntry { offset: -210, name: "_LVOAreaMove" },
    LvoEntry { offset: -204, name: "_LVODraw" },
    LvoEntry { offset: -198, name: "_LVOMove" },
    LvoEntry { offset: -90, name: "_LVOSetSoftStyle" },
    LvoEntry { offset: -84, name: "_LVOAskSoftStyle" },
    LvoEntry { offset: -78, name: "_LVOCloseFont" },
    LvoEntry { offset: -72, name: "_LVOOpenFont" },
    LvoEntry { offset: -66, name: "_LVOSetFont" },
    LvoEntry { offset: -60, name: "_LVOText" },
    LvoEntry { offset: -54, name: "_LVOTextLength" },
    LvoEntry { offset: -48, name: "_LVOClearScreen" },
    LvoEntry { offset: -42, name: "_LVOClearEOL" },
    LvoEntry { offset: -36, name: "_LVOBltTemplate" },
    LvoEntry { offset: -30, name: "_LVOBltBitMap" },
];

static ALL_LIBRARIES: &[Library] = &[
    Library { name: "exec", entries: EXEC_ENTRIES },
    Library { name: "dos", entries: DOS_ENTRIES },
    Library { name: "intuition", entries: INTUITION_ENTRIES },
    Library { name: "graphics", entries: GRAPHICS_ENTRIES },
];

/// Look up an LVO name by library name and offset.
///
/// Uses binary search since entries are sorted by offset.
pub fn lookup_lvo(library_name: &str, offset: i16) -> Option<&'static str> {
    let lib = ALL_LIBRARIES.iter().find(|l| l.name == library_name)?;
    // Entries are sorted descending by offset; binary search needs ascending,
    // so we search by negated offset comparison.
    lib.entries
        .binary_search_by(|e| e.offset.cmp(&offset))
        .ok()
        .map(|idx| lib.entries[idx].name)
}

/// Returns all built-in library definitions.
pub fn all_libraries() -> &'static [Library] {
    ALL_LIBRARIES
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_exec_open_library() {
        assert_eq!(lookup_lvo("exec", -552), Some("_LVOOpenLibrary"));
    }

    #[test]
    fn lookup_exec_close_library() {
        assert_eq!(lookup_lvo("exec", -504), Some("_LVOCloseLibrary"));
    }

    #[test]
    fn lookup_exec_forbid() {
        assert_eq!(lookup_lvo("exec", -132), Some("_LVOForbid"));
    }

    #[test]
    fn lookup_dos_open() {
        assert_eq!(lookup_lvo("dos", -30), Some("_LVOOpen"));
    }

    #[test]
    fn lookup_unknown_offset() {
        assert_eq!(lookup_lvo("exec", -999), None);
    }

    #[test]
    fn lookup_unknown_library() {
        assert_eq!(lookup_lvo("nonexistent", -552), None);
    }

    #[test]
    fn all_libraries_has_four() {
        assert_eq!(all_libraries().len(), 4);
    }

    #[test]
    fn entries_sorted_descending() {
        // Verify our sort invariant holds for all libraries
        for lib in all_libraries() {
            for window in lib.entries.windows(2) {
                assert!(
                    window[0].offset < window[1].offset,
                    "Library '{}': offset {} should be < {} (descending order)",
                    lib.name,
                    window[0].offset,
                    window[1].offset,
                );
            }
        }
    }
}
