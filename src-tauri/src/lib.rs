use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use sharks::{Share, Sharks};
use std::str;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn simple_split(secret: &str, threshold: u8, shares: u8) -> Result<Vec<String>, String> {
    if threshold < 1 {
        return Err("invalid threshold: 0".to_string());
    }

    if shares < 1 {
        return Err("invalid shares: 0".to_string());
    }

    if threshold > shares {
        return Err("threshold can't be bigger than shares".to_string());
    }

    let sharks = Sharks(threshold);

    let dealer = sharks.dealer(secret.as_bytes());

    let mut shares_vec: Vec<String> = Vec::new();

    for s in dealer.take(usize::from(shares)) {
        let bytes = Vec::from(&s);
        shares_vec.push(URL_SAFE.encode(bytes));
    }

    Ok(shares_vec)
}

#[tauri::command]
fn simple_combine(shares: Vec<String>) -> Result<String, String> {
    // Make sure that shares can't be bigger than u8
    let len = u8::try_from(shares.len()).map_err(|err| err.to_string())?;

    let sharks = Sharks(len);

    let shares: Result<Vec<Share>, String> = shares
        .iter()
        .map(|s| -> Result<Share, String> {
            let bytes = URL_SAFE
                .decode(s.as_bytes())
                .map_err(|err| err.to_string())?;

            Share::try_from(bytes.as_slice()).map_err(|err| err.to_string())
        })
        .collect();
    let shares = shares?;

    let secret = sharks
        .recover(shares.as_slice())
        .map_err(|err| err.to_string())?;

    let secret = str::from_utf8(&secret).map_err(|err| err.to_string())?;

    Ok(secret.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::{Alphanumeric, DistString};
    use rand::{thread_rng, Rng};

    #[test]
    fn test_simple_split() {
        let secret = "Hello World!";
        let threshold = 2;
        let shares = 4;

        // Test with valid input
        let result = simple_split(secret, threshold, shares);

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), shares as usize);

        // Tests with invalid input
        let result = simple_split(secret, 0, shares);

        assert!(result.is_err());
        let result = result.unwrap_err();
        assert_eq!(result, "invalid threshold: 0");

        let result = simple_split(secret, threshold, 0);

        assert!(result.is_err());
        let result = result.unwrap_err();
        assert_eq!(result, "invalid shares: 0");

        let result = simple_split(secret, 4, 2);

        assert!(result.is_err());
        let result = result.unwrap_err();
        assert_eq!(result, "threshold can't be bigger than shares");
    }

    #[test]
    fn test_simple_combine() {
        let expected_secret = "Hello World!";
        let shares = vec![
            "AR0UGMgRlTD5XNUsyw==".to_string(),
            "AuKHhDmTV5leLgP06A==".to_string(),
            "A7f28J3t4v7IALq8Ag==".to_string(),
            "BAG8ocaKztYNyrJZrg==".to_string(),
        ];

        let result = simple_combine(shares.to_vec());

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected_secret);

        // Test only 2 shares
        let result = simple_combine(shares[0..2].to_vec());

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, expected_secret);

        // TODO: Test with invalid
    }

    #[test]
    fn test_simple_flow_loop() {
        let secret = "Hello World!";

        // Test with all combinations of threshold and shares
        for threshold in 1..=255 {
            // 80% change of skipping this loop
            if rand::random::<u8>() < 204 {
                continue;
            }

            for shares in threshold..=255 {
                // 80% change of skipping this loop
                if rand::random::<u8>() < 204 {
                    continue;
                }

                let result = simple_split(secret, threshold, shares);

                assert!(result.is_ok());
                let result = result.unwrap();
                assert_eq!(result.len(), shares as usize);

                // Test with all shares
                let combine = simple_combine(result.to_vec());

                assert!(combine.is_ok());
                let combine = combine.unwrap();
                assert_eq!(combine, secret);

                // Test with minimum shares
                let combine = simple_combine(result[0..threshold as usize].to_vec());

                assert!(combine.is_ok());
                let combine = combine.unwrap();
                assert_eq!(combine, secret);
            }
        }
    }

        #[test]
    fn test_simple_flow_random() {
        // Test with random secret, threshold and shares
        let mut rng = thread_rng();
        for _ in 0..=100 {
            let secret_len = rng.gen_range(0..=512);
            let secret = Alphanumeric.sample_string(&mut rng, secret_len);
            let threshold = rng.gen_range(1..=255);
            let shares = rng.gen_range(threshold..=255);

            let result = simple_split(&secret, threshold, shares);

            assert!(result.is_ok());
            let result = result.unwrap();
            assert_eq!(result.len(), shares as usize);

            let combine = simple_combine(result);

            assert!(combine.is_ok());
            let combine = combine.unwrap();
            assert_eq!(combine, secret);
        }
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
