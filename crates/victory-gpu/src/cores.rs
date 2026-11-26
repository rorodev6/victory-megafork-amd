//! Which processors our own threads run on, where a processor has two kinds
//! of cores. `cpu_sets` and `keep_on_fast_cores` are fxceless/victory-fork's
//! (`framelog.rs`, commit 0b0074e0, GPL-3.0-or-later, as this).

/// The system's processor sets of group 0: each one's id, its efficiency
/// class (higher is faster) and its core (which two of them may share).
fn cpu_sets() -> Vec<(u32, u8, u8)> {
    use windows_sys::Win32::System::SystemInformation::{GetSystemCpuSetInformation, SYSTEM_CPU_SET_INFORMATION};
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    let mut bytes = 0u32;
    // SAFETY: a size query, then the call into a buffer of that size.
    unsafe { GetSystemCpuSetInformation(std::ptr::null_mut(), 0, &mut bytes, GetCurrentProcess(), 0) };
    let mut buffer = vec![0u64; (bytes as usize).div_ceil(8).max(1)];
    // SAFETY: as above; the buffer holds `bytes` bytes, aligned for the records.
    let ok = unsafe { GetSystemCpuSetInformation(buffer.as_mut_ptr().cast(), bytes, &mut bytes, GetCurrentProcess(), 0) };
    let mut sets = Vec::new();
    let mut at = 0usize;
    while ok != 0 && at + size_of::<SYSTEM_CPU_SET_INFORMATION>() <= bytes as usize {
        // SAFETY: a record inside the buffer; each says its own size.
        let info = unsafe { &*(buffer.as_ptr().cast::<u8>().add(at).cast::<SYSTEM_CPU_SET_INFORMATION>()) };
        if info.Size == 0 {
            break;
        }
        // SAFETY: the record's type is CpuSet (the only one there is).
        let set = unsafe { &info.Anonymous.CpuSet };
        if set.Group == 0 {
            sets.push((set.Id, set.EfficiencyClass, set.CoreIndex));
        }
        at += info.Size as usize;
    }
    sets
}

/// The bytes of the last-level cache each processor set of group 0 shares,
/// by set id (none known: empty).
fn l3_of_sets() -> Vec<(u32, u32)> {
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, GetSystemCpuSetInformation, RelationCache, SYSTEM_CPU_SET_INFORMATION,
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    // The largest cache of each logical processor of group 0, by its index.
    let mut by_processor = [0u32; 64];
    let mut bytes = 0u32;
    // SAFETY: a size query, then the call into a buffer of that size.
    unsafe { GetLogicalProcessorInformationEx(RelationCache, std::ptr::null_mut(), &mut bytes) };
    let mut buffer = vec![0u64; (bytes as usize).div_ceil(8).max(1)];
    // SAFETY: as above; the buffer holds `bytes` bytes, aligned for the records.
    let ok = unsafe { GetLogicalProcessorInformationEx(RelationCache, buffer.as_mut_ptr().cast(), &mut bytes) };
    let mut at = 0usize;
    while ok != 0 && at + size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>() <= bytes as usize {
        // SAFETY: a record inside the buffer; each says its own size.
        let info = unsafe { &*(buffer.as_ptr().cast::<u8>().add(at).cast::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>()) };
        if info.Size == 0 {
            break;
        }
        if info.Relationship == RelationCache {
            // SAFETY: a cache record (asked for), with at least its first group mask.
            let cache = unsafe { &info.Anonymous.Cache };
            let mask = unsafe { cache.Anonymous.GroupMask };
            if mask.Group == 0 {
                for (p, size) in by_processor.iter_mut().enumerate() {
                    if mask.Mask >> p & 1 != 0 {
                        *size = (*size).max(cache.CacheSize);
                    }
                }
            }
        }
        at += info.Size as usize;
    }
    // Processor indices to set ids.
    let mut bytes = 0u32;
    // SAFETY: as for `cpu_sets`.
    unsafe { GetSystemCpuSetInformation(std::ptr::null_mut(), 0, &mut bytes, GetCurrentProcess(), 0) };
    let mut buffer = vec![0u64; (bytes as usize).div_ceil(8).max(1)];
    // SAFETY: as above.
    let ok = unsafe { GetSystemCpuSetInformation(buffer.as_mut_ptr().cast(), bytes, &mut bytes, GetCurrentProcess(), 0) };
    let mut sizes = Vec::new();
    let mut at = 0usize;
    while ok != 0 && at + size_of::<SYSTEM_CPU_SET_INFORMATION>() <= bytes as usize {
        // SAFETY: a record inside the buffer; each says its own size.
        let info = unsafe { &*(buffer.as_ptr().cast::<u8>().add(at).cast::<SYSTEM_CPU_SET_INFORMATION>()) };
        if info.Size == 0 {
            break;
        }
        // SAFETY: the record's type is CpuSet (the only one there is).
        let set = unsafe { &info.Anonymous.CpuSet };
        if set.Group == 0 && let Some(&size) = by_processor.get(set.LogicalProcessorIndex as usize) {
            sizes.push((set.Id, size));
        }
        at += info.Size as usize;
    }
    sizes
}

