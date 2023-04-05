use num_enum::UnsafeFromPrimitive;

use crate::{
    config::DB_FLOAT,
    db::{db_error::*, db_sql_lite::*},
};

/// Dimension = 24
#[derive(Clone, Copy)]
pub struct GeoPos {
    pub lat: f64,
    pub long: f64,
    pub elev: f64,
}

impl Default for GeoPos {
    #[inline]
    #[rustfmt::skip]
    fn default() -> Self {
         //return gandara position as default
         GeoPos { lat: 40.440_725, long: -8.682_944, elev: 51.0, }
    }
}

#[derive(UnsafeFromPrimitive)]
#[repr(u8)]
pub enum GeoPosParams {
    Latitude = 0,
    Longitude,
    Elevation,
}

pub trait ModelGeoPosConfig<'a>: DB {
    const GET_MODULE_CONFIG: &'a str = "SELECT float FROM mods_data where module=3 order by param;";
    const UPDATE_MODULE_CONFIG: &'a str = "update mods_data set float=?1 where module=3 and param=?1;";

    #[inline]
    fn get_geo_pos_config(&self) -> Result<GeoPos, DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_MODULE_CONFIG).unwrap();

        let mut geo_pos = GeoPos::default();
        let mut rows = stmt.raw_query();
        let mut idx = 0;
        let mut geo_pos_param: GeoPosParams; //alocar o espaço para a variavel
        while let Some(row) = rows.next()? {
            geo_pos_param = unsafe { GeoPosParams::from_unchecked(idx) };
            match geo_pos_param {
                GeoPosParams::Latitude => geo_pos.lat = row.get_unwrap(DB_FLOAT),
                GeoPosParams::Longitude => geo_pos.long = row.get_unwrap(DB_FLOAT),
                GeoPosParams::Elevation => geo_pos.elev = row.get_unwrap(DB_FLOAT),
            }
            idx += 1;
        }
        Ok(geo_pos)
    }

    #[inline]
    fn save_geo_pos_config(&self, geo: &GeoPos) -> SimpleResult {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::UPDATE_MODULE_CONFIG).unwrap();

        _ = stmt.raw_bind_parameter(1, geo.lat);
        _ = stmt.raw_bind_parameter(2, GeoPosParams::Latitude as u8);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(1, geo.long);
        _ = stmt.raw_bind_parameter(2, GeoPosParams::Longitude as u8);
        _ = self.exec_prep(&mut stmt);

        _ = stmt.raw_bind_parameter(1, geo.elev);
        _ = stmt.raw_bind_parameter(2, GeoPosParams::Elevation as u8);
        self.exec_prep(&mut stmt)
    }
}

impl<'a> ModelGeoPosConfig<'a> for Persist {}
