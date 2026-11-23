//! Where the process's memory goes: one line for the log – at a crash, when the process ends
//! with an error, and while the title runs whenever its commit charge has grown by a GiB
//! (at most every 30 s). A title that runs out of memory dies in an allocation of its own,
//! far from the cause (a write to address 0 in a thread's start-up, a PSO the driver refuses
//! with E_OUTOFMEMORY); the line says whose memory it is: the process's commit against the
//! system's limit, the video memory in use (local, and the system memory the driver spills
//! into), what the title asked kernelx for (graphics memory, Windows allocations, heaps) and
//! the tiles of title memory that have video memory.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use windows::core::Interface;
use windows::Win32::Graphics::Direct3D12::ID3D12Device;
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter3, IDXGIFactory4, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL, DXGI_QUERY_VIDEO_MEMORY_INFO};
use windows_sys::Win32::System::ProcessStatus::{K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX};
use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

/// Kinds of memory the title asks kernelx for (`note`).
pub const GRAPHICS: usize = 0;
pub const WINDOWS: usize = 1;
pub const HEAP: usize = 2;
pub const PHYSICAL: usize = 3;

/// Bytes the title asked for since the start, by kind (gross: frees are not taken off).
static TITLE: [AtomicU64; 4] = [const { AtomicU64::new(0) }; 4];

/// The title committed `bytes` of `kind` (`GRAPHICS`: memory that gets video memory – the
/// sparse windows, UMA; `WINDOWS`: an ordinary VirtualAlloc commit; `HEAP`: XMemAlloc;
/// `PHYSICAL`: the homes of its physical pages).
pub fn note(kind: usize, bytes: usize) {
    TITLE[kind].fetch_add(bytes as u64, Ordering::Relaxed);
}

fn gib(b: u64) -> String {
    format!("{:.1} GiB", b as f64 / (1u64 << 30) as f64)
}

/// (commit charge, working set) of the process
fn process() -> Option<(u64, u64)> {
    unsafe {
        let mut c: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
        c.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        (K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c as *mut _ as *mut _, c.cb) != 0).then_some((c.PrivateUsage as u64, c.WorkingSetSize as u64))
    }
}

/// (commit in use, commit limit, physical memory free, load %) of the system
fn system() -> Option<(u64, u64, u64, u32)> {
    unsafe {
        let mut m: MEMORYSTATUSEX = std::mem::zeroed();
        m.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
        (GlobalMemoryStatusEx(&mut m) != 0).then_some((m.ullTotalPageFile.saturating_sub(m.ullAvailPageFile), m.ullTotalPageFile, m.ullAvailPhys, m.dwMemoryLoad))
    }
}

static ADAPTER: Mutex<Option<IDXGIAdapter3>> = Mutex::new(None);

/// The device's adapter, found once (None: no device yet, or the lock is held – a crash in a
/// thread that holds it must not wait on it).
fn adapter() -> Option<IDXGIAdapter3> {
    let mut g = ADAPTER.try_lock().ok()?;
    if g.is_none() {
        let raw = crate::hle::d3d12x::real::device_raw();
        if raw.is_null() {
            return None;
        }
        let dev: &ID3D12Device = unsafe { ID3D12Device::from_raw_borrowed(&raw)? };
        let luid = unsafe { dev.GetAdapterLuid() };
        let factory: IDXGIFactory4 = unsafe { CreateDXGIFactory1() }.ok()?;
        *g = unsafe { factory.EnumAdapterByLuid::<IDXGIAdapter3>(luid) }.ok();
    }
    g.clone()
}

/// ((in use, budget) local video memory, (in use, budget) non-local: system memory the driver uses)
fn video() -> Option<((u64, u64), (u64, u64))> {
    let a = adapter()?;
    let mut local = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
    let mut shared = DXGI_QUERY_VIDEO_MEMORY_INFO::default();
    unsafe { a.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, &mut local) }.ok()?;
    unsafe { a.QueryVideoMemoryInfo(0, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL, &mut shared) }.ok()?;
    Some(((local.CurrentUsage, local.Budget), (shared.CurrentUsage, shared.Budget)))
}

/// The line: `memory: process commit 61.2 GiB (working set 24.1 GiB); system commit 74.9 of
/// 79.5 GiB, 5.1 GiB of RAM free (84 % load); video 11.3 of 11.6 GiB local, 18.0 of 15.9 GiB
/// shared; the title asked for graphics 20.1 GiB, Windows 1.2 GiB, heaps 3.4 GiB; sparse
/// windows 180224 tiles (11.0 GiB) in 176 pools`.
pub fn line() -> String {
    let mut parts = Vec::new();
    if let Some((commit, ws)) = process() {
        parts.push(format!("process commit {} (working set {})", gib(commit), gib(ws)));
    }
    if let Some((used, limit, free, load)) = system() {
        parts.push(format!("system commit {} of {}, {} of RAM free ({load} % load)", gib(used), gib(limit), gib(free)));
    }
    if let Some(((lu, lb), (su, sb))) = video() {
        parts.push(format!("video {} of {} local, {} of {} shared", gib(lu), gib(lb), gib(su), gib(sb)));
    }
    let t = TITLE.each_ref().map(|a| a.load(Ordering::Relaxed));
    parts.push(format!("the title asked for graphics {}, Windows {}, heaps {}, physical pages {}", gib(t[GRAPHICS]), gib(t[WINDOWS]), gib(t[HEAP]), gib(t[PHYSICAL])));
    if let Some((tiles, pools)) = crate::hle::d3d12x::sparse::stats() {
        parts.push(format!("sparse windows {tiles} tiles ({}) in {pools} pools", gib((tiles as u64) << 16)));
    }
    format!("memory: {}", parts.join("; "))
}

/// The log line now (a crash, the process ending with an error).
pub fn log() {
    crate::xlog_info!("{}", line());
}

/// A thread that logs the line whenever the process's commit charge has grown by a GiB since
/// the last line, looking every 30 s.
pub fn start() {
    let _ = std::thread::Builder::new().name("xcl-memstat".into()).spawn(|| {
        let mut last = 0u64;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(30));
            let Some((commit, _)) = process() else { continue };
            if commit >= last + (1 << 30) {
                last = commit;
                log();
            }
        }
    });
}
