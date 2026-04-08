use chrono::{Datelike, Duration, FixedOffset, NaiveDate, Utc};
use std::env;
use std::f64::consts::PI;
use std::fs;
use std::path::PathBuf;

#[macro_use]
extern crate astro;
use astro::{coords, ecliptic, lunar, nutation, sun, time as atime, transit};
use astro::time::{CalType, Date};

/// 朔望月（日）
const SYNODIC_MONTH: f64 = 29.530588853;
const ASTRONOMICAL_UNIT_KM: f64 = 149_597_870.7;
const DEFAULT_LAT: f64 = 35.6762;
const DEFAULT_LON: f64 = 139.6503;
const JST_OFFSET_HOURS: f64 = 9.0;
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Clone, Copy, Debug, PartialEq)]
enum Language {
    Ja,
    En,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AppConfig {
    lat: f64,
    lon: f64,
    art: bool,
    lang: Language,
}

fn date_to_jd(year: i32, month: u32, day: u32) -> f64 {
    // グレゴリオ暦 → ユリウス日（0:00 UTC 基準）
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y as f64 + 4716.0)).floor()
        + (30.6001 * (m as f64 + 1.0)).floor()
        + day as f64
        + b
        - 1524.5
}

fn local_midnight_jd(year: i32, month: u32, day: u32, tz_offset_hours: f64) -> f64 {
    date_to_jd(year, month, day) - tz_offset_hours / 24.0
}

fn parse_language(value: &str) -> Result<Language, String> {
    match value {
        "ja" => Ok(Language::Ja),
        "en" => Ok(Language::En),
        _ => Err("language must be `ja` or `en`".to_string()),
    }
}

fn print_usage(exit_code: i32) -> ! {
    println!("moon {}", VERSION);
    println!();
    println!("Usage:");
    println!("  moon [YYYY-MM-DD|today|prev|next] [LAT LON]");
    println!("  moon --date [YYYY-MM-DD|today|prev|next] --lat LAT --lon LON [--art|--no-art] [--lang ja|en]");
    println!("  moon --help");
    println!("  moon --version");
    if let Some(path) = default_config_path() {
        println!();
        println!("Config file:");
        println!("  {}", path.display());
        println!("  lat = 35.6762");
        println!("  lon = 139.6503");
        println!("  art = true");
        println!("  lang = \"ja\"");
    }
    std::process::exit(exit_code);
}

fn print_version_and_exit() -> ! {
    println!("moon {}", VERSION);
    std::process::exit(0);
}

fn resolve_date_spec(base_date: NaiveDate, spec: &str) -> Result<NaiveDate, String> {
    match spec {
        "today" => Ok(base_date),
        "prev" | "yesterday" => Ok(base_date - Duration::days(1)),
        "next" | "tomorrow" => Ok(base_date + Duration::days(1)),
        _ => NaiveDate::parse_from_str(spec, "%Y-%m-%d")
            .map_err(|_| "invalid date format".to_string()),
    }
}

fn default_config_path() -> Option<PathBuf> {
    if cfg!(target_os = "windows") {
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .map(|base| base.join("moon").join("config.toml"))
    } else if cfg!(target_os = "macos") {
        env::var_os("HOME")
            .map(PathBuf::from)
            .map(|base| base.join("Library").join("Application Support").join("moon").join("config.toml"))
    } else {
        if let Some(xdg) = env::var_os("XDG_CONFIG_HOME") {
            Some(PathBuf::from(xdg).join("moon").join("config.toml"))
        } else {
            env::var_os("HOME")
                .map(PathBuf::from)
                .map(|base| base.join(".config").join("moon").join("config.toml"))
        }
    }
}

