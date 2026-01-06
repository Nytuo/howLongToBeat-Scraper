use headless_chrome::{Browser, LaunchOptions};
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use urlencoding::encode;

#[derive(Deserialize, Debug, PartialEq, Serialize, Clone)]
pub struct Styles {
    pub average: Option<f32>,
    pub median: Option<f32>,
    pub rushed: Option<f32>,
    pub leisure: Option<f32>,
}

impl Styles {
    /// Creates a new Styles struct
    ///
    /// # Arguments
    ///
    /// * `average`:  Option<f32> - The average time it takes to complete the game
    /// * `median`:  Option<f32> - The median time it takes to complete the game
    /// * `rushed`:  Option<f32> - The rushed time it takes to complete the game
    /// * `leisure`:  Option<f32> - The leisure time it takes to complete the game
    ///
    /// returns: Styles
    fn new(
        average: Option<f32>,
        median: Option<f32>,
        rushed: Option<f32>,
        leisure: Option<f32>,
    ) -> Styles {
        Styles {
            average,
            median,
            rushed,
            leisure,
        }
    }

    fn empty() -> Styles {
        Styles {
            average: None,
            median: None,
            rushed: None,
            leisure: None,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq, Serialize)]
pub struct Game {
    pub hltb_id: u32,
    pub title: String,
    pub main_story: Option<Styles>,
    pub main_extra: Option<Styles>,
    pub completionist: Option<Styles>,
    pub all_styles: Option<Styles>,
    pub co_op: Option<Styles>,
    pub vs: Option<Styles>,
}

impl Game {
    /// Creates a new Game struct
    ///
    /// # Arguments
    ///
    /// * `title`:  String - The title of the game
    /// * `hltb_id`:  u32 - The ID of the game on How Long to Beat
    /// * `main_story`:  Option<Styles> - The time it takes to complete the main story
    /// * `main_extra`:  Option<Styles> - The time it takes to complete the main story and extras
    /// * `completionist`:  Option<Styles> - The time it takes to complete the game 100%
    /// * `all_styles`:  Option<Styles> - The time it takes to complete the game in all styles
    /// * `co_op`:  Option<Styles> - The time it takes to complete the game in co-op mode
    /// * `vs`:  Option<Styles> - The time it takes to complete the game in competitive mode
    ///
    /// returns: Game
    fn new(
        title: String,
        hltb_id: u32,
        main_story: Option<Styles>,
        main_extra: Option<Styles>,
        completionist: Option<Styles>,
        all_styles: Option<Styles>,
        co_op: Option<Styles>,
        vs: Option<Styles>,
    ) -> Game {
        Game {
            hltb_id,
            title,
            main_story,
            main_extra,
            completionist,
            all_styles,
            co_op,
            vs,
        }
    }
}

const BASE_URL: &str = "https://howlongtobeat.com/";

/// Searches the search page for a game
///
/// # Arguments
///
/// * `name`:  &str - The name of the game to search for
/// * `sandbox`:  bool - Whether to enable sandbox mode for the browser
///
/// returns: Result<u32, Box<dyn Error, Global>>
async fn search_search_page_for_with_sandbox(
    name: &str,
    sandbox: bool,
) -> Result<u32, Box<dyn Error>> {
    let url = BASE_URL.to_owned() + "?q=" + &encode(name);
    let launch_options = LaunchOptions {
        headless: true,
        sandbox,
        ..Default::default()
    };
    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    tab.set_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36", None, None)?;
    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;
    tab.wait_for_element("#search-results-header > ul > li:nth-child(1) > div > div[class*='_search_list_image'] > a")?;

    let content = tab.get_content()?;
    let document = Html::parse_document(&content);
    let selector = Selector::parse("#search-results-header > ul > li:nth-child(1) > div > div[class*='_search_list_image'] > a").unwrap();

    for element in document.select(&selector) {
        if let Some(link) = element.value().attr("href") {
            let id = link.split("/").last().unwrap().parse::<u32>().unwrap();
            return Ok(id);
        }
    }
    Err("Element not found".into())
}

/// Searches the search page for a game (with sandbox enabled by default)
///
/// # Arguments
///
/// * `name`:  &str - The name of the game to search for
///
/// returns: Result<u32, Box<dyn Error, Global>>
async fn search_search_page_for(name: &str) -> Result<u32, Box<dyn Error>> {
    search_search_page_for_with_sandbox(name, true).await
}

/// Searches for the details page of a game
///
/// # Arguments
///
/// * `hltb_id`:  u32 - The ID of the game on How Long to Beat
/// * `sandbox`:  bool - Whether to enable sandbox mode for the browser
///
/// returns: Result<Game, Box<dyn Error, Global>>
async fn search_details_page_for_with_sandbox(
    hltb_id: u32,
    sandbox: bool,
) -> Result<Game, Box<dyn Error>> {
    let url = BASE_URL.to_owned() + "game/" + hltb_id.to_string().as_str();
    let launch_options = LaunchOptions {
        headless: true,
        sandbox,
        ..Default::default()
    };
    let browser = Browser::new(launch_options)?;
    let tab = browser.new_tab()?;
    tab.set_user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36", None, None)?;
    tab.navigate_to(&url)?;
    tab.wait_until_navigated()?;

    tab.wait_for_element("#__next > div > main > div:nth-child(2) > div > div[class*='content'] > div.in.scrollable.scroll_blue.shadow_box.back_primary > table[class*='_game_main_table']")?;

    let content = tab.get_content()?;
    let document = Html::parse_document(&content);
    let title_selector = Selector::parse(
        "#__next > div > main > div:nth-child(1) > div > div > div > div[class*='_profile_header']",
    )
    .unwrap();
    let title = document
        .select(&title_selector)
        .next()
        .unwrap()
        .inner_html()
        .trim()
        .to_string()
        .replace("<!-- -->", "");
    let table_selector = Selector::parse("#__next > div > main > div:nth-child(2) > div > div[class*='content'] > div.in.scrollable.scroll_blue.shadow_box.back_primary > table[class*='_game_main_table']").unwrap();
    let table = document.select(&table_selector).next().unwrap();
    let tr_selector = Selector::parse("tbody > tr").unwrap();
    let rows: Vec<_> = table.select(&tr_selector).collect();

    let mut main_story = None;
    let mut main_extra = None;
    let mut completionist = None;
    let mut all_styles = None;
    let mut co_op = None;
    let mut vs = None;

    let td_selector = Selector::parse("td").unwrap();
    for row in rows {
        if let Some(first_cell) = row.select(&td_selector).next() {
            let row_type = first_cell.inner_html().trim().to_string();
            match row_type.as_str() {
                "Main Story" => main_story = Some(parse_row(row)),
                "Main + Extra" | "Main + Extras" => main_extra = Some(parse_row(row)),
                "Completionist" | "Completionists" => completionist = Some(parse_row(row)),
                "All PlayStyles" => all_styles = Some(parse_row(row)),
                "Co-Op" => co_op = Some(parse_row(row)),
                "Competitive" => vs = Some(parse_row(row)),
                _ => {}
            }
        }
    }

    Ok(Game::new(
        title,
        hltb_id,
        main_story,
        main_extra,
        completionist,
        all_styles,
        co_op,
        vs,
    ))
}

/// Searches for the details page of a game (with sandbox enabled by default)
///
/// # Arguments
///
/// * `hltb_id`:  u32 - The ID of the game on How Long to Beat
///
/// returns: Result<Game, Box<dyn Error, Global>>
async fn search_details_page_for(hltb_id: u32) -> Result<Game, Box<dyn Error>> {
    search_details_page_for_with_sandbox(hltb_id, true).await
}

/// Parses a row of a table
///
/// # Arguments
///
/// * `row`:  ElementRef - The row to parse
///
/// returns: Styles
fn parse_row(row: ElementRef) -> Styles {
    let selector = Selector::parse("td").unwrap();
    let mut cells = row.select(&selector);
    cells.next();
    cells.next();
    let average = convert_hours_minutes_to_sec_opt(cells.next().unwrap().inner_html().as_str());
    let median = convert_hours_minutes_to_sec_opt(cells.next().unwrap().inner_html().as_str());
    let rushed = convert_hours_minutes_to_sec_opt(cells.next().unwrap().inner_html().as_str());
    let leisure = convert_hours_minutes_to_sec_opt(cells.next().unwrap().inner_html().as_str());
    Styles::new(average, median, rushed, leisure)
}

/// Converts a string of hours and minutes to seconds, returning None for empty/invalid values
///
/// # Arguments
///
/// * `text`:  &str - The text to convert to seconds (e.g. "26h 21m", "83 Hours", "59½ Hours")
///
/// returns: Option<f32>
fn convert_hours_minutes_to_sec_opt(text: &str) -> Option<f32> {
    let text = text.trim();

    if text.is_empty() || text == "--" || text == "-" {
        return None;
    }

    if text.contains("Hours") || text.contains("Hour") {
        let parts: Vec<&str> = text.split_whitespace().collect();
        if let Some(time_str) = parts.first() {
            let time_str = time_str
                .replace("½", ".5")
                .replace("¼", ".25")
                .replace("¾", ".75");

            if let Ok(hours) = time_str.parse::<f32>() {
                return Some(hours * 3600.0);
            }
        }
        return None;
    }

    let parts = text.split_whitespace();
    let mut total = 0.0;
    for part in parts {
        if part.contains('h') {
            if let Ok(hours) = part.replace('h', "").parse::<f32>() {
                total += hours * 3600.0;
            }
        } else if part.contains('m') {
            if let Ok(minutes) = part.replace('m', "").parse::<f32>() {
                total += minutes * 60.0;
            }
        }
    }

    if total > 0.0 {
        Some(total)
    } else {
        None
    }
}

/// Converts a string of hours and minutes to seconds
///
/// # Arguments
///
/// * `text`:  &str - The text to convert to seconds (e.g. "26h 21m")
///
/// returns: f32
fn convert_hours_minutes_to_sec(text: &str) -> f32 {
    convert_hours_minutes_to_sec_opt(text).unwrap_or(0.0)
}

/// Searches for a game by name
///
/// # Arguments
///
/// * `name`:  &str - The name of the game to search for
///
/// returns: Result<String, Box<dyn Error, Global>>
pub async fn search_by_name(name: &str) -> Result<Game, Box<dyn Error>> {
    search_by_name_with_sandbox(name, true).await
}

/// Searches for a game by name with custom sandbox setting
///
/// # Arguments
///
/// * `name`:  &str - The name of the game to search for
/// * `sandbox`:  bool - Whether to enable sandbox mode for the browser (set to false for Docker/CI environments)
///
/// returns: Result<String, Box<dyn Error, Global>>
pub async fn search_by_name_with_sandbox(
    name: &str,
    sandbox: bool,
) -> Result<Game, Box<dyn Error>> {
    let hltb_id = search_search_page_for_with_sandbox(name, sandbox)
        .await
        .unwrap();
    let game = search_details_page_for_with_sandbox(hltb_id, sandbox)
        .await
        .unwrap();
    Ok(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_search_search_page_for() {
        assert_eq!(search_search_page_for("Metal Gear").await.unwrap(), 5900);
    }

    #[tokio::test]
    async fn test_search_details_page_for() {
        let game = search_details_page_for(5900).await.unwrap();
        assert_eq!(game.hltb_id, 5900);
        assert_eq!(
            game.main_story,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("4h 10m")),
                Some(convert_hours_minutes_to_sec("4h")),
                Some(convert_hours_minutes_to_sec("2h 46m")),
                Some(convert_hours_minutes_to_sec("7h 12m"))
            ))
        );
        assert_eq!(
            game.main_extra,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("4h 54m")),
                Some(convert_hours_minutes_to_sec("4h 51m")),
                Some(convert_hours_minutes_to_sec("3h 24m")),
                Some(convert_hours_minutes_to_sec("7h 29m"))
            ))
        );
        assert_eq!(
            game.completionist,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("5h 41m")),
                Some(convert_hours_minutes_to_sec("5h")),
                Some(convert_hours_minutes_to_sec("3h 58m")),
                Some(convert_hours_minutes_to_sec("14h 52m"))
            ))
        );
        assert_eq!(
            game.all_styles,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("4h 34m")),
                Some(convert_hours_minutes_to_sec("4h")),
                Some(convert_hours_minutes_to_sec("2h 52m")),
                Some(convert_hours_minutes_to_sec("14h 20m"))
            ))
        );
        assert_eq!(game.co_op, None);
        assert_eq!(game.vs, None);
        assert_eq!(game.title, "Metal Gear");
    }

    #[tokio::test]
    async fn test_search_by_name() {
        let game = search_by_name("Metal Gear").await.unwrap();
        let expected = Game::new(
            "Metal Gear".to_string(),
            5900,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("4h 10m")),
                Some(convert_hours_minutes_to_sec("4h")),
                Some(convert_hours_minutes_to_sec("2h 46m")),
                Some(convert_hours_minutes_to_sec("7h 12m")),
            )),
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("4h 54m")),
                Some(convert_hours_minutes_to_sec("4h 51m")),
                Some(convert_hours_minutes_to_sec("3h 24m")),
                Some(convert_hours_minutes_to_sec("7h 29m")),
            )),
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("5h 41m")),
                Some(convert_hours_minutes_to_sec("5h")),
                Some(convert_hours_minutes_to_sec("3h 58m")),
                Some(convert_hours_minutes_to_sec("14h 52m")),
            )),
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("4h 34m")),
                Some(convert_hours_minutes_to_sec("4h")),
                Some(convert_hours_minutes_to_sec("2h 52m")),
                Some(convert_hours_minutes_to_sec("14h 20m")),
            )),
            None,
            None,
        );
        assert_eq!(game, expected);
    }

    #[tokio::test]
    async fn test_search_details_page_for_coopvs() {
        let game = search_details_page_for(129232).await.unwrap();
        assert_eq!(game.hltb_id, 129232);
        assert_eq!(game.main_story, None);
        assert_eq!(game.main_extra, None);
        assert_eq!(game.completionist, None);
        assert_eq!(game.all_styles, None);
        assert_eq!(
            game.co_op,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("83 Hours")),
                Some(convert_hours_minutes_to_sec("59½ Hours")),
                Some(convert_hours_minutes_to_sec("38½ Hours")),
                Some(convert_hours_minutes_to_sec("205 Hours"))
            ))
        );
        assert_eq!(
            game.vs,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("31 Hours")),
                Some(convert_hours_minutes_to_sec("31 Hours")),
                Some(convert_hours_minutes_to_sec("19 Hours")),
                Some(convert_hours_minutes_to_sec("43 Hours"))
            ))
        );
        assert_eq!(game.title, "Helldivers 2");
    }

    #[tokio::test]
    async fn test_search_by_name_coopvs() {
        let game = search_by_name("Helldivers 2").await.unwrap();
        let expected = Game::new(
            "Helldivers 2".to_string(),
            129232,
            None,
            None,
            None,
            None,
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("83 Hours")),
                Some(convert_hours_minutes_to_sec("59½ Hours")),
                Some(convert_hours_minutes_to_sec("38½ Hours")),
                Some(convert_hours_minutes_to_sec("205 Hours")),
            )),
            Some(Styles::new(
                Some(convert_hours_minutes_to_sec("31 Hours")),
                Some(convert_hours_minutes_to_sec("31 Hours")),
                Some(convert_hours_minutes_to_sec("19 Hours")),
                Some(convert_hours_minutes_to_sec("43 Hours")),
            )),
        );
        assert_eq!(game, expected);
    }

    #[tokio::test]
    async fn test_search_by_name_with_sandbox_disabled() {
        let game = search_by_name_with_sandbox("Metal Gear", false)
            .await
            .unwrap();
        assert_eq!(game.hltb_id, 5900);
        assert_eq!(game.title, "Metal Gear");
    }
}
