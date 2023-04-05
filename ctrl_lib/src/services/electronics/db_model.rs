use crate::db::{db_error::*, db_sql_lite::*};
use crate::services::electronics::{device_gen::*, device_lawn::*, devices_and_scenes::*, devices_svc::*, scene::Scene};

pub trait DBModelDevice<'a>: DB {
    const GET_DEVICES_REGA: &'a str = "select id,identifier,status,ip,cmd_on, cmd_off, get_status from device where id<=5;";
    const GET_ALL_DEVICES: &'a str =
        "select id,identifier,status,ip,cmd_on,cmd_off,get_status,desc,cmd_up,cmd_stop,cmd_down,shutter_get_status,device_type from device where id>5;";
    const GET_SCENES: &'a str = "select id,desc from scene order by id;";
    const GET_SCENE_DEVICES: &'a str = "select id_device from scene_device where id_scene=?;";

    fn get_devices(&self, list: &mut DeviceWateringList) -> SimpleResult;
    fn get_devices_and_scenes(&self, desc_to_device: &mut DevicesMapList) -> Result<DevicesAndScenesData, DBError>;
}

impl<'a> DBModelDevice<'a> for Persist {
    #[inline]
    fn get_devices(&self, list: &mut DeviceWateringList) -> Result<(), DBError> {
        let conn = &self.get_conn().conn;
        let mut stmt = conn.prepare_cached(Self::GET_DEVICES_REGA).unwrap();

        let mut rows = stmt.raw_query();

        while let Some(row) = rows.next()? {
            let device: DeviceLawn = row.into();
            list.insert(device.id, device);
        }
        // Isto é unsafe, por causa da potencial concorrência nos updates,
        // mas o update é feito no arranque uma unica vez, e a partir daí será só leitura.
        unsafe { NR_OF_WATER_DEVICES = list.len() as u16 };
        Ok(())
    }

    #[inline]
    fn get_devices_and_scenes(&self, desc_to_device: &mut DevicesMapList) -> Result<DevicesAndScenesData, DBError> {
        let conn = &self.get_conn().conn;

        let mut stmt = conn.prepare_cached(Self::GET_ALL_DEVICES).unwrap();
        let mut rows = stmt.raw_query();

        let mut data = DevicesAndScenesData::new();

        // get devices
        let mut device: DeviceGen;
        while let Some(row) = rows.next()? {
            device = row.into();
            desc_to_device.insert(device.desc.clone(), data.devices.len() + 1);
            data.devices.push(device);
        }

        // get scenes
        let mut stmt = conn.prepare_cached(Self::GET_SCENES).unwrap();
        let mut rows = stmt.raw_query();
        let mut scene: Scene;
        while let Some(row) = rows.next()? {
            scene = row.into();
            data.scenes.push(scene);
        }

        for i in 0..data.scenes.len() {
            let mut stmt = conn.prepare_cached(Self::GET_SCENE_DEVICES).unwrap();
            _ = stmt.raw_bind_parameter(1, data.scenes[i].id);
            let mut rows = stmt.raw_query();
            while let Some(row) = rows.next()? {
                data.scenes[i].devices.push(row.get(0).unwrap());
            }
        }

        Ok(data)
    }
}