/// The last-level cache each processor set of group 0 shares, by set id: the
/// lowest logical processor index under that cache (so the one holding
/// processor 0 is domain 0). On a Ryzen with two complexes alike (5900X,
/// 5950X, 7950X, 9950X) that is one domain per complex; the 3000 series has
/// one per four-core group.
fn l3_domain_of_sets() -> Vec<(u32, u32)> {
    use windows_sys::Win32::System::SystemInformation::{
        GetLogicalProcessorInformationEx, GetSystemCpuSetInformation, RelationCache, SYSTEM_CPU_SET_INFORMATION,
        SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX,
    };
    use windows_sys::Win32::System::Threading::GetCurrentProcess;
    // The largest cache of each logical processor of group 0: its size and the
    // processors it serves.
    let mut by_processor = [(0u32, 0usize); 64];
    let mut bytes = 0u32;
    // SAFETY: a size query, then the call into a buffer of that size.
    unsafe { GetLogicalProcessorInformationEx(RelationCache, std::ptr::null_mut(), &mut bytes) };
    let mut buffer = vec![0u64; (bytes as usize).div_ceil(8).max(1)];
    // SAFETY: as above; the buffer holds `bytes` bytes, aligned for the records.
    let ok = unsafe { GetLogicalProcessorInformationEx(RelationCache, buffer.as_mut_ptr().cast(), &mut bytes) };
    let mut at = 0usize;
    while ok != 0 && at + size_of::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>() <= bytes as usize {
        // SAFETY: a record inside the buffer; each says its own size.
        let info = unsafe { &*(buffer.as_ptr().cast::<u8>().add(at).cast::<SYSTEM_LOGICAL_PROCESSOR_INFORMATION_EX>()) };
        if info.Size == 0 {
            break;
        }
        if info.Relationship == RelationCache {
            // SAFETY: a cache record (asked for), with at least its first group mask.
            let cache = unsafe { &info.Anonymous.Cache };
            let mask = unsafe { cache.Anonymous.GroupMask };
            if mask.Group == 0 {
                for (p, slot) in by_processor.iter_mut().enumerate() {
                    if mask.Mask >> p & 1 != 0 && cache.CacheSize > slot.0 {
                        *slot = (cache.CacheSize, mask.Mask);
                    }
                }
            }
        }
        at += info.Size as usize;
    }
    // Processor indices to set ids.
    let mut bytes = 0u32;
    // SAFETY: as for `cpu_sets`.
    unsafe { GetSystemCpuSetInformation(std::ptr::null_mut(), 0, &mut bytes, GetCurrentProcess(), 0) };
    let mut buffer = vec![0u64; (bytes as usize).div_ceil(8).max(1)];
    // SAFETY: as above.
    let ok = unsafe { GetSystemCpuSetInformation(buffer.as_mut_ptr().cast(), bytes, &mut bytes, GetCurrentProcess(), 0) };
    let mut domains = Vec::new();
    let mut at = 0usize;
    while ok != 0 && at + size_of::<SYSTEM_CPU_SET_INFORMATION>() <= bytes as usize {
        // SAFETY: a record inside the buffer; each says its own size.
        let info = unsafe { &*(buffer.as_ptr().cast::<u8>().add(at).cast::<SYSTEM_CPU_SET_INFORMATION>()) };
        if info.Size == 0 {
            break;
        }
        // SAFETY: the record's type is CpuSet (the only one there is).
        let set = unsafe { &info.Anonymous.CpuSet };
        if set.Group == 0
            && let Some(&(size, mask)) = by_processor.get(set.LogicalProcessorIndex as usize)
            && size != 0
            && mask != 0
        {
            domains.push((set.Id, mask.trailing_zeros()));
        }
        at += info.Size as usize;
    }
    domains
}

