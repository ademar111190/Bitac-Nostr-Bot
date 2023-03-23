use nostr_bot::*;

#[derive(Clone)]
struct State {}

const URL: &str = "https://mempool.space/api";

fn extract_btc_address(message: &str) -> Option<String> {
    let re = regex::Regex::new(r"\b(bc(0([ac-hj-np-z02-9]{39}|[ac-hj-np-z02-9]{59})|1[ac-hj-np-z02-9]{8,87})|[13][a-km-zA-HJ-NP-Z1-9]{25,35})\b").unwrap();
    let caps = re.captures(message);
    match caps {
        Some(c) => {
            let address = c.get(1).unwrap().as_str();
            Some(String::from(address))
        }
        None => None,
    }
}

fn format_btc_balance(balance: u64) -> String {
    if balance >= 100_000_000 {
        let btc = balance as f64 / 100_000_000.0;
        format!("₿ {:.8}", btc)
    } else {
        let balance = balance.to_string().chars().rev().collect::<Vec<char>>();
        let mut formatted = String::new();
        let mut count = 0;
        for c in balance {
            formatted.push(c);
            count += 1;
            if count == 3 {
                formatted.push(' ');
                count = 0;
            }
        }
        if count > 0 {
            formatted.push(' ');
        }
        formatted.push('丰');
        return formatted.chars().rev().collect();
    }
}

async fn get_response(url: &str) -> Result<String, reqwest::Error> {
    reqwest::get(url).await?.text().await
}

async fn get_balance(event: Event, _: State) -> EventNonSigned {
    println!("Getting balance with event: {}", event.format());
    let address = extract_btc_address(&event.content).unwrap();
    println!("Getting balance for address: {}", address);
    let url = format!("{}/address/{}", URL, address);
    let response = get_response(&url).await;
    let response = match response {
        Ok(r) => r,
        Err(e) => {
            println!("Error: {}", e);
            return get_reply(
                event,
                String::from("Error getting balance."),
            );
        }
    };
    println!("Response: {:?}", response);
    let json: serde_json::Value = serde_json::from_str(&response).unwrap();
    let funded = json["chain_stats"]["funded_txo_sum"].as_u64().unwrap();
    let spent = json["chain_stats"]["spent_txo_sum"].as_u64().unwrap();
    let balance = funded - spent;
    println!("Balance: {} for address: {}", balance, address);
    let formatted = format_btc_balance(balance);
    return get_reply(
        event,
        formatted,
    );
}

async fn help(event: Event, _: State) -> EventNonSigned {
    get_reply(
        event,
        String::from("Send me a bitcoin address and I'll tell you how much bitcoin it has."),
    )
}

#[tokio::main]
async fn main() {
    init_logger();

    let relays = vec![
        "your relay",
    ];

    let keypair = keypair_from_secret(
        "you nsec",
    );

    Bot::new(keypair, relays, State {})
        .name("Bitac Bot")
        .about("I'm a bot that answers how much bitcoin a given address has.")
        .picture("https://upload.wikimedia.org/wikipedia/commons/1/19/Stingless_Bees_%28Tetragonisca_angustula%29_%286788207763%29.jpg")
        .intro_message("Send me a bitcoin address, and I'll tell you how much it has.")
        .command(Command::new("!help", wrap!(help)).description("Show this help message."))
        .command(Command::new("", wrap!(get_balance)).description("Get the balance of a bitcoin address."))
        .run()
        .await;
}

/// tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_btc_address_1() {
        let message = "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa";
        let address = extract_btc_address(&message);
        assert_eq!(address.unwrap(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
    }

    #[test]
    fn test_extract_btc_address_1_from_message() {
        let message = "Hi!, How much btc 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa address has?";
        let address = extract_btc_address(&message);
        assert_eq!(address.unwrap(), "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
    }

    #[test]
    fn test_extract_btc_address_3() {
        let message = "3E8ociqZa9mZUSwGdSmAEMAoAxBK3FNDcd";
        let address = extract_btc_address(&message);
        assert_eq!(address.unwrap(), "3E8ociqZa9mZUSwGdSmAEMAoAxBK3FNDcd");
    }

    #[test]
    fn test_extract_btc_address_3_from_message() {
        let message = "Hi!, How much btc 3E8ociqZa9mZUSwGdSmAEMAoAxBK3FNDcd address has?";
        let address = extract_btc_address(&message);
        assert_eq!(address.unwrap(), "3E8ociqZa9mZUSwGdSmAEMAoAxBK3FNDcd");
    }

    #[test]
    fn test_extract_btc_address_bc1() {
        let message = "bc1qm9n8x3jge2356hhyywfwrsmfczr49fxz37da8y";
        let address = extract_btc_address(&message);
        assert_eq!(address.unwrap(), "bc1qm9n8x3jge2356hhyywfwrsmfczr49fxz37da8y");
    }

    #[test]
    fn test_extract_btc_address_bc1_from_message() {
        let message = "Hi!, How much btc bc1qm9n8x3jge2356hhyywfwrsmfczr49fxz37da8y address has?";
        let address = extract_btc_address(&message);
        assert_eq!(address.unwrap(), "bc1qm9n8x3jge2356hhyywfwrsmfczr49fxz37da8y");
    }

    #[test]
    fn test_format_btc_balance() {
        let balance = 123_456_789;
        let formatted = format_btc_balance(balance);
        assert_eq!(formatted, "₿ 1.23456789");
    }

    #[test]
    fn test_format_btc_balance_one_btc() {
        let balance = 100_000_000;
        let formatted = format_btc_balance(balance);
        assert_eq!(formatted, "₿ 1.00000000");
    }

    #[test]
    fn test_format_btc_balance_satoshis() {
        let balance = 123_456;
        let formatted = format_btc_balance(balance);
        assert_eq!(formatted, "丰 123 456");
    }

    #[test]
    fn test_format_btc_balance_zero() {
        let balance = 0;
        let formatted = format_btc_balance(balance);
        assert_eq!(formatted, "丰 0");
    }

    #[test]
    fn test_format_btc_huge_balance() {
        let balance = 987_123_456_789;
        let formatted = format_btc_balance(balance);
        assert_eq!(formatted, "₿ 9871.23456789");
    }
}
