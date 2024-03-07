use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use sharks::{Share, Sharks};
use std::str;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn simple_split(secret: &str, threshold: u8, shares: u8) -> Vec<String> {
    let sharks = Sharks(threshold);

    let dealer = sharks.dealer(secret.as_bytes());

    let mut shares_vec: Vec<String> = Vec::new();

    for s in dealer.take(usize::from(shares)) {
        let bytes = Vec::from(&s);
        shares_vec.push(URL_SAFE.encode(bytes));
    }

    shares_vec
}

#[tauri::command]
fn simple_combine(shares: Vec<String>) -> String {
    // Make sure that shares can't be bigger than u8
    let len = match u8::try_from(shares.len()) {
        Ok(l) => l,
        Err(_) => return "".to_string(),
    };

    let sharks = Sharks(len);

    let shares_vec: Vec<Share> = shares
        .iter()
        .map(|s| {
            let bytes = match URL_SAFE.decode(s.as_bytes()) {
                Ok(b) => b,
                Err(_) => Vec::new(),
            };
            Share::try_from(bytes.as_slice()).unwrap()
        })
        .collect();

    match sharks.recover(shares_vec.as_slice()) {
        Ok(secret) => str::from_utf8(&secret).unwrap().to_string(),
        Err(_) => "".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_split() {
        let secret = "Hello World!";
        let threshold = 2;
        let shares = 4;
        let result = simple_split(secret, threshold, shares);
        assert_eq!(result.len(), 4);
    }

    #[test]
    fn test_simple_combine() {
        let secret = "Hello World!";
        let threshold = 2;
        let shares = 4;
        let result = simple_split(secret, threshold, shares);

        // Works with all shares
        let combined = simple_combine(result.to_owned());
        assert_eq!(combined, secret);

        // Works with the first two shares
        let combined = simple_combine(result[0..2].to_vec().to_owned());
        assert_eq!(combined, secret);

        // Works with the last two shares
        let combined = simple_combine(result[1..3].to_vec().to_owned());
        assert_eq!(combined, secret);

        // Doesn't work with only one share
        let combined = simple_combine(result[0..1].to_vec().to_owned());
        assert_ne!(combined, secret);

        // Doesn't work with no shares
        let combined = simple_combine(vec![]);
        assert_ne!(combined, secret);

        // Doesn't work with invalid shares
        let combined = simple_combine(vec![URL_SAFE.encode("Hello World!".as_bytes())]);
        assert_ne!(combined, secret);

        // Doesn't work with invalid shares
        let combined = simple_combine(vec![URL_SAFE.encode("World!".as_bytes()), URL_SAFE.encode("Hello".as_bytes())]);
        assert_ne!(combined, secret);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![simple_split, simple_combine])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
