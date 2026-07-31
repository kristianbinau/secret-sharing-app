use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use blahaj::{Share, Sharks};
use std::collections::HashMap;
use std::str;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
struct GroupConfig {
    threshold: u8,
    count: u8,
    #[serde(default)]
    groups: Vec<GroupConfig>,
}

#[derive(Clone, Debug)]
struct ParsedShare {
    thresholds: Vec<u8>,
    group_indices: Vec<u8>,
    leaf_threshold: u8,
    raw_bytes: Vec<u8>,
}

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

#[tauri::command]
fn nested_split(
    secret: &str,
    threshold: u8,
    groups: Vec<GroupConfig>,
) -> Result<Vec<String>, String> {
    let parsed = do_split(secret.as_bytes(), threshold, &groups)?;
    Ok(parsed.iter().map(encode_share).collect())
}

fn do_split(
    secret: &[u8],
    threshold: u8,
    groups: &[GroupConfig],
) -> Result<Vec<ParsedShare>, String> {
    if threshold < 1 {
        return Err("invalid threshold: 0".to_string());
    }
    if groups.is_empty() {
        return Err("groups cannot be empty".to_string());
    }
    if groups.len() > 255 {
        return Err("too many groups (max 255)".to_string());
    }
    if threshold > groups.len() as u8 {
        return Err("threshold can't be bigger than groups".to_string());
    }

    for g in groups {
        if g.threshold < 1 {
            return Err("invalid group threshold: 0".to_string());
        }
        if g.count < 1 {
            return Err("invalid group count: 0".to_string());
        }
        if g.threshold > g.count {
            return Err("group threshold can't be bigger than count".to_string());
        }
        if !g.groups.is_empty() && g.groups.len() != g.count as usize {
            return Err("group count doesn't match number of sub-groups".to_string());
        }
    }

    let sharks = Sharks(threshold);
    let dealer = sharks.dealer(secret);
    let group_shares: Vec<Vec<u8>> = dealer.take(groups.len()).map(|s| Vec::from(&s)).collect();

    let mut result = Vec::new();

    for (i, group_share) in group_shares.iter().enumerate() {
        let x_coord = group_share[0];
        let group = &groups[i];

        if group.groups.is_empty() {
            let leaf_sharks = Sharks(group.threshold);
            let leaf_dealer = leaf_sharks.dealer(group_share);
            for leaf_share in leaf_dealer.take(group.count as usize) {
                let leaf_bytes = Vec::from(&leaf_share);
                result.push(ParsedShare {
                    thresholds: vec![threshold],
                    group_indices: vec![x_coord],
                    leaf_threshold: group.threshold,
                    raw_bytes: leaf_bytes,
                });
            }
        } else {
            let sub_result = do_split(group_share, group.threshold, &group.groups)?;
            for mut ps in sub_result {
                ps.thresholds.insert(0, threshold);
                ps.group_indices.insert(0, x_coord);
                result.push(ps);
            }
        }
    }

    Ok(result)
}

fn encode_share(ps: &ParsedShare) -> String {
    let mut bytes = vec![0x00];
    bytes.push(ps.thresholds.len() as u8);
    for i in 0..ps.thresholds.len() {
        bytes.push(ps.thresholds[i]);
        bytes.push(ps.group_indices[i]);
    }
    bytes.push(ps.leaf_threshold);
    bytes.extend(&ps.raw_bytes);
    URL_SAFE.encode(&bytes)
}

fn decode_share_from_bytes(bytes: &[u8]) -> Result<ParsedShare, String> {
    if bytes.len() < 2 {
        return Err("share too short".to_string());
    }
    if bytes[0] != 0x00 {
        return Err("not a nested share".to_string());
    }
    let depth = bytes[1] as usize;
    if depth < 1 {
        return Err("invalid depth: 0".to_string());
    }
    let needed = 2 + depth * 2 + 1;
    if bytes.len() < needed + 2 {
        return Err("share too short for declared depth".to_string());
    }
    let mut thresholds = Vec::with_capacity(depth);
    let mut group_indices = Vec::with_capacity(depth);
    let mut offset = 2;
    for _ in 0..depth {
        thresholds.push(bytes[offset]);
        group_indices.push(bytes[offset + 1]);
        offset += 2;
    }
    let leaf_threshold = bytes[offset];
    offset += 1;
    let raw_bytes = bytes[offset..].to_vec();
    Ok(ParsedShare {
        thresholds,
        group_indices,
        leaf_threshold,
        raw_bytes,
    })
}