/// Where our threads go on a processor of several last-level caches all of
/// one size (`VICTORY_CCD`): `far` (all of them away from the cache holding
/// processor 0, where the title pins its own threads, so that those have
/// their cores to themselves), `near` (the threads a frame waits for beside
/// the title's, on the cache its data is in; the many that translate lists
/// away from it), or off (unset or `0`: as before, anywhere). With
/// `VICTORY_AB=<s>,VICTORY_CCD` (diagnostics) it changes while the title
/// runs; `VICTORY_CCD_MODE=near` then picks `near` for its on phases.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ccd {
    Off,
    Near,
    Far,
}

fn ccd_from(text: Option<String>) -> Ccd {
    match text.as_deref().map(|v| v.trim().to_ascii_lowercase()).as_deref() {
        Some("far") | Some("1") => Ccd::Far,
        Some("near") => Ccd::Near,
        _ => Ccd::Off,
    }
}

fn ccd_static() -> Ccd {
    static MODE: std::sync::OnceLock<Ccd> = std::sync::OnceLock::new();
    *MODE.get_or_init(|| ccd_from(std::env::var("VICTORY_CCD").ok()))
}

fn ccd() -> Ccd {
    #[cfg(feature = "diagnostics")]
    if let Some(on) = crate::ab::pick("VICTORY_CCD") {
        if !on {
            return Ccd::Off;
        }
        return match ccd_from(std::env::var("VICTORY_CCD_MODE").ok()) {
            Ccd::Near => Ccd::Near,
            _ => Ccd::Far,
        };
    }
    ccd_static()
}

/// Whether [`ccd`] can be on at some point of this run.
fn ccd_possible() -> bool {
    #[cfg(feature = "diagnostics")]
    if crate::ab::bit("VICTORY_CCD").is_some() {
        return true;
    }
    ccd_static() != Ccd::Off
}

/// The sets of the cache holding processor 0 and those of the others, where
/// there are two or more last-level caches, all of one size, and no class or
/// cache already makes some cores the fast ones (`domains`: domain by set id,
/// [`l3_domain_of_sets`]). `None` otherwise (one complex, X3D, two kinds of
/// cores, or nothing known).
fn ccd_split(sets: &[(u32, u8, u8)], l3: &[(u32, u32)], domains: &[(u32, u32)]) -> Option<(Vec<u32>, Vec<u32>)> {
    if !fast_by_class(sets).is_empty() || !fast_by_cache(sets, l3).is_empty() {
        return None;
    }
    let domain = |id: u32| domains.iter().find(|d| d.0 == id).map(|d| d.1);
    if sets.iter().any(|s| domain(s.0).is_none()) {
        return None;
    }
    let mut ids: Vec<u32> = sets.iter().filter_map(|s| domain(s.0)).collect();
    ids.sort_unstable();
    ids.dedup();
    let &home = ids.first()?;
    if ids.len() < 2 {
        return None;
    }
    let near: Vec<u32> = sets.iter().filter(|s| domain(s.0) == Some(home)).map(|s| s.0).collect();
    let far: Vec<u32> = sets.iter().filter(|s| domain(s.0) != Some(home)).map(|s| s.0).collect();
    Some((near, far))
}

fn l3_sizes() -> &'static [(u32, u32)] {
    static L3: std::sync::OnceLock<Vec<(u32, u32)>> = std::sync::OnceLock::new();
    L3.get_or_init(l3_of_sets)
}

fn l3_domains() -> &'static [(u32, u32)] {
    static DOMAINS: std::sync::OnceLock<Vec<(u32, u32)>> = std::sync::OnceLock::new();
    DOMAINS.get_or_init(l3_domain_of_sets)
}