fn parse_app_config(content: &str) -> Result<AppConfig, String> {
    let mut lat = None;
    let mut lon = None;
    let mut art = None;
    let mut lang = None;

    for raw_line in content.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = value.trim().trim_matches('"');

        match key {
            "lat" => {
                lat = Some(
                    value
                        .parse()
                        .map_err(|_| "invalid `lat` in config.toml".to_string())?,
                );
            }
            "lon" => {
                lon = Some(
                    value
                        .parse()
                        .map_err(|_| "invalid `lon` in config.toml".to_string())?,
                );
            }
            "art" => {
                art = Some(match value {
                    "true" => true,
                    "false" => false,
                    _ => return Err("`art` in config.toml must be true or false".to_string()),
                });
            }
            "lang" => {
                lang = Some(parse_language(value)?);
            }
            _ => {}
        }
    }

    match (lat, lon) {
        (Some(lat), Some(lon)) => Ok(AppConfig {
            lat,
            lon,
            art: art.unwrap_or(true),
            lang: lang.unwrap_or(Language::Ja),
        }),
        (None, None) => Err("config.toml is missing `lat` and `lon`".to_string()),
        (None, Some(_)) => Err("config.toml is missing `lat`".to_string()),
        (Some(_), None) => Err("config.toml is missing `lon`".to_string()),
    }
}

fn load_app_config() -> Result<Option<AppConfig>, String> {
    let Some(path) = default_config_path() else {
        return Ok(None);
    };

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| format!("failed to read config file ({}): {}", path.display(), e))?;
    let config = parse_app_config(&content)?;
    Ok(Some(config))
}

fn parse_cli_args(
    args: &[String],
    base_date: NaiveDate,
    default_config: AppConfig,
) -> Result<(NaiveDate, f64, f64, bool, Language), String> {
    let mut date = base_date;
    let mut lat = default_config.lat;
    let mut lon = default_config.lon;
    let mut art = default_config.art;
    let mut lang = default_config.lang;

    let mut positional = Vec::new();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--date" => {
                i += 1;
                if i >= args.len() {
                    return Err("missing value for `--date`".to_string());
                }
                date = resolve_date_spec(base_date, &args[i])?;
            }
            "--lat" => {
                i += 1;
                if i >= args.len() {
                    return Err("missing value for `--lat`".to_string());
                }
                lat = args[i]
                    .parse()
                    .map_err(|_| "invalid latitude".to_string())?;
            }
            "--lon" => {
                i += 1;
                if i >= args.len() {
                    return Err("missing value for `--lon`".to_string());
                }
                lon = args[i]
                    .parse()
                    .map_err(|_| "invalid longitude".to_string())?;
            }
            "--lang" => {
                i += 1;
                if i >= args.len() {
                    return Err("missing value for `--lang`".to_string());
                }
                lang = parse_language(&args[i])?;
            }
            "--art" => {
                art = true;
            }
            "--no-art" => {
                art = false;
            }
            "--help" | "-h" => print_usage(0),
            "--version" | "-V" => print_version_and_exit(),
            arg if arg.starts_with("--") => {
                return Err(format!("unknown option: {}", arg));
            }
            _ => positional.push(args[i].as_str()),
        }
        i += 1;
    }

    match positional.as_slice() {
        [] => {}
        [date_spec] => {
            date = resolve_date_spec(base_date, date_spec)?;
        }
        [date_spec, lat_s, lon_s] => {
            date = resolve_date_spec(base_date, date_spec)?;
            lat = lat_s
                .parse()
                .map_err(|_| "invalid latitude".to_string())?;
            lon = lon_s
                .parse()
                .map_err(|_| "invalid longitude".to_string())?;
        }
        _ => return Err("invalid number of arguments".to_string()),
    }

    Ok((date, lat, lon, art, lang))
}

/// スーパームーン判定
/// 満月（月齢14〜16付近）かつ近地点付近（距離の下位10%以内）
fn is_supermoon(age: f64, distance: f64) -> bool {
    let is_full = (age - SYNODIC_MONTH / 2.0).abs() < 1.5;
    let perigee = 356_400.0;
    let apogee = 406_700.0;
    let threshold = perigee + (apogee - perigee) * 0.10;
    is_full && distance < threshold
}

