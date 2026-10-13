use libc::{
    self, _exit, PTRACE_GETREGS, PTRACE_SYSCALL, PTRACE_TRACEME, WUNTRACED, c_char, c_int, execv,
    execve, fork, ptrace, waitpid,
};
use std::env;
use std::ffi::CString;
use std::io::Error;
use std::ptr;

fn run_binary(raw_input: Vec<String>) {
    if let Some((binary, args)) = raw_input.split_first() {
        let binary_path = CString::new(binary.as_str()).expect("Cstring failed on binary path");

        let c_args_string: Vec<CString> = args
            .iter()
            .map(|s| CString::new(s.as_str()).expect("CString failed"))
            .collect();
        let mut argv: Vec<*const c_char> = c_args_string.iter().map(|s| s.as_ptr()).collect();
        argv.push(ptr::null());

        unsafe {
            let id: i32 = fork();

            if id == 0 {
                ptrace(PTRACE_TRACEME as u32, 0, 0, 0);

                execve(binary_path.as_ptr(), argv.as_ptr(), ptr::null());

                let execv_error = Error::last_os_error();
                if execv_error.raw_os_error().is_some() {
                    eprintln!("Execv error : {:?}", execv_error);
                    _exit(1);
                }
            } else {
                let mut status: c_int = 0;
                waitpid(id, &mut status as *mut c_int, WUNTRACED as i32);

                loop {
                    ptrace(PTRACE_SYSCALL, id, 0, 0);
                    waitpid(id, &mut status as *mut c_int, WUNTRACED as i32);

                    let mut regs: libc::user_regs_struct = std::mem::zeroed();
                    ptrace(PTRACE_GETREGS, id, 0, &mut regs);
                    println!("data : {:?}", regs);

                    ptrace(PTRACE_SYSCALL, id, 0, 0);
                    waitpid(id, &mut status as *mut c_int, WUNTRACED as i32);

                    println!("\n#############################################\n");

                    ptrace(PTRACE_GETREGS, id, 0, &mut regs);
                    println!("data : {:?}", regs);

                    break;
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    println!("Fichier lu : {:?}", args);
    run_binary(args);
}
