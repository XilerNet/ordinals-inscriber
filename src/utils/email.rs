pub fn extract_main_email(email_with_label: &str) -> String {
    if let Some(plus_index) = email_with_label.find('+') {
        if let Some(at_index) = email_with_label.find('@') {
            return String::from(&email_with_label[..plus_index]) + &email_with_label[at_index..];
        }
    }
    email_with_label.to_string()
}