/// astro クレートを使って月の赤道座標と距離を取得
/// 戻り値: (EqPoint { asc, dec }, distance_km)
fn lunar_eq_pos(jd: f64) -> (coords::EqPoint, f64) {
    let (ecl_point, dist) = lunar::geocent_ecl_pos(jd);
    let (nut_in_long, nut_in_oblq) = nutation::nutation(jd);
    let oblq = ecliptic::mn_oblq_laskar(jd) + nut_in_oblq;

    let asc = coords::asc_frm_ecl(ecl_point.long + nut_in_long, ecl_point.lat, oblq);
    let dec = coords::dec_frm_ecl(ecl_point.long + nut_in_long, ecl_point.lat, oblq);

    (coords::EqPoint { asc, dec }, dist)
}

/// astro クレートを使って太陽の赤道座標を取得
fn solar_eq_pos(jd: f64) -> coords::EqPoint {
    let (ecl_point, _) = sun::geocent_ecl_pos(jd);
    let (nut_in_long, nut_in_oblq) = nutation::nutation(jd);
    let oblq = ecliptic::mn_oblq_laskar(jd) + nut_in_oblq;

    let asc = coords::asc_frm_ecl(ecl_point.long + nut_in_long, ecl_point.lat, oblq);
    let dec = coords::dec_frm_ecl(ecl_point.long + nut_in_long, ecl_point.lat, oblq);

    coords::EqPoint { asc, dec }
}

/// `astro` クレート由来の月齢・照明率・距離を返す
fn lunar_metrics(jd: f64, year: i32, month: u32, day: u32) -> (f64, f64, f64) {
    let date = Date {
        year: year as i16,
        month: month as u8,
        decimal_day: day as f64,
        cal_type: CalType::Gregorian,
    };
    let prev_probe_date = NaiveDate::from_ymd_opt(year, month, day)
        .expect("valid date")
        + Duration::days(-15);
    let prev_date = Date {
        year: prev_probe_date.year() as i16,
        month: prev_probe_date.month() as u8,
        decimal_day: prev_probe_date.day() as f64,
        cal_type: CalType::Gregorian,
    };

    let nearest_new_moon = lunar::time_of_phase(&date, &lunar::Phase::New);
    let fallback_prev_new_moon = lunar::time_of_phase(&prev_date, &lunar::Phase::New);
    let prev_new_moon = if nearest_new_moon <= jd {
        nearest_new_moon
    } else {
        fallback_prev_new_moon
    };
    let age = (jd - prev_new_moon).rem_euclid(SYNODIC_MONTH);

    let (moon_ecl, earth_moon_dist) = lunar::geocent_ecl_pos(jd);
    let (sun_ecl, earth_sun_dist_au) = sun::geocent_ecl_pos(jd);
    let illum = lunar::illum_frac_frm_ecl_coords(
        moon_ecl.long,
        moon_ecl.lat,
        sun_ecl.long,
        earth_moon_dist,
        earth_sun_dist_au * ASTRONOMICAL_UNIT_KM,
    );

    (age, illum, earth_moon_dist)
}

