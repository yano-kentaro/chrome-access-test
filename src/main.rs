extern crate serde_derive;
extern crate toml;

use anyhow::Result;
use curl::easy::{Easy, List};
use headless_chrome::{protocol::cdp::Network::CookieParam, Browser, LaunchOptions};
use rayon::prelude::*;
use serde_derive::Deserialize;
use std::{fs, path::PathBuf};

mod error;

/// Struct of toml file ( 2023/01/01 : 1 ) [ Kentaro Yano ]
/// # Note
/// * The toml file must be in the following format.
/// ```
/// access_url = "https://example.com"
/// find_selector = "#selector"
/// [cookie]
/// name = "cookie_name"
/// value = "cookie_value"
/// ```
/// * The cookie section is optional.
#[derive(Deserialize)]
struct AccessConf {
    #[serde(default)]
    access_url: String,
    #[serde(default)]
    find_selector: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    cookie: Option<CookieParam>,
}

/// Struct of toml file ( 2023/01/01 : 1 ) [ Kentaro Yano ]
/// # Note
/// * The toml file must be in the following format.
/// ```
/// webhook_url = "https://chat.googleapis.com/v1/spaces/XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
///              /messages?key=XXXXXXXXXXXXXXXXXXXXXXXX"
/// alert_message = " is not working."
/// ```
#[derive(Deserialize)]
struct GoogleChatConf {
    webhook_url: String,
}

/// EntryPoint ( 2023/01/01 : 1 ) [ Kentaro Yano ]
fn main() {
    let target_dir = create_path(vec!["conf", "service"]);
    let paths = fs::read_dir(target_dir).unwrap();
    paths.par_bridge().for_each(|path| {
        let path = path.unwrap().path();
        let conf = parse_toml(path);
        let result = access_test(&conf);
        match result {
            Ok(_) => (),
            Err(e) => notify_google_chat(&conf.access_url, e),
        }
    });
}

/// Create path from manifest_dir and Arguments ( 2023/01/08 : 1 ) [ Kentaro Yano ]
/// # Arguments
/// * `dirs` - Vector of directory names
/// # Returns
/// * `PathBuf` - PathBuf of path
/// # Examples
/// ```
/// let path = create_path(vec!["conf", "service", "sample.toml"]);
/// ```
fn create_path(dirs: Vec<&str>) -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut path = PathBuf::from(manifest_dir);
    for dir in dirs {
        if dir.contains(".") {
            let filename_and_ext: Vec<&str> = dir.split(".").collect();
            path.push(filename_and_ext[0]);
            path.set_extension(filename_and_ext[1]);
        } else {
            path.push(dir);
        }
    }
    path
}

/// Parse Toml file to Struct ( 2023/01/01 : 1 ) [ Kentaro Yano ]
/// # Arguments
/// * `path` - Path to toml file
/// # Returns
/// * `AccessConf` - Struct of toml file
/// # Examples
/// ```
/// let conf = parse_toml("./conf/service/sample.toml");
/// ```
fn parse_toml(path: PathBuf) -> AccessConf {
    let toml_str = fs::read_to_string(path).unwrap();
    let toml_struct: AccessConf = toml::from_str(&toml_str).unwrap();
    toml_struct
}

/// Test access and notify if it fails ( 2023/01/01 : 1 ) [ Kentaro Yano ]
/// # Arguments
/// * `conf` - Struct of toml file
/// # Returns
/// * `Result<()>` - Result of access test
/// # Examples
/// ```
/// let conf = parse_toml("./conf/service/sample.toml");
/// access_test(&conf).unwrap();
/// ```
fn access_test(conf: &AccessConf) -> Result<(), error::CustomError> {
    // Set headless chrome option.
    let option = LaunchOptions {
        // If you want to see what's going on, comment in the line below.
        // headless: false,
        ..Default::default()
    };

    // Launch headless chrome.
    let browser = Browser::new(option).unwrap();
    let tab = browser.wait_for_initial_tab().unwrap();

    // Navigate to access_url.
    tab.navigate_to(&conf.access_url).unwrap();
    let result = tab.wait_until_navigated();
    match result {
        Ok(_) => (),
        Err(_) => return Err(error::CustomError::AccessUrlError),
    };

    // Set cookie if one is defined.
    if let Some(cookie) = &conf.cookie {
        tab.set_cookies(vec![cookie.clone()]).unwrap();
        tab.reload(false, None).unwrap();
        tab.wait_until_navigated().unwrap();

        // Verify the presence of cookie for authentication.
        let cookies = tab.get_cookies().unwrap();
        let cookie = cookies
            .iter()
            .find(|c| c.name == cookie.name && c.value == cookie.value);
        match cookie {
            Some(_) => (),
            None => return Err(error::CustomError::CookieError),
        };
    }

    // Verify the presence of selector for access test.
    let selector = tab.wait_for_element(&conf.find_selector);
    match selector {
        Ok(selector) => selector,
        Err(_) => return Err(error::CustomError::FindSelectorError),
    };

    Ok(())
}

/// Notify to Google Chat ( 2023/01/01 : 1 ) [ Kentaro Yano ]
/// # Arguments
/// * `url` - URL of service
/// # Examples
/// ```
/// notify_google_chat("https://example.com");
/// ```
fn notify_google_chat(url: &str, error: error::CustomError) {
    // Retrieve settings from toml file
    let conf_file = create_path(vec!["conf", "webhook", "google_chat.toml"]);
    let toml_str = fs::read_to_string(conf_file).unwrap();
    let conf: GoogleChatConf = toml::from_str(&toml_str).unwrap();

    let mut handle = Easy::new();
    handle.url(&conf.webhook_url).unwrap();
    handle.post(true).unwrap();

    let mut list = List::new();
    list.append("Content-Type: application/json").unwrap();
    handle.http_headers(list).unwrap();

    let message = format!("{{\"text\": \"{}\n{}\"}}", url, error);
    let data = message.as_bytes();
    handle.post_fields_copy(data).unwrap();

    handle.perform().unwrap();
}

/// Test ( 2023/01/01 : 1 ) [ Kentaro Yano ]
#[cfg(test)]
mod tests {
    use super::*;

    /// Test create_path ( 2023/01/08 : 1 ) [ Kentaro Yano ]
    #[test]
    fn test_create_path() {
        let path = create_path(vec!["conf", "test", "service", "sample.toml"]);
        let current_dir = std::env::current_dir().unwrap();
        assert_eq!(
            path.to_str().unwrap(),
            format!(
                "{}/conf/test/service/sample.toml",
                current_dir.to_str().unwrap()
            )
        );
    }

    /// Test parse_toml ( 2023/01/08 : 1 ) [ Kentaro Yano ]
    #[test]
    fn test_parse_toml() {
        // Success case (with cookie)
        {
            let path = create_path(vec!["conf", "test", "service", "success-def_cookie.toml"]);
            let conf = parse_toml(path);
            assert_eq!(conf.access_url, "https://test.demo.com/");
            assert_eq!(conf.find_selector, "test_selector");
            if let Some(cookie) = &conf.cookie {
                assert_eq!(cookie.name, "cookie_name");
                assert_eq!(cookie.value, "cookie_value");
            }
        }

        // Success case (without cookie)
        {
            let path = create_path(vec!["conf", "test", "service", "success-no_cookie.toml"]);
            let conf = parse_toml(path);
            assert_eq!(conf.access_url, "https://test.demo.com/");
            assert_eq!(conf.find_selector, "test_selector");
            assert_eq!(conf.cookie, None);
        }
    }
}
