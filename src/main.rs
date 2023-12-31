extern crate winrt_notification;

use std::fmt::{Display, Formatter};
use std::io::Error;
use std::time;
use snmp::{SnmpError, SyncSession, Value};
use clap::Parser;
use winrt_notification::{Duration, Sound, Toast};

#[derive(Debug)]
struct MyError{
    message: String,
}

impl Display for MyError{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", self.message)
    }
}

impl From<std::io::Error> for MyError{
    fn from(value: Error) -> Self {
        MyError { message: value.to_string()}
    }
}

impl From<SnmpError> for MyError{
    fn from(value: SnmpError) -> Self {
        MyError { message: format!("{:?}",value)}
    }
}

fn show_notification(message: &str) {
    // Erstellen eines Toast-Objekts mit dem Nachrichtentext
    let toast=Toast::new(Toast::POWERSHELL_APP_ID);
        toast
        .title("Warnung Stromversorgung")
        .text1(message)
        .sound(Some(Sound::SMS))
        .duration(Duration::Long)
        .show()
        .expect("kann keinen toast erstellen");
}

#[derive(Parser)]
struct Args {
    /// USV_IP
    usv_ip: String,
    /// USV_COMMUNITY
    usv_community: String,
}

fn get_input_voltage(apc_ip: &str, apc_community: &[u8]) -> Result<usize,MyError> {
    let apc_input_voltage_oid = [1,3,6,1,4,1,318,1,1,1,3,3,1,0];
    let timeout = std::time::Duration::from_secs(2);
    let mut session = SyncSession::new(apc_ip, apc_community,Some(timeout), 0)?;
    let mut response = session.get(&apc_input_voltage_oid)?;

    if let Some((_oid, value)) = response.varbinds.next() {
        // Überprüfen Sie, ob der Wert ein Integer ist
        if let Value::Unsigned32(battery_time) = value {
            // Zeigen Sie die verbleibende Batterielaufzeit im Terminal an
            let voltage:usize = battery_time as usize / 10;
            return Ok(voltage)
        }
    }
    Err(MyError{message: "error reading voltage".to_string()})
}

fn get_remaining_minutes(apc_ip: &str, apc_community: &[u8]) -> Result<usize,MyError> {
    let apc_battery_time_oid = [1,3,6,1,4,1,318,1,1,1,2,2,3,0];
    let timeout = std::time::Duration::from_secs(2);
    let mut session = SyncSession::new(apc_ip, apc_community,Some(timeout), 0)?;
    let mut response = session.get(&apc_battery_time_oid)?;
    if let Some((_oid, value)) = response.varbinds.next() {
        // Überprüfen Sie, ob der Wert ein Integer ist
        if let Value::Timeticks(battery_time) = value {
            // Zeigen Sie die verbleibende Batterielaufzeit im Terminal an
            let minutes :usize = battery_time as usize / 100 /60;
            return Ok(minutes)
        }
    }
    Err(MyError{message: "error reading time remaining".to_string()})
}


fn main() {

    let args=Args::parse();
    let usv_ip =format!("{}:161",args.usv_ip);
    let usv_community: String = args.usv_community;
    // Definieren Sie die IP-Adresse und den Community-String der APC-USV
    let apc_ip = &usv_ip;
    let apc_community = usv_community.as_bytes();

    loop {

        match get_input_voltage(apc_ip, apc_community) {
            Ok(input_voltage) => {
                // got input voltage
                match get_remaining_minutes(apc_ip, apc_community) {
                    Ok(restzeit) =>{
                        // got restzeit and input voltage
                        if input_voltage < 200{
                            let message=format!(
                            "Eingangsspannung: {} Volt \nRestlaufzeit: {} Minuten",
                            input_voltage,restzeit);
                            show_notification( &message );
                        }
                    }
                    Err(e) =>{
                        // Error getting restzeit
                        let message=format!(
                            "Fehler: {}",e);
                        show_notification( &message );
                    }
                }
            }
            Err(e) => {
                // Error getting input voltage
                let message=format!(
                            "Fehler: {}",e);
                show_notification( &message );
            }
        }

        std::thread::sleep(time::Duration::from_secs(30));
    }

}