/// astro クレートの transit::time を使って月の出・月の入りを計算
/// 戻り値: (月の出 Option<(h,m)>, 月の入り Option<(h,m)>) UTC
fn moonrise_moonset_utc(jd_utc_midnight: f64, lat_deg: f64, lon_deg: f64, year: i32, month: u32) -> (Option<(u32, u32)>, Option<(u32, u32)>) {
    // `date_to_jd` は対象日の 0:00 UTC を返すので、その JD をそのまま基準日に使う。
    // ここでさらに -0.5 すると前日 12:00 UTC 基準となり、月の出入りが大きくずれる。
    let jd0 = jd_utc_midnight;

    let geograph = coords::GeographPoint {
        long: (-lon_deg).to_radians(), // Meeus: 西経が正
        lat: lat_deg.to_radians(),
    };

    // 当日・前日・翌日の月の位置（0h UT 基準）
    let (eq1, _) = lunar_eq_pos(jd0 - 1.0);
    let (eq2, dist2) = lunar_eq_pos(jd0);
    let (eq3, _) = lunar_eq_pos(jd0 + 1.0);

    let apprnt_sidr = apprnt_sidr!(jd0);
    let delta_t = atime::delta_t(year, month as u8);
    let parallax = lunar::eq_hz_parllx(dist2);

    let to_hm = |result: (i64, i64, f64)| -> Option<(u32, u32)> {
        let (h, m, s) = result;
        let total_seconds = h as f64 * 3600.0 + m as f64 * 60.0 + s;
        let total_seconds = total_seconds.rem_euclid(86_400.0);
        let total_minutes = (total_seconds / 60.0).floor() as u32;
        Some((total_minutes / 60, total_minutes % 60))
    };

    let rise = to_hm(transit::time(
        &transit::TransitType::Rise,
        &transit::TransitBody::Moon,
        &geograph, &eq1, &eq2, &eq3,
        apprnt_sidr, delta_t, parallax,
    ));

    let set = to_hm(transit::time(
        &transit::TransitType::Set,
        &transit::TransitBody::Moon,
        &geograph, &eq1, &eq2, &eq3,
        apprnt_sidr, delta_t, parallax,
    ));

    (rise, set)
}

fn utc_event_to_local(
    utc_date: NaiveDate,
    utc_hm: (u32, u32),
    tz_offset_hours: f64,
) -> (NaiveDate, (u32, u32)) {
    let utc_minutes = utc_hm.0 as i64 * 60 + utc_hm.1 as i64;
    let offset_minutes = (tz_offset_hours * 60.0).round() as i64;
    let local_total = utc_minutes + offset_minutes;
    let day_offset = local_total.div_euclid(1_440);
    let minute_of_day = local_total.rem_euclid(1_440) as u32;
    let local_date = utc_date + Duration::days(day_offset);
    (local_date, (minute_of_day / 60, minute_of_day % 60))
}

fn moonrise_moonset_local_on_date(
    year: i32,
    month: u32,
    day: u32,
    lat_deg: f64,
    lon_deg: f64,
    tz_offset_hours: f64,
) -> (Option<(u32, u32)>, Option<(u32, u32)>) {
    let target_date = NaiveDate::from_ymd_opt(year, month, day)
        .expect("valid local date");
    let mut moonrise = None;
    let mut moonset = None;

    for delta in -1..=1 {
        let utc_date = target_date + Duration::days(delta);
        let jd = date_to_jd(utc_date.year(), utc_date.month(), utc_date.day());
        let (rise_utc, set_utc) = moonrise_moonset_utc(
            jd,
            lat_deg,
            lon_deg,
            utc_date.year(),
            utc_date.month(),
        );

        if let Some(utc_hm) = rise_utc {
            let (local_date, local_hm) = utc_event_to_local(utc_date, utc_hm, tz_offset_hours);
            if local_date == target_date && moonrise.is_none() {
                moonrise = Some(local_hm);
            }
        }

        if let Some(utc_hm) = set_utc {
            let (local_date, local_hm) = utc_event_to_local(utc_date, utc_hm, tz_offset_hours);
            if local_date == target_date && moonset.is_none() {
                moonset = Some(local_hm);
            }
        }
    }

    (moonrise, moonset)
}