/// Whether the cores of the largest last-level cache count as the fast
/// ones where all cores are of one class (`VICTORY_CACHE_CORES=0`: not).
/// On a processor with two core complexes of which one has the stacked cache
/// (Ryzen 7900X3D, 7950X3D, 9900X3D, 9950X3D) every core reports the same
/// efficiency class, so nothing was asked for: our threads ran on either
/// complex. Their work (compares, hashes, copies of tens of megabytes a
/// frame) is the kind the larger cache serves. Nothing changes on a processor
/// whose caches are all alike, or with two kinds of cores.
fn cache_cores() -> bool {
    #[cfg(feature = "diagnostics")]
    if let Some(on) = crate::ab::pick("VICTORY_CACHE_CORES") {
        return on;
    }
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_CACHE_CORES").is_none_or(|v| v != "0"))
}

/// The sets of the fastest class, if there are slower ones too.
fn fast_by_class(sets: &[(u32, u8, u8)]) -> Vec<u32> {
    let best = sets.iter().map(|c| c.1).max().unwrap_or(0);
    let fast: Vec<u32> = sets.iter().filter(|c| c.1 == best).map(|c| c.0).collect();
    if fast.len() == sets.len() { Vec::new() } else { fast }
}

/// The sets on the largest last-level cache (`l3`: bytes by set id), if
/// another one is smaller and the largest serves at least eight sets.
fn fast_by_cache(sets: &[(u32, u8, u8)], l3: &[(u32, u32)]) -> Vec<u32> {
    let size = |id: u32| l3.iter().find(|s| s.0 == id).map_or(0, |s| s.1);
    let sizes: Vec<u32> = sets.iter().map(|s| size(s.0)).collect();
    let (Some(&most), Some(&least)) = (sizes.iter().max(), sizes.iter().min()) else {
        return Vec::new();
    };
    if least == 0 || least == most {
        return Vec::new();
    }
    let fast: Vec<u32> = sets.iter().filter(|s| size(s.0) == most).map(|s| s.0).collect();
    if fast.len() < 8 { Vec::new() } else { fast }
}

/// The fast sets: those of the fastest class if there are slower ones, else
/// those of the largest cache ([`cache_cores`]), else, where the caches are
/// several and alike, those [`ccd`] picks (`far`: the caches without
/// processor 0; `near`: the one with it). `now`: as things are at this
/// moment (else whether there could be any, for threads to ask at all).
fn fast_sets(sets: &[(u32, u8, u8)], now: bool) -> Vec<u32> {
    let by_class = fast_by_class(sets);
    if !by_class.is_empty() {
        return by_class;
    }
    if !now || cache_cores() {
        let by_cache = fast_by_cache(sets, l3_sizes());
        if !by_cache.is_empty() {
            return by_cache;
        }
    }
    let mode = if now {
        ccd()
    } else if ccd_possible() {
        Ccd::Far
    } else {
        Ccd::Off
    };
    match (mode, ccd_split(sets, l3_sizes(), l3_domains())) {
        (Ccd::Near, Some((near, _))) => near,
        (Ccd::Far, Some((_, far))) => far,
        _ => Vec::new(),
    }
}

fn fast_of(sets: &[(u32, u8, u8)]) -> Vec<u32> {
    fast_sets(sets, true)
}

/// The sets of one fast core of the last, and all the others, for
/// [`lane_core`]; `None` with fewer than six fast cores.
fn one_core(sets: &[(u32, u8, u8)]) -> Option<(Vec<u32>, Vec<u32>)> {
    let best = sets.iter().map(|c| c.1).max()?;
    let mut cores: Vec<u8> = sets.iter().filter(|c| c.1 == best).map(|c| c.2).collect();
    cores.sort_unstable();
    cores.dedup();
    let core = *cores.last().filter(|_| cores.len() >= 6)?;
    let of = |own: bool| sets.iter().filter(|c| (c.2 == core) == own).map(|c| c.0).collect();
    Some((of(true), of(false)))
}

