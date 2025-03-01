use colorful::{
    core::{color_string::CString, colors},
    Colorful,
};

pub const LOGO: &str = r#"
_____                   _                 _            
 |_   _|   ___    _ __   | |__     ___     | |    __ _   
   | |    / _ \  | '  \  | '_ \   / _ \    | |   / _` |  
  _|_|_   \___/  |_|_|_| |_.__/   \___/   _|_|_  \__,_|  
_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""|_|"""""| 
"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"`-0-0-'"#;

pub const INFO: &str = r#"
To learn more about Tombola

Made with ❤️ by Tombo

----------"#;

pub fn styled_logo() -> CString {
    LOGO.color(colors::Color::Purple1b)
}

pub fn styled_info() -> CString {
    INFO.color(colors::Color::DarkGray)
}