fn sunrise_sunset_utc(
    jd_utc_midnight: f64,
    lat_deg: f64,
    lon_deg: f64,
    year: i32,
    month: u32,
) -> (Option<(u32, u32)>, Option<(u32, u32)>) {
    let geograph = coords::GeographPoint {
        long: (-lon_deg).to_radians(),
        lat: lat_deg.to_radians(),
    };
    let eq1 = solar_eq_pos(jd_utc_midnight - 1.0);
    let eq2 = solar_eq_pos(jd_utc_midnight);
    let eq3 = solar_eq_pos(jd_utc_midnight + 1.0);
    let apprnt_sidr = apprnt_sidr!(jd_utc_midnight);
    let delta_t = atime::delta_t(year, month as u8);

    let to_hm = |result: (i64, i64, f64)| -> Option<(u32, u32)> {
        let (h, m, s) = result;
        let total_seconds = h as f64 * 3600.0 + m as f64 * 60.0 + s;
        let total_seconds = total_seconds.rem_euclid(86_400.0);
        let total_minutes = (total_seconds / 60.0).floor() as u32;
        Some((total_minutes / 60, total_minutes % 60))
    };

    let rise = to_hm(transit::time(
        &transit::TransitType::Rise,
        &transit::TransitBody::Sun,
        &geograph, &eq1, &eq2, &eq3,
        apprnt_sidr, delta_t, 0.0,
    ));
    let set = to_hm(transit::time(
        &transit::TransitType::Set,
        &transit::TransitBody::Sun,
        &geograph, &eq1, &eq2, &eq3,
        apprnt_sidr, delta_t, 0.0,
    ));

    (rise, set)
}

fn sunrise_sunset_local_on_date(
    year: i32,
    month: u32,
    day: u32,
    lat_deg: f64,
    lon_deg: f64,
    tz_offset_hours: f64,
) -> (Option<(u32, u32)>, Option<(u32, u32)>) {
    let target_date = NaiveDate::from_ymd_opt(year, month, day)
        .expect("valid local date");
    let mut sunrise = None;
    let mut sunset = None;

    for delta in -1..=1 {
        let utc_date = target_date + Duration::days(delta);
        let jd = date_to_jd(utc_date.year(), utc_date.month(), utc_date.day());
        let (rise_utc, set_utc) = sunrise_sunset_utc(
            jd,
            lat_deg,
            lon_deg,
            utc_date.year(),
            utc_date.month(),
        );

        if let Some(utc_hm) = rise_utc {
            let (local_date, local_hm) = utc_event_to_local(utc_date, utc_hm, tz_offset_hours);
            if local_date == target_date && sunrise.is_none() {
                sunrise = Some(local_hm);
            }
        }

        if let Some(utc_hm) = set_utc {
            let (local_date, local_hm) = utc_event_to_local(utc_date, utc_hm, tz_offset_hours);
            if local_date == target_date && sunset.is_none() {
                sunset = Some(local_hm);
            }
        }
    }

    (sunrise, sunset)
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use super::{
        date_to_jd, moonrise_moonset_local_on_date, parse_app_config, parse_language,
        resolve_date_spec, AppConfig, Language,
        sunrise_sunset_local_on_date,
    };

    #[test]
    fn date_to_jd_returns_midnight_utc() {
        assert!((date_to_jd(2000, 1, 1) - 2451544.5).abs() < 1e-9);
    }

    #[test]
    fn moonrise_moonset_tokyo_regression_2026_04_08() {
        let (moonrise, moonset) =
            moonrise_moonset_local_on_date(2026, 4, 8, 35.6762, 139.6503, 9.0);

        assert_eq!(moonrise, Some((23, 20)));
        assert_eq!(moonset, Some((7, 50)));
    }

    #[test]
    fn sunrise_sunset_tokyo_regression_2026_04_08() {
        let (sunrise, sunset) =
            sunrise_sunset_local_on_date(2026, 4, 8, 35.6762, 139.6503, 9.0);

        assert_eq!(sunrise, Some((5, 20)));
        assert_eq!(sunset, Some((18, 8)));
    }

    #[test]
    fn relative_date_keywords_use_local_day() {
        let base = NaiveDate::from_ymd_opt(2026, 4, 8).unwrap();
        assert_eq!(resolve_date_spec(base, "today").unwrap(), base);
        assert_eq!(resolve_date_spec(base, "prev").unwrap(), NaiveDate::from_ymd_opt(2026, 4, 7).unwrap());
        assert_eq!(resolve_date_spec(base, "next").unwrap(), NaiveDate::from_ymd_opt(2026, 4, 9).unwrap());
    }

    #[test]
    fn app_config_parses_lat_lon_and_art() {
        let config =
            parse_app_config("lat = 34.6937\nlon = 135.5023\nart = false\nlang = \"en\"\n")
                .unwrap();
        assert_eq!(
            config,
            AppConfig {
                lat: 34.6937,
                lon: 135.5023,
                art: false,
                lang: Language::En,
            }
        );
    }

    #[test]
    fn parse_language_supports_ja_and_en() {
        assert_eq!(parse_language("ja").unwrap(), Language::Ja);
        assert_eq!(parse_language("en").unwrap(), Language::En);
    }
}