/// Whether the submitting thread has a fast core to itself, as far as this
/// process goes (`VICTORY_LANE_CORE=1`): nothing else of it runs on the
/// core's two threads. It was the one a frame waited for (94 % of it in a
/// street), and shared its core's units with whatever ran beside it.
fn lane_core() -> bool {
    #[cfg(feature = "diagnostics")]
    if let Some(on) = crate::ab::pick("VICTORY_LANE_CORE") {
        return on;
    }
    static ON: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("VICTORY_LANE_CORE").is_some_and(|v| v == "1"))
}

/// Threads that asked: a handle of each, whether it is the submitting one,
/// and its stage (1 to 3, as [`stage`]).
static ASKED: std::sync::Mutex<Vec<(usize, bool, u32)>> = std::sync::Mutex::new(Vec::new());

/// What the threads of the process, and those that asked, are told now
/// ([`lane_core`], which may have changed).
pub(crate) fn settle() {
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, SetProcessDefaultCpuSets, SetThreadSelectedCpuSets};
    if stage() == 0 {
        return;
    }
    let sets = cpu_sets();
    let fast = fast_of(&sets);
    let one = one_core(&sets).filter(|_| lane_core());
    let others: &[u32] = one.as_ref().map_or(&[], |o| &o.1);
    let null = std::ptr::null::<u32>();
    // SAFETY: our own process and the system's own set ids (none: all).
    unsafe {
        let at = if others.is_empty() { null } else { others.as_ptr() };
        SetProcessDefaultCpuSets(GetCurrentProcess(), at, others.len() as u32);
    }
    let helpers: Vec<u32> = match &one {
        Some((own, _)) => fast.iter().copied().filter(|id| !own.contains(id)).collect(),
        None => fast.clone(),
    };
    // `VICTORY_CCD=near`: the many threads that translate lists go away from
    // the title's cache, the rest stays beside it.
    let mode = ccd();
    let away: Option<Vec<u32>> =
        if mode == Ccd::Near { ccd_split(&sets, l3_sizes(), l3_domains()).map(|(_, far)| far) } else { None };
    for &(thread, lane, from) in ASKED.lock().unwrap_or_else(|e| e.into_inner()).iter() {
        let sets = match (&one, &away) {
            (Some((own, _)), _) if lane => own,
            (_, Some(far)) if from == 3 => far,
            _ => &helpers,
        };
        // SAFETY: a handle kept open, and the system's own set ids.
        unsafe { SetThreadSelectedCpuSets(thread as _, sets.as_ptr(), sets.len() as u32) };
    }
    tracing::info!(on = one.is_some(), core = ?one.as_ref().map(|o| &o.0), "the submitting thread has a core of its own (VICTORY_LANE_CORE)");
    tracing::info!(
        fast = fast.len(),
        of = sets.len(),
        by_cache = fast_by_class(&sets).is_empty() && !fast.is_empty(),
        ccd = ?mode,
        threads = ASKED.lock().unwrap_or_else(|e| e.into_inner()).len(),
        "our threads' processors (VICTORY_CACHE_CORES, VICTORY_CCD)"
    );
}

/// `VICTORY_FAST_CORES`: 0: no thread asks; 1: the threads a frame waits for
/// one by one; 2 (the default): the threads they share their work with too;
/// 3: those that work beside the rest as well (lists translated, descriptors
/// made): more threads by then than the fast cores run at a time, while the
/// slow ones stood idle.
fn stage() -> u32 {
    static STAGE: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
    *STAGE.get_or_init(|| std::env::var("VICTORY_FAST_CORES").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(2))
}

/// Asks the system to run the calling thread, which a frame waits for, on
/// the fast cores only, where the processor has two kinds. That fork's submit
/// thread was found on a slow one in 1,529 looks of 24,021. (A selection of
/// processors is a preference the scheduler keeps unless nothing of it is
/// free.) The title's threads are not asked for.
pub(crate) fn keep_on_fast_cores(what: &str) {
    ask(what, 1);
}

/// [`keep_on_fast_cores`] for one of the threads a submission's preparing or
/// a landing shares its work with.
pub(crate) fn keep_helper_on_fast_cores(what: &str) {
    ask(what, 2);
}

/// [`keep_on_fast_cores`] for one of many threads whose work is ahead of what
/// a frame waits for.
pub(crate) fn keep_many_on_fast_cores(what: &str) {
    ask(what, 3);
}

