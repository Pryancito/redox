use std::process::Command;
use std::fs;
use std::os::unix::fs::FileTypeExt;
use crate::DiskInfo;

pub struct DiskManager {
    disks: Vec<DiskInfo>,
}

impl DiskManager {
    pub fn new() -> Self {
        Self {
            disks: Vec::new(),
        }
    }
    
    pub fn list_disks(&mut self) -> Vec<DiskInfo> {
        self.scan_disks();
        self.disks.clone()
    }
    
    fn scan_disks(&mut self) {
        self.disks.clear();
        
        // Escanear discos usando lsblk
        let output = Command::new("lsblk")
            .args(&["-d", "-o", "NAME,SIZE,MODEL,TYPE", "-n"])
            .output();
            
        match output {
            Ok(result) => {
                let output_str = String::from_utf8_lossy(&result.stdout);
                
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    
                    if parts.len() >= 4 && parts[parts.len() - 1] == "disk" {
                        let name = format!("/dev/{}", parts[0]);
                        let size = parts[1].to_string();
                        let model = if parts.len() > 3 {
                            parts[2..parts.len() - 1].join(" ")
                        } else {
                            "Unknown".to_string()
                        };
                        
                        // Verificar que el disco existe y es accesible
                        if self.is_disk_accessible(&name) {
                            self.disks.push(DiskInfo {
                                name: name.clone(),
                                size,
                                model,
                                disk_type: self.get_disk_type(&name),
                            });
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("⚠️  Error escaneando discos: {}", e);
            }
        }
    }
    
    fn is_disk_accessible(&self, disk_path: &str) -> bool {
        // Verificar que el disco existe y es accesible
        if let Ok(metadata) = fs::metadata(disk_path) {
            metadata.file_type().is_block_device()
        } else {
            false
        }
    }
    
    fn get_disk_type(&self, disk_path: &str) -> String {
        // Determinar el tipo de disco
        if disk_path.contains("nvme") {
            "NVMe SSD".to_string()
        } else if disk_path.contains("sd") {
            // Intentar determinar si es SSD o HDD
            let device_name = disk_path.trim_start_matches("/dev/");
            let rotational_path = format!("/sys/block/{}/queue/rotational", device_name);
            
            if let Ok(content) = fs::read_to_string(&rotational_path) {
                if content.trim() == "0" {
                    return "SATA SSD".to_string();
                } else {
                    return "SATA HDD".to_string();
                }
            }
            
            "SATA/SCSI".to_string()
        } else if disk_path.contains("hd") {
            "IDE HDD".to_string()
        } else if disk_path.contains("vd") {
            "Virtual Disk".to_string()
        } else if disk_path.contains("mmcblk") {
            "MMC/SD Card".to_string()
        } else {
            "Unknown".to_string()
        }
    }
    
    pub fn is_disk_mounted(&self, disk_path: &str) -> bool {
        let output = Command::new("mount")
            .output();
            
        match output {
            Ok(result) => {
                let output_str = String::from_utf8_lossy(&result.stdout);
                output_str.contains(disk_path)
            }
            Err(_) => false
        }
    }
    
    pub fn unmount_disk(&self, disk_path: &str) -> Result<(), String> {
        // Intentar desmontar todas las particiones del disco
        let device_name = disk_path.trim_start_matches("/dev/");
        
        // Listar todas las particiones montadas
        let output = Command::new("mount")
            .output()
            .map_err(|e| format!("Error ejecutando mount: {}", e))?;
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        
        for line in output_str.lines() {
            if line.contains(device_name) {
                if let Some(partition) = line.split_whitespace().next() {
                    println!("   Desmontando {}...", partition);
                    let _ = Command::new("umount")
                        .arg(partition)
                        .output();
                }
            }
        }
        
        Ok(())
    }
}

