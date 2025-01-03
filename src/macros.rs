#[macro_export]
macro_rules! prompt_input {
    ($($arg:tt)*) => {{
        use std::io::{self, Write};
        print!($($arg)*);
        io::stdout().flush().unwrap(); // Flush the output to ensure it's displayed
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");
        input.trim().to_string()
    }};
}