fn ask(what: &str, from: u32) {
    use windows_sys::Win32::System::Threading::{
        GetCurrentThreadId, OpenThread, THREAD_SET_LIMITED_INFORMATION,
    };
    // The many (stage 3) ask too where `VICTORY_CCD` may place them.
    if stage() < from && !(from == 3 && ccd_possible()) {
        return;
    }
    static FAST: std::sync::OnceLock<(Vec<u32>, usize)> = std::sync::OnceLock::new();
    let (fast, of) = FAST.get_or_init(|| {
        let sets = cpu_sets();
        // (whether there can be fast sets at all: `VICTORY_CACHE_CORES` may
        // change while the title runs, and `settle` follows it)
        (fast_sets(&sets, false), sets.len())
    });
    // All alike (or nothing known): nothing to ask for.
    if fast.is_empty() {
        return;
    }
    // SAFETY: our own thread; the handle stays open.
    let thread = unsafe { OpenThread(THREAD_SET_LIMITED_INFORMATION, 0, GetCurrentThreadId()) };
    if thread.is_null() {
        tracing::warn!(what, error = %std::io::Error::last_os_error(), "the thread could not be kept on the fast cores");
        return;
    }
    ASKED.lock().unwrap_or_else(|e| e.into_inner()).push((thread as usize, what == "victory-submit", from));
    settle();
    tracing::info!(what, fast = fast.len(), of, "the thread is kept on the fast cores (VICTORY_FAST_CORES=0: not)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fast_cores_are_those_of_the_best_class_if_there_is_a_worse_one() {
        assert_eq!(fast_by_class(&[(256, 1, 0), (257, 1, 0), (258, 0, 2)]), [256, 257]);
        assert!(fast_by_class(&[(256, 0, 0), (257, 0, 1)]).is_empty());
        assert!(one_core(&[(256, 1, 0), (257, 1, 0), (258, 0, 2)]).is_none());
        let eight: Vec<(u32, u8, u8)> = (0..16).map(|n| (n, 1, n as u8 / 2 * 2)).chain((16..24).map(|n| (n, 0, n as u8))).collect();
        let (own, others) = one_core(&eight).unwrap();
        assert_eq!((own, others.len()), (vec![14, 15], 22));
        assert!(fast_by_class(&[]).is_empty());
        assert!(!cpu_sets().is_empty());
        std::thread::spawn(|| keep_on_fast_cores("test")).join().unwrap();
    }

    const MB: u32 = 1 << 20;

    /// `n` sets of one class, two to a core, from id 256.
    fn alike(n: u32) -> Vec<(u32, u8, u8)> {
        (0..n).map(|i| (256 + i, 0, (i / 2) as u8)).collect()
    }

    #[test]
    fn the_largest_cache_is_fast_only_where_the_caches_differ() {
        // Two complexes, one with the stacked cache (9950X3D: 16 sets each).
        let sets = alike(32);
        let l3: Vec<(u32, u32)> = (0..32).map(|i| (256 + i, if i < 16 { 96 * MB } else { 32 * MB })).collect();
        assert_eq!(fast_by_cache(&sets, &l3), (256..272).collect::<Vec<_>>());
        // The stacked cache on the second complex: that one.
        let flipped: Vec<(u32, u32)> = (0..32).map(|i| (256 + i, if i < 16 { 32 * MB } else { 96 * MB })).collect();
        assert_eq!(fast_by_cache(&sets, &flipped), (272..288).collect::<Vec<_>>());
        // Two complexes alike (9950X): nothing.
        let even: Vec<(u32, u32)> = (0..32).map(|i| (256 + i, 32 * MB)).collect();
        assert!(fast_by_cache(&sets, &even).is_empty());
        // One complex (7800X3D, 9800X3D): nothing.
        let one: Vec<(u32, u32)> = (0..16).map(|i| (256 + i, 96 * MB)).collect();
        assert!(fast_by_cache(&alike(16), &one).is_empty());
        // Caches not known: nothing.
        assert!(fast_by_cache(&sets, &[]).is_empty());
        assert!(fast_by_cache(&sets, &l3[..16]).is_empty());
        // A larger cache of under eight sets: nothing.
        let few: Vec<(u32, u32)> = (0..32).map(|i| (256 + i, if i < 4 { 64 * MB } else { 32 * MB })).collect();
        assert!(fast_by_cache(&sets, &few).is_empty());
    }

    /// Domains by set id from 256: `per` sets to each cache.
    fn domains(n: u32, per: u32) -> Vec<(u32, u32)> {
        (0..n).map(|i| (256 + i, i / per * per)).collect()
    }

    #[test]
    fn several_caches_alike_split_into_the_one_with_processor_0_and_the_rest() {
        let even = |n: u32| -> Vec<(u32, u32)> { (0..n).map(|i| (256 + i, 32 * MB)).collect() };
        // Two complexes alike (5950X, 7950X, 9950X): 16 sets each.
        let (near, far) = ccd_split(&alike(32), &even(32), &domains(32, 16)).unwrap();
        assert_eq!(near, (256..272).collect::<Vec<_>>());
        assert_eq!(far, (272..288).collect::<Vec<_>>());
        // Twelve cores (5900X): 12 sets each.
        let (near, far) = ccd_split(&alike(24), &even(24), &domains(24, 12)).unwrap();
        assert_eq!((near.len(), far.len()), (12, 12));
        // Zen 2 (3950X): four caches of eight sets; processor 0's is one of them.
        let (near, far) = ccd_split(&alike(32), &even(32), &domains(32, 8)).unwrap();
        assert_eq!((near.len(), far.len()), (8, 24));
        // One complex (9800X3D): nothing.
        assert!(ccd_split(&alike(16), &even(16), &domains(16, 16)).is_none());
        // The stacked cache on one complex (9950X3D): left to `fast_by_cache`.
        let x3d: Vec<(u32, u32)> = (0..32).map(|i| (256 + i, if i < 16 { 96 * MB } else { 32 * MB })).collect();
        assert!(ccd_split(&alike(32), &x3d, &domains(32, 16)).is_none());
        // Two kinds of cores: left to `fast_by_class`.
        let hybrid: Vec<(u32, u8, u8)> = (0..16).map(|n| (n, 1, n as u8 / 2)).chain((16..24).map(|n| (n, 0, n as u8))).collect();
        let hd: Vec<(u32, u32)> = (0..24).map(|i| (i, if i < 16 { 0 } else { 16 })).collect();
        assert!(ccd_split(&hybrid, &[], &hd).is_none());
        // A set of unknown domain, or nothing known: nothing.
        assert!(ccd_split(&alike(32), &even(32), &domains(31, 16)).is_none());
        assert!(ccd_split(&alike(32), &even(32), &[]).is_none());
    }

    #[test]
    fn the_mode_reads_far_near_or_off() {
        assert_eq!(ccd_from(Some("far".into())), Ccd::Far);
        assert_eq!(ccd_from(Some(" FAR ".into())), Ccd::Far);
        assert_eq!(ccd_from(Some("1".into())), Ccd::Far);
        assert_eq!(ccd_from(Some("near".into())), Ccd::Near);
        assert_eq!(ccd_from(Some("0".into())), Ccd::Off);
        assert_eq!(ccd_from(Some("other".into())), Ccd::Off);
        assert_eq!(ccd_from(None), Ccd::Off);
    }

    #[test]
    fn this_machine_has_a_domain_for_every_set() {
        let sets = cpu_sets();
        let domains = l3_domain_of_sets();
        assert_eq!(domains.len(), sets.len());
        assert!(domains.iter().any(|d| d.1 == 0));
    }

    #[test]
    fn two_kinds_of_cores_still_go_by_their_class() {
        // Eight fast cores (two sets each) and eight slow ones, the caches
        // told to differ too: the class decides, whatever the caches say.
        let sets: Vec<(u32, u8, u8)> =
            (0..16).map(|n| (n, 1, n as u8 / 2 * 2)).chain((16..24).map(|n| (n, 0, n as u8))).collect();
        assert_eq!(fast_sets(&sets, true), (0..16).collect::<Vec<_>>());
        assert_eq!(fast_sets(&sets, false), (0..16).collect::<Vec<_>>());
    }
}