type ShareEntry = (Vec<u8>, Vec<u8>, Vec<u8>, u8);
type GroupedShares = HashMap<Vec<u8>, Vec<(Vec<u8>, Vec<u8>, u8)>>;

#[tauri::command]
fn nested_combine(shares: Vec<String>) -> Result<String, String> {
    if shares.is_empty() {
        return Err("no shares provided".to_string());
    }

    let first_bytes = URL_SAFE
        .decode(shares[0].as_bytes())
        .map_err(|e| e.to_string())?;

    if first_bytes.is_empty() || first_bytes[0] != 0x00 {
        for s in &shares {
            let b = URL_SAFE.decode(s.as_bytes()).map_err(|e| e.to_string())?;
            if !b.is_empty() && b[0] == 0x00 {
                return Err("cannot mix simple and nested shares".to_string());
            }
        }
        return simple_combine(shares);
    }

    for s in &shares {
        let b = URL_SAFE.decode(s.as_bytes()).map_err(|e| e.to_string())?;
        if b.is_empty() || b[0] != 0x00 {
            return Err("cannot mix simple and nested shares".to_string());
        }
    }

    let parsed_shares: Result<Vec<ParsedShare>, String> = shares
        .iter()
        .map(|s| {
            let bytes = URL_SAFE.decode(s.as_bytes()).map_err(|e| e.to_string())?;
            decode_share_from_bytes(&bytes)
        })
        .collect();
    let parsed_shares = parsed_shares?;

    let depth = parsed_shares[0].thresholds.len();
    if depth < 1 {
        return Err("invalid share depth".to_string());
    }

    for ps in &parsed_shares {
        if ps.thresholds.len() != depth {
            return Err("shares have inconsistent depth".to_string());
        }
    }

    let mut current: Vec<ShareEntry> = parsed_shares
        .iter()
        .map(|ps| {
            (
                ps.group_indices.clone(),
                ps.raw_bytes.clone(),
                ps.thresholds.clone(),
                ps.leaf_threshold,
            )
        })
        .collect();

    for step in 0..=depth {
        let mut groups: GroupedShares = HashMap::new();
        for (path, bytes, thresholds, threshold) in &current {
            groups.entry(path.clone()).or_default().push((
                bytes.clone(),
                thresholds.clone(),
                *threshold,
            ));
        }

        let mut next: Vec<ShareEntry> = Vec::new();

        for (path, group_shares) in &groups {
            let threshold = group_shares[0].2;
            if group_shares.len() >= threshold as usize {
                let blahaj_shares: Result<Vec<Share>, String> = group_shares
                    .iter()
                    .map(|(b, _, _)| Share::try_from(b.as_slice()).map_err(|e| e.to_string()))
                    .collect();
                let blahaj_shares = blahaj_shares?;

                let n = u8::try_from(group_shares.len())
                    .map_err(|_| "too many shares in group".to_string())?;
                let sharks = Sharks(n);
                let recovered = sharks
                    .recover(blahaj_shares.as_slice())
                    .map_err(|e| e.to_string())?;

                let parent_path = if path.is_empty() {
                    vec![]
                } else {
                    path[..path.len() - 1].to_vec()
                };
                let thresholds = group_shares[0].1.clone();
                let next_threshold = if step < depth {
                    thresholds[depth - 1 - step]
                } else {
                    0
                };
                next.push((parent_path, recovered, thresholds, next_threshold));
            }
        }

        current = next;

        if current.is_empty() {
            return Err("not enough shares to recover".to_string());
        }
    }

    let secret_bytes = &current[0].1;
    let secret = str::from_utf8(secret_bytes).map_err(|e| e.to_string())?;
    Ok(secret.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            simple_split,
            simple_combine,
            nested_split,
            nested_combine
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distr::{Alphanumeric, SampleString};
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::collections::BTreeMap;

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
        let shares = [
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
        let mut rng = StdRng::seed_from_u64(42);

        // Test with all combinations of threshold and shares
        for threshold in 1..=255 {
            // 80% chance of skipping this loop
            if rng.random::<u8>() < 204 {
                continue;
            }

            for shares in threshold..=255 {
                // 80% chance of skipping this loop
                if rng.random::<u8>() < 204 {
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
        let mut rng = StdRng::seed_from_u64(42);
        for _ in 0..=100 {
            let secret_len = rng.random_range(0..=512);
            let secret = Alphanumeric.sample_string(&mut rng, secret_len);
            let threshold = rng.random_range(1..=255);
            let shares = rng.random_range(threshold..=255);

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

    #[test]
    fn test_nested_split_simple() {
        let secret = "Hello World!";
        let groups = vec![
            GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            },
            GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            },
        ];

        let result = nested_split(secret, 1, groups);
        assert!(result.is_ok());
        let shares = result.unwrap();
        assert_eq!(shares.len(), 10);

        let combine = nested_combine(shares);
        assert!(combine.is_ok());
        assert_eq!(combine.unwrap(), secret);
    }

    #[test]
    fn test_nested_split_deep() {
        let secret = "Deep nested secret";
        let sub_groups: Vec<GroupConfig> = (0..4)
            .map(|_| GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            })
            .collect();
        let groups: Vec<GroupConfig> = (0..3)
            .map(|_| GroupConfig {
                threshold: 2,
                count: 4,
                groups: sub_groups.clone(),
            })
            .collect();

        let result = nested_split(secret, 2, groups);
        assert!(result.is_ok());
        let shares = result.unwrap();
        assert_eq!(shares.len(), 60);

        let combine = nested_combine(shares.clone());
        assert!(combine.is_ok());
        assert_eq!(combine.unwrap(), secret);

        let parsed: Vec<ParsedShare> = shares
            .iter()
            .map(|s| {
                let bytes = URL_SAFE.decode(s.as_bytes()).unwrap();
                decode_share_from_bytes(&bytes).unwrap()
            })
            .collect();

        let mut min_shares = Vec::new();
        let mut top_groups: BTreeMap<u8, Vec<&ParsedShare>> = BTreeMap::new();
        for ps in &parsed {
            top_groups.entry(ps.group_indices[0]).or_default().push(ps);
        }
        for (_, top_group_shares) in top_groups.iter().take(2) {
            let mut sub_groups_map: BTreeMap<u8, Vec<&ParsedShare>> = BTreeMap::new();
            for ps in top_group_shares {
                sub_groups_map
                    .entry(ps.group_indices[1])
                    .or_default()
                    .push(ps);
            }
            for (_, sub_group_shares) in sub_groups_map.iter().take(2) {
                for ps in sub_group_shares.iter().take(3) {
                    min_shares.push(encode_share(ps));
                }
            }
        }

        let min_combine = nested_combine(min_shares);
        assert!(min_combine.is_ok());
        assert_eq!(min_combine.unwrap(), secret);
    }

    #[test]
    fn test_nested_combine_partial() {
        let secret = "Partial test";

        let groups = vec![
            GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            },
            GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            },
        ];
        let shares = nested_split(secret, 2, groups).unwrap();

        let parsed: Vec<ParsedShare> = shares
            .iter()
            .map(|s| {
                let bytes = URL_SAFE.decode(s.as_bytes()).unwrap();
                decode_share_from_bytes(&bytes).unwrap()
            })
            .collect();

        let first_group = parsed[0].group_indices[0];
        let group1_shares: Vec<String> = parsed
            .iter()
            .filter(|ps| ps.group_indices[0] == first_group)
            .map(encode_share)
            .collect();

        let result = nested_combine(group1_shares);
        assert!(result.is_err());

        let groups2 = vec![GroupConfig {
            threshold: 3,
            count: 5,
            groups: vec![],
        }];
        let shares2 = nested_split(secret, 1, groups2).unwrap();

        let result2 = nested_combine(shares2[0..2].to_vec());
        assert!(result2.is_err());
    }

    #[test]
    fn test_nested_combine_simple_share() {
        let secret = "Hello World!";
        let shares = simple_split(secret, 2, 4).unwrap();

        let result = nested_combine(shares);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), secret);
    }

    #[test]
    fn test_nested_round_trip() {
        let mut rng = StdRng::seed_from_u64(42);

        for _ in 0..=20 {
            let secret_len = rng.random_range(1..=256);
            let secret = Alphanumeric.sample_string(&mut rng, secret_len);

            let group_count = rng.random_range(1u8..=5u8);
            let groups: Vec<GroupConfig> = (0..group_count)
                .map(|_| {
                    let threshold = rng.random_range(1u8..=5u8);
                    let count = rng.random_range(threshold..=10u8);
                    GroupConfig {
                        threshold,
                        count,
                        groups: vec![],
                    }
                })
                .collect();
            let top_threshold = rng.random_range(1u8..=group_count);

            let result = nested_split(&secret, top_threshold, groups);
            assert!(result.is_ok(), "nested_split failed: {:?}", result.err());
            let shares = result.unwrap();

            let combine = nested_combine(shares);
            assert!(
                combine.is_ok(),
                "nested_combine failed: {:?}",
                combine.err()
            );
            assert_eq!(combine.unwrap(), secret);
        }

        for _ in 0..=10 {
            let secret_len = rng.random_range(1..=128);
            let secret = Alphanumeric.sample_string(&mut rng, secret_len);

            let sub_groups: Vec<GroupConfig> = (0..3)
                .map(|_| {
                    let threshold = rng.random_range(1u8..=4u8);
                    let count = rng.random_range(threshold..=6u8);
                    GroupConfig {
                        threshold,
                        count,
                        groups: vec![],
                    }
                })
                .collect();
            let groups: Vec<GroupConfig> = (0..3)
                .map(|_| GroupConfig {
                    threshold: 2,
                    count: 3,
                    groups: sub_groups.clone(),
                })
                .collect();

            let result = nested_split(&secret, 2, groups);
            assert!(result.is_ok(), "nested_split failed: {:?}", result.err());
            let shares = result.unwrap();

            let combine = nested_combine(shares);
            assert!(
                combine.is_ok(),
                "nested_combine failed: {:?}",
                combine.err()
            );
            assert_eq!(combine.unwrap(), secret);
        }
    }

    #[test]
    fn test_nested_validation() {
        let secret = "test";

        let result = nested_split(
            secret,
            0,
            vec![GroupConfig {
                threshold: 1,
                count: 1,
                groups: vec![],
            }],
        );
        assert!(result.is_err());

        let result = nested_split(secret, 1, vec![]);
        assert!(result.is_err());

        let result = nested_split(
            secret,
            3,
            vec![GroupConfig {
                threshold: 1,
                count: 1,
                groups: vec![],
            }],
        );
        assert!(result.is_err());

        let result = nested_split(
            secret,
            1,
            vec![GroupConfig {
                threshold: 0,
                count: 1,
                groups: vec![],
            }],
        );
        assert!(result.is_err());

        let result = nested_split(
            secret,
            1,
            vec![GroupConfig {
                threshold: 3,
                count: 2,
                groups: vec![],
            }],
        );
        assert!(result.is_err());

        let result = nested_split(
            secret,
            1,
            vec![GroupConfig {
                threshold: 1,
                count: 3,
                groups: vec![GroupConfig {
                    threshold: 1,
                    count: 1,
                    groups: vec![],
                }],
            }],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_nested_different_thresholds() {
        let secret = "Different thresholds";

        let groups = vec![
            GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            },
            GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            },
            GroupConfig {
                threshold: 3,
                count: 5,
                groups: vec![],
            },
        ];

        let result = nested_split(secret, 1, groups);
        assert!(result.is_ok());
        let shares = result.unwrap();
        assert_eq!(shares.len(), 15);

        let combine = nested_combine(shares);
        assert!(combine.is_ok());
        assert_eq!(combine.unwrap(), secret);

        let groups2 = vec![
            GroupConfig {
                threshold: 2,
                count: 2,
                groups: vec![
                    GroupConfig {
                        threshold: 1,
                        count: 2,
                        groups: vec![],
                    },
                    GroupConfig {
                        threshold: 1,
                        count: 3,
                        groups: vec![],
                    },
                ],
            },
            GroupConfig {
                threshold: 2,
                count: 2,
                groups: vec![
                    GroupConfig {
                        threshold: 2,
                        count: 4,
                        groups: vec![],
                    },
                    GroupConfig {
                        threshold: 1,
                        count: 2,
                        groups: vec![],
                    },
                ],
            },
        ];

        let result2 = nested_split(secret, 1, groups2);
        assert!(result2.is_ok());
        let shares2 = result2.unwrap();

        let combine2 = nested_combine(shares2);
        assert!(combine2.is_ok());
        assert_eq!(combine2.unwrap(), secret);
    }

    #[test]
    fn test_nested_mixed_shares() {
        let secret = "Mixed test";

        let nested_shares = nested_split(
            secret,
            1,
            vec![GroupConfig {
                threshold: 2,
                count: 3,
                groups: vec![],
            }],
        )
        .unwrap();

        let simple_shares = simple_split(secret, 2, 3).unwrap();

        let mut mixed = vec![nested_shares[0].clone(), simple_shares[0].clone()];
        let result = nested_combine(mixed.clone());
        assert!(result.is_err());

        mixed = vec![simple_shares[0].clone(), nested_shares[0].clone()];
        let result2 = nested_combine(mixed);
        assert!(result2.is_err());

        let all_nested = nested_combine(nested_shares);
        assert!(all_nested.is_ok());
        assert_eq!(all_nested.unwrap(), secret);

        let all_simple = nested_combine(simple_shares);
        assert!(all_simple.is_ok());
        assert_eq!(all_simple.unwrap(), secret);
    }
}
