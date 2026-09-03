use std::net::IpAddr;

use maxminddb::geoip2::city::Country;
use maxminddb::geoip2::City;
use maxminddb::LookupResult;
use mlua::prelude::{IntoLua, Lua, LuaValue};

use crate::db::Database;
use crate::GeoValue;

// Global maxmind city database shared between all workers
pub(crate) static DB: Database = Database::new();

pub(crate) fn lookup<'a>(lua: &Lua, ip: IpAddr, props: &[String]) -> Option<LuaValue> {
    DB.check_status(lua);

    let db = DB.load();
    let reader = db.as_ref()?;
    let lookup = reader.lookup(ip).ok()?;
    let city = lookup.decode::<City>().ok().flatten()?;
    lookup_city(&lookup, &city, props).and_then(|v| v.into_lua(lua).ok())
}

fn lookup_city<'a>(lookup: &LookupResult<'a, Vec<u8>>, city: &'a City, props: &[String]) -> Option<GeoValue<'a>> {
    match props.get(0)?.as_str() {
	"mask" | "network_mask" | "prefix_len" => {
            match lookup.network() {
                Ok(network) => {
                    return Some(GeoValue::UInt(network.prefix() as u32));
                    }
                Err(_) => return None,
            };
        }
        "city" => match props.get(1).map(|s| s.as_str()) {
            Some("names") => {
                match props.get(2)?.as_str() {
                    "de"    => city.city.names.german.map(GeoValue::Str),
                    "en"    => city.city.names.english.map(GeoValue::Str),
                    "es"    => city.city.names.spanish.map(GeoValue::Str),
                    "fr"    => city.city.names.french.map(GeoValue::Str),
                    "ja"    => city.city.names.japanese.map(GeoValue::Str),
                    "pt-BR" => city.city.names.brazilian_portuguese.map(GeoValue::Str),
                    "ru"    => city.city.names.russian.map(GeoValue::Str),
                    "zh-CN" => city.city.names.simplified_chinese.map(GeoValue::Str),
                    _ =>       city.city.names.english.map(GeoValue::Str),
                };
            }
            Some(_) => {}
            None => {
                return city.city.names.english.map(GeoValue::Str)
            }
        },
        "country" => {
            return lookup_country(&city.country, &props[1..]);
        }
        "registered_country" => {
            return lookup_country(&city.registered_country, &props[1..]);
        }
        "location" => match props.get(1).map(|s| s.as_str()) {
            Some("latitude") => {
                return city.location.latitude.map(GeoValue::Float)
            }
            Some("longitude") => {
                return city.location.longitude.map(GeoValue::Float);
            }
            Some("timezone") => {
                return city.location.time_zone.map(GeoValue::Str);
            }
            Some("metro_code") => {
                if let Some(code) = city.location.metro_code {
                    return Some(GeoValue::UInt(code as u32));
                }
            }
            _ => {}
        },
        "postal" => match props.get(1).map(|s| s.as_str()) {
            Some("code") | None => {
                return city.postal.code.map(GeoValue::Str);
            }
            _ => {}
        },
        "continent" => match props.get(1).map(|s| s.as_str()) {
            Some("code") | None => {
                return city.continent.code.map(GeoValue::Str);
            }
            Some("names") => {
                match props.get(2)?.as_str() {
                    "de"    => city.continent.names.german.map(GeoValue::Str),
                    "en"    => city.continent.names.english.map(GeoValue::Str),
                    "es"    => city.continent.names.spanish.map(GeoValue::Str),
                    "fr"    => city.continent.names.french.map(GeoValue::Str),
                    "ja"    => city.continent.names.japanese.map(GeoValue::Str),
                    "pt-BR" => city.continent.names.brazilian_portuguese.map(GeoValue::Str),
                    "ru"    => city.continent.names.russian.map(GeoValue::Str),
                    "zh-CN" => city.continent.names.simplified_chinese.map(GeoValue::Str),
                    _ =>       city.continent.names.english.map(GeoValue::Str),
                };
            }
            _ => {}
        },
        "subdivisions" | "subdivision" => {
            let subdivisions = &city.subdivisions;
            {
                if let Some(index) = props.get(1).and_then(|s| s.parse::<usize>().ok()) {
                    if let Some(subdivision) = subdivisions.get(index) {
                        match props.get(2).map(|s| s.as_str()) {
                            Some("names") => {
                                match props.get(3)?.as_str() {
                                    "de"    => subdivision.names.german.map(GeoValue::Str),
                                    "en"    => subdivision.names.english.map(GeoValue::Str),
                                    "es"    => subdivision.names.spanish.map(GeoValue::Str),
                                    "fr"    => subdivision.names.french.map(GeoValue::Str),
                                    "ja"    => subdivision.names.japanese.map(GeoValue::Str),
                                    "pt-BR" => subdivision.names.brazilian_portuguese.map(GeoValue::Str),
                                    "ru"    => subdivision.names.russian.map(GeoValue::Str),
                                    "zh-CN" => subdivision.names.simplified_chinese.map(GeoValue::Str),
                                    _ =>       subdivision.names.english.map(GeoValue::Str),
                                };
                            }
                            Some("iso_code") | None => {
                                if let Some(code) = subdivision.iso_code {
                                    return Some(GeoValue::Str(code));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        "traits" => {
            let traits = &city.traits;
            {
                match props.get(1).map(|s| s.as_str()) {
                    Some("is_anycast") => {
                        return traits.is_anycast.map(GeoValue::Bool);
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
    None
}

fn lookup_country<'a>(country: &'a Country, props: &[String]) -> Option<GeoValue<'a>> {
    match props.get(0).map(|s| s.as_str()) {
        Some("names") => {
            match props.get(2)?.as_str() {
                "de"    => country.names.german.map(GeoValue::Str),
                "en"    => country.names.english.map(GeoValue::Str),
                "es"    => country.names.spanish.map(GeoValue::Str),
                "fr"    => country.names.french.map(GeoValue::Str),
                "ja"    => country.names.japanese.map(GeoValue::Str),
                "pt-BR" => country.names.brazilian_portuguese.map(GeoValue::Str),
                "ru"    => country.names.russian.map(GeoValue::Str),
                "zh-CN" => country.names.simplified_chinese.map(GeoValue::Str),
                _ =>       country.names.english.map(GeoValue::Str),
            };
        }
        Some("iso_code") | None => {
            if let Some(code) = country.iso_code {
                return Some(GeoValue::Str(code));
            }
        }
        Some("is_in_european_union") => {
            if let Some(is_eu) = country.is_in_european_union {
                return Some(GeoValue::Bool(is_eu));
            }
        }
        _ => {}
    }
    None
}