fn phase_name(age: f64, lang: Language) -> &'static str {
    match lang {
        Language::Ja => match age {
            a if a < 1.0 => "新月",
            a if a < 6.4 => "三日月（満ちていく細い月）",
            a if a < 8.4 => "上弦の月",
            a if a < 13.8 => "十三夜月（満ちていく凸月）",
            a if a < 15.8 => "満月",
            a if a < 17.0 => "十六夜月",
            a if a < 18.0 => "立待月",
            a if a < 19.0 => "居待月",
            a if a < 20.0 => "寝待月",
            a if a < 21.2 => "更待月",
            a if a < 23.1 => "下弦の月",
            a if a < 29.0 => "有明月（欠けていく細い月）",
            _ => "晦日",
        },
        Language::En => match age {
            a if a < 1.0 => "New Moon",
            a if a < 6.4 => "Crescent Moon (waxing)",
            a if a < 8.4 => "First Quarter",
            a if a < 13.8 => "Gibbous Moon (waxing)",
            a if a < 15.8 => "Full Moon",
            a if a < 17.0 => "Izayoi Moon",
            a if a < 18.0 => "Tachimachi Moon",
            a if a < 19.0 => "Imachi Moon",
            a if a < 20.0 => "Nemachi Moon",
            a if a < 21.2 => "Fukemachi Moon",
            a if a < 23.1 => "Last Quarter",
            a if a < 29.0 => "Morning Moon (waning crescent)",
            _ => "Dark Moon",
        },
    }
}

fn label(lang: Language, key: &str) -> &'static str {
    match (lang, key) {
        (Language::Ja, "age") => "月齢",
        (Language::Ja, "illumination") => "照明率",
        (Language::Ja, "phase") => "月相",
        (Language::Ja, "distance") => "距離",
        (Language::Ja, "sunrise") => "日の出",
        (Language::Ja, "sunset") => "日の入",
        (Language::Ja, "sunrise_sunset_none") => "日の出/日の入",
        (Language::Ja, "sunrise_sunset_none_value") => "該当なし（白夜/極夜）",
        (Language::Ja, "moonrise") => "月の出",
        (Language::Ja, "moonset") => "月の入",
        (Language::Ja, "days") => "日",
        (Language::En, "age") => "Age",
        (Language::En, "illumination") => "Illum.",
        (Language::En, "phase") => "Phase",
        (Language::En, "distance") => "Distance",
        (Language::En, "sunrise") => "Sunrise",
        (Language::En, "sunset") => "Sunset",
        (Language::En, "sunrise_sunset_none") => "Sunrise/Sunset",
        (Language::En, "sunrise_sunset_none_value") => "none (polar day/night)",
        (Language::En, "moonrise") => "Moonrise",
        (Language::En, "moonset") => "Moonset",
        (Language::En, "days") => "days",
        _ => "",
    }
}

fn separator(lang: Language) -> &'static str {
    match lang {
        Language::Ja => "──",
        Language::En => ":",
    }
}

/// 明るさ (0.0〜1.0) をグラデーション文字に変換（月面用・高精細）
fn shade_char_moon(brightness: f64) -> char {
    match brightness {
        b if b < 0.08 => ' ',
        b if b < 0.16 => '・',
        b if b < 0.30 => '░',
        b if b < 0.48 => '▒',
        b if b < 0.68 => '▓',
        _ => '█',
    }
}

