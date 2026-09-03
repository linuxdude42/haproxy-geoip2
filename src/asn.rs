use std::net::IpAddr;

use maxminddb::geoip2::Asn;
use maxminddb::LookupResult;
use mlua::prelude::{IntoLua, Lua, LuaValue};

use crate::db::Database;
use crate::GeoValue;

// Global maxmind ASN database shared between all workers
pub(crate) static DB: Database = Database::new();

pub(crate) fn lookup<'a>(lua: &'a Lua, ip: IpAddr, props: &[String]) -> Option<LuaValue> {
    DB.check_status(lua);

    let db = DB.load();
    let reader = db.as_ref()?;
    let lookup = reader.lookup(ip).ok()?;
    let asn = lookup.decode::<Asn>().ok().flatten()?;
    lookup_asn(&lookup, &asn, props).and_then(|v| v.into_lua(lua).ok())
}

fn lookup_asn<'a>(lookup: &LookupResult<'a, Vec<u8>>, asn: &'a Asn, props: &[String]) -> Option<GeoValue<'a>> {
    match props.get(0)?.as_str() {
	"mask" | "network_mask" | "prefix_len" => {
            match lookup.network() {
                Ok(network) => {
                    return Some(GeoValue::UInt(network.prefix() as u32));
                    }
                Err(_) => return None,
            };
        }
        "autonomous_system_number" | "asn" => {
            if let Some(number) = asn.autonomous_system_number {
                return Some(GeoValue::UInt(number));
            }
        }
        "autonomous_system_organization" => {
            if let Some(org) = asn.autonomous_system_organization {
                return Some(GeoValue::Str(org));
            }
        }
        _ => {}
    }
    None
}
