//! Fuzz target for IRC message parsing
//!
//! This fuzzer tests the robustness of the IRC message parser by feeding it
//! randomly generated input data and ensuring it doesn't panic or crash.

#![no_main]

use libfuzzer_sys::fuzz_target;
use std::str;

fuzz_target!(|data: &[u8]| {
    // Only fuzz valid UTF-8 strings to focus on protocol-level issues
    if let Ok(input) = str::from_utf8(data) {
        // Skip empty inputs and very long inputs (over 512 bytes is unusual for IRC)
        if input.is_empty() || input.len() > 512 {
            return;
        }
        
        // Test message parsing - should never panic
        let _ = input.parse::<slirc_proto::Message>();
        
        // Test IRC codec sanitization - should never panic
        let _ = slirc_proto::IrcCodec::sanitize(input.to_string());
    }
});