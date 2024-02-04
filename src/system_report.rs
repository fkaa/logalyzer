use std::fs::File;
use std::io::BufReader;

#[derive(Debug)]
pub struct SystemReport {
    archive: zip::ZipArchive<BufReader<File>>,
    client_logs: Vec<(String, zip::DateTime)>,
    server_logs: Vec<(String, zip::DateTime)>,
}

struct SystemInformation {
    program_version: String,
    protocol_version: String,
    process: String,
    directx_version: String,
    net_clr_version: String,
    application_culture: String,
    installation_path: String,
    is_axir_nvr: bool,
    machine_name: String,
    operating_system: String,
    os_culture: String,
    os_version: String,
    domain: bool,
    generated: String,
}

struct NetworkInformation {
    adapters: Vec<NetworkAdapter>,
}

struct NetworkAdapter {
    name: String,
    ip: String,
}

pub fn open(path: &str) -> anyhow::Result<SystemReport> {
    let reader = std::fs::File::open(path)?;
    let mut zip = zip::ZipArchive::new(BufReader::new(reader))?;

    let mut client_logs = Vec::new();
    let mut server_logs = Vec::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i)?;

        if file.name().ends_with(".log") {
            if file.name().starts_with("Server/AcsService.exe") {
                server_logs.push((file.name().to_string(), file.last_modified()));
            } else if file.name().starts_with("Client/AcsClient.exe") {
                client_logs.push((file.name().to_string(), file.last_modified()));
            }
        }

        println!("Filename: {}", file.name());
    }

    let system_report = SystemReport {
        archive: zip,
        client_logs,
        server_logs,
    };

    Ok(system_report)
}
