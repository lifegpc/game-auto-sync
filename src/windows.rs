use std::ffi::{OsStr, OsString};
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{addr_of_mut, null, null_mut};
use winapi::ctypes::c_void;
use winapi::shared::basetsd::ULONG_PTR;
use winapi::shared::minwindef::DWORD;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::ioapiset::{CreateIoCompletionPort, GetQueuedCompletionStatus};
use winapi::um::jobapi2::{AssignProcessToJobObject, SetInformationJobObject};
use winapi::um::minwinbase::LPOVERLAPPED;
use winapi::um::processthreadsapi::{
    CreateProcessW, GetExitCodeProcess, ResumeThread, PROCESS_INFORMATION, STARTUPINFOW,
};
use winapi::um::winbase::{CreateJobObjectA, CREATE_SUSPENDED, INFINITE};
use winapi::um::wincon::GetConsoleWindow;
use winapi::um::winnt::{
    JobObjectAssociateCompletionPortInformation, JOBOBJECT_ASSOCIATE_COMPLETION_PORT,
    JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO,
};
use winapi::um::winuser::{ShowWindow, SW_HIDE, SW_SHOW};

fn console_show_window(n_cmd_show: i32) -> bool {
    let h_wnd = unsafe { GetConsoleWindow() };
    if h_wnd.is_null() {
        println!("Failed to get console window.");
        return false;
    }
    unsafe { ShowWindow(h_wnd, n_cmd_show) != 0 }
}

pub fn show_window() -> bool {
    console_show_window(SW_SHOW)
}

pub fn hide_window() -> bool {
    console_show_window(SW_HIDE)
}

#[derive(Debug, derive_more::Display, derive_more::From)]
pub enum PopenError {
    CreateJobFailed,
    CreateProcessFailed,
    AssignJobFailed,
}

pub fn call<S: AsRef<OsStr>>(argv: &[S]) -> Result<u32, PopenError> {
    let job = unsafe { CreateJobObjectA(null_mut(), null()) };
    if job.is_null() {
        println!("Failed to create job: {}.", unsafe { GetLastError() });
        return Err(PopenError::CreateJobFailed);
    }
    let io_port = unsafe { CreateIoCompletionPort(INVALID_HANDLE_VALUE, null_mut(), 0, 1) };
    if io_port.is_null() {
        unsafe { CloseHandle(job) };
        println!("CreateIoCompletionPort: {}", unsafe { GetLastError() });
        return Err(PopenError::CreateJobFailed);
    }
    let mut port = JOBOBJECT_ASSOCIATE_COMPLETION_PORT::default();
    port.CompletionKey = job;
    port.CompletionPort = io_port;
    let ok = unsafe {
        SetInformationJobObject(
            job,
            JobObjectAssociateCompletionPortInformation,
            addr_of_mut!(port) as *mut c_void,
            size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as u32,
        ) != 0
    };
    if !ok {
        unsafe { CloseHandle(job) };
        unsafe { CloseHandle(io_port) };
        println!("SetInformationJobObject: {}", unsafe { GetLastError() });
        return Err(PopenError::CreateJobFailed);
    }
    let mut si = STARTUPINFOW::default();
    let mut pi = PROCESS_INFORMATION::default();
    let mut cml = OsString::new();
    for i in argv.iter() {
        let t = i.as_ref();
        if !cml.is_empty() {
            cml.push(" ");
        }
        if t.to_string_lossy().find(' ').is_some() {
            cml.push("\"");
            cml.push(
                t.to_string_lossy()
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\""),
            );
            cml.push("\"");
        } else {
            cml.push(
                t.to_string_lossy()
                    .replace("\\", "\\\\")
                    .replace("\"", "\\\""),
            );
        }
    }
    let mut cmlw: Vec<_> = cml.encode_wide().collect();
    cmlw.resize(cmlw.len() + 1000, 0);
    let re = unsafe {
        CreateProcessW(
            null(),
            cmlw.as_mut_ptr(),
            null_mut(),
            null_mut(),
            1,
            CREATE_SUSPENDED,
            null_mut(),
            null_mut(),
            addr_of_mut!(si),
            addr_of_mut!(pi),
        ) != 0
    };
    if !re {
        println!("Failed to create process: {}.", unsafe { GetLastError() });
        unsafe { CloseHandle(job) };
        unsafe { CloseHandle(io_port) };
        return Err(PopenError::CreateProcessFailed);
    }
    let re = unsafe { AssignProcessToJobObject(job, pi.hProcess) != 0 };
    if !re {
        println!("Failed to assign process to job.");
        unsafe { CloseHandle(job) };
        unsafe { CloseHandle(pi.hProcess) };
        unsafe { CloseHandle(pi.hThread) };
        unsafe { CloseHandle(io_port) };
        return Err(PopenError::AssignJobFailed);
    }
    unsafe { ResumeThread(pi.hThread) };
    let mut code = DWORD::default();
    let mut key = ULONG_PTR::default();
    let mut overlapped: LPOVERLAPPED = null_mut();
    while unsafe {
        GetQueuedCompletionStatus(
            io_port,
            addr_of_mut!(code),
            addr_of_mut!(key),
            addr_of_mut!(overlapped),
            INFINITE,
        )
    } != 0
        && !(job.wrapping_sub(key).is_null() && code == JOB_OBJECT_MSG_ACTIVE_PROCESS_ZERO)
    {}
    let mut c = DWORD::default();
    unsafe { GetExitCodeProcess(pi.hProcess, addr_of_mut!(c)) };
    unsafe { CloseHandle(job) };
    unsafe { CloseHandle(io_port) };
    unsafe { CloseHandle(pi.hThread) };
    unsafe { CloseHandle(pi.hProcess) };
    Ok(c)
}