/// 月面のクレーター模様（決定的ノイズ）
fn surface_texture(nx: f64, ny: f64) -> f64 {
    // 複数スケールのノイズを重ねて月面のテクスチャを生成
    let s1 = ((nx * 7.0).sin() * (ny * 11.0).cos() * 0.5 + 0.5) * 0.15;
    let s2 = ((nx * 13.0 + 1.7).sin() * (ny * 17.0 + 2.3).cos() * 0.5 + 0.5) * 0.10;
    let s3 = ((nx * 23.0 + 0.3).cos() * (ny * 19.0 + 1.1).sin() * 0.5 + 0.5) * 0.08;
    // 大きなクレーター的な暗い領域（海）
    let mare1 = (-(((nx - 0.2).powi(2) + (ny + 0.1).powi(2)) * 8.0)).exp() * 0.12;
    let mare2 = (-(((nx + 0.3).powi(2) + (ny - 0.25).powi(2)) * 6.0)).exp() * 0.10;
    let mare3 = (-(((nx + 0.05).powi(2) + (ny + 0.3).powi(2)) * 10.0)).exp() * 0.08;
    s1 + s2 + s3 - mare1 - mare2 - mare3
}

/// アスキーアートで月と星空を描画
fn draw_moon(age: f64, width: usize, height: usize) -> Vec<String> {
    let phase = age / SYNODIC_MONTH; // 0.0〜1.0
    let waxing = phase <= 0.5;

    let cx = (width as f64 - 1.0) / 2.0;
    let cy = (height as f64 - 1.0) / 2.0;
    // 端末の文字セルは縦長なので、見た目で円になるよう横方向を広めに補正する。
    let rx = cx * 1.08;
    let ry = cy;

    let term_cos = (2.0 * PI * phase).cos();

    let mut lines = Vec::new();
    for row in 0..height {
        let mut line = String::new();
        let ny = (row as f64 - cy) / ry;

        for col in 0..width {
            let nx = (col as f64 - cx) / rx;
            let r2 = nx * nx + ny * ny;
            let r = r2.sqrt();

            // ── 円の外: 余白 ──
            if r > 1.0 {
                line.push(' ');
                continue;
            }

            // ── 円の内部: 月面 ──
            let dist_from_edge = 1.0 - r;
            let half_width = (1.0 - ny * ny).sqrt().max(0.001);

            // ターミネーター境界
            let term_x = term_cos * half_width;
            let signed_dist = if waxing {
                (nx - term_x) / half_width
            } else {
                (-nx - term_x) / half_width
            };

            // 滑らかなターミネーター (sigmoid)
            let sharpness = 5.2;
            let light = 1.0 / (1.0 + (-signed_dist * sharpness).exp());

            // リムダーケニング（球面効果）
            let limb = (1.0 - r2).sqrt();
            let limb_factor = 0.4 + 0.6 * limb;

            // 月面テクスチャ
            let texture = surface_texture(nx * 0.8, ny * 0.8) * 0.10;

            // 地球照
            let earthshine = 0.05;

            // 明るさ合成
            let base_brightness = (light * limb_factor).max(earthshine * limb_factor);
            let brightness = (base_brightness + texture * light).clamp(0.0, 1.0);

            // 輪郭アンチエイリアス（滑らかな境界）
            let edge_aa = (dist_from_edge * ry * 1.6).clamp(0.0, 1.0);
            let final_brightness = brightness * edge_aa;

            // 暗い面の処理
            let ch = if final_brightness < 0.04 {
                // 月の輪郭を薄く表示（新月でも形がわかる）
                if dist_from_edge < 0.04 {
                    '・'
                } else if light < 0.1 {
                    // 暗い面: かすかな地球照の表現
                    let earthglow = earthshine * limb_factor * edge_aa;
                    if earthglow > 0.032 { '・' } else { ' ' }
                } else {
                    ' '
                }
            } else {
                shade_char_moon(final_brightness)
            };
            line.push(ch);
        }
        lines.push(line);
    }
    lines
}

fn main() {
    // 引数:
    //   moon [YYYY-MM-DD|today|prev|next] [緯度 経度]
    //   moon --date 2026-04-08 --lat 35.68 --lon 139.77
    let args: Vec<String> = std::env::args().collect();
    let tz_offset: f64 = JST_OFFSET_HOURS;
    let jst = FixedOffset::east_opt((tz_offset * 3600.0) as i32)
        .expect("valid JST offset");
    let today_jst = Utc::now().with_timezone(&jst).date_naive();
    let default_config = match load_app_config() {
        Ok(Some(config)) => config,
        Ok(None) => AppConfig {
            lat: DEFAULT_LAT,
            lon: DEFAULT_LON,
            art: true,
            lang: Language::Ja,
        },
        Err(message) => {
            eprintln!("{}", message);
            std::process::exit(1);
        }
    };
    let (date, lat, lon, art_enabled, lang) = match parse_cli_args(&args, today_jst, default_config) {
        Ok(values) => values,
        Err(message) => {
            eprintln!("{}", message);
            print_usage(1);
        }
    };
    let year = date.year();
    let month = date.month();
    let day = date.day();

    let jd = local_midnight_jd(year, month, day, tz_offset);
    let (age, illum_frac, distance) = lunar_metrics(jd, year, month, day);
    let illum = illum_frac * 100.0;
    let name = phase_name(age, lang);
    let supermoon = is_supermoon(age, distance);

    println!();

    if art_enabled {
        // アスキーアート（45×21）
        let art = draw_moon(age, 45, 21);
        for line in &art {
            println!("    {}", line);
        }
        println!();
    }

    // フッター情報
    let sep = separator(lang);
    println!("    {:<9}{} {:.2} {}", label(lang, "age"), sep, age, label(lang, "days"));
    println!("    {:<9}{} {:.1}%", label(lang, "illumination"), sep, illum);
    println!("    {:<9}{} {}", label(lang, "phase"), sep, name);
    println!("    {:<9}{} {:.0} km", label(lang, "distance"), sep, distance);
    let (sunrise, sunset) = sunrise_sunset_local_on_date(year, month, day, lat, lon, tz_offset);
    if let (Some((sr_h, sr_m)), Some((ss_h, ss_m))) = (sunrise, sunset) {
        println!("    {:<9}{} {:02}:{:02} JST", label(lang, "sunrise"), sep, sr_h, sr_m);
        println!("    {:<9}{} {:02}:{:02} JST", label(lang, "sunset"), sep, ss_h, ss_m);
    } else {
        println!(
            "    {:<9}{} {}",
            label(lang, "sunrise_sunset_none"),
            sep,
            label(lang, "sunrise_sunset_none_value")
        );
    }

    // 月の出・月の入りは、指定したローカル日(JST)に属するイベントを拾う。
    let (moonrise, moonset) = moonrise_moonset_local_on_date(year, month, day, lat, lon, tz_offset);
    let fmt_moon = |opt: Option<(u32, u32)>| -> String {
        match opt {
            Some((h, m)) => format!("{:02}:{:02} JST", h, m),
            None => "──:──".to_string(),
        }
    };
    let moon_events = [
        (label(lang, "moonrise"), moonrise),
        (label(lang, "moonset"), moonset),
    ];
    let mut present_events: Vec<_> = moon_events
        .iter()
        .filter_map(|(label, time)| time.map(|(h, m)| (*label, h, m)))
        .collect();
    present_events.sort_by_key(|(_, h, m)| (*h, *m));

    for (label, h, m) in &present_events {
        println!("    {:<9}{} {:02}:{:02} JST", label, sep, h, m);
    }
    for (label, time) in &moon_events {
        if time.is_none() {
            println!("    {:<9}{} {}", label, sep, fmt_moon(*time));
        }
    }

    if supermoon {
        println!("    ** SUPER MOON **");
    }
    println!();
}
