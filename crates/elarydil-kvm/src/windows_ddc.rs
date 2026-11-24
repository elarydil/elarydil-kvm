//! Win32 physical monitor handles are resolved from GDI display interface + registry EDID.
//! A logical display exposing several physical handles is inhibited rather than guessed.
use crate::{model::*, platform::capability_inputs, Error, Result};
use serde_json::{json, Value};
use std::{mem, ptr};
use windows_sys::Win32::{Devices::Display::*, Graphics::Gdi::*, System::Registry::*};
struct Physical {
    handle: windows_sys::Win32::Foundation::HANDLE,
    monitor: Monitor,
}
impl Drop for Physical {
    fn drop(&mut self) {
        unsafe {
            DestroyPhysicalMonitor(self.handle);
        }
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn string(s: &[u16]) -> String {
    String::from_utf16_lossy(&s[..s.iter().position(|x| *x == 0).unwrap_or(s.len())])
}
fn enumerate() -> Result<Vec<Physical>> {
    unsafe extern "system" fn callback(
        h: HMONITOR,
        _: HDC,
        _: *mut windows_sys::Win32::Foundation::RECT,
        p: isize,
    ) -> i32 {
        (*(p as *mut Vec<HMONITOR>)).push(h);
        1
    }
    let mut handles = Vec::<HMONITOR>::new();
    if unsafe {
        EnumDisplayMonitors(
            ptr::null_mut(),
            ptr::null(),
            Some(callback),
            &mut handles as *mut _ as isize,
        )
    } == 0
    {
        return Err(Error::new("ddc_unavailable"));
    }
    let mut out = vec![];
    for h in handles {
        unsafe {
            let mut info: MONITORINFOEXW = mem::zeroed();
            info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;
            if GetMonitorInfoW(h, (&mut info as *mut MONITORINFOEXW).cast()) == 0 {
                continue;
            }
            let mut device: DISPLAY_DEVICEW = mem::zeroed();
            device.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;
            if EnumDisplayDevicesW(
                info.szDevice.as_ptr(),
                0,
                &mut device,
                1, /* EDD_GET_DEVICE_INTERFACE_NAME */
            ) == 0
            {
                continue;
            }
            let interface = string(&device.DeviceID);
            let parts: Vec<_> = interface.split('#').collect();
            if parts.len() < 3 {
                continue;
            }
            let subkey = wide(&format!(
                "SYSTEM\\CurrentControlSet\\Enum\\DISPLAY\\{}\\{}\\Device Parameters",
                parts[1], parts[2]
            ));
            let mut edid = [0u8; 4096];
            let mut len = edid.len() as u32;
            if RegGetValueW(
                HKEY_LOCAL_MACHINE,
                subkey.as_ptr(),
                wide("EDID").as_ptr(),
                RRF_RT_REG_BINARY,
                ptr::null_mut(),
                edid.as_mut_ptr().cast(),
                &mut len,
            ) != 0
                || len < 128
                || !crate::platform::valid_edid(&edid)
            {
                continue;
            }
            let mut count = 0;
            if GetNumberOfPhysicalMonitorsFromHMONITOR(h, &mut count) == 0 || count == 0 {
                continue;
            }
            let mut physical: Vec<PHYSICAL_MONITOR> = (0..count).map(|_| mem::zeroed()).collect();
            if GetPhysicalMonitorsFromHMONITOR(h, count, physical.as_mut_ptr()) == 0 {
                continue;
            }
            if count != 1 {
                for p in physical {
                    DestroyPhysicalMonitor(p.hPhysicalMonitor);
                }
                continue;
            }
            let p = &physical[0];
            let description = p.szPhysicalMonitorDescription;
            let mut identity = edid[..128].to_vec();
            identity[126] = 0;
            identity[127] = 0;
            let fingerprint = hash(&identity);
            let name = edid[54..126]
                .chunks(18)
                .find(|d| d[..5] == [0, 0, 0, 0xfc, 0])
                .map(|d| String::from_utf8_lossy(&d[5..]).trim().to_owned())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| string(&description));
            let m = Monitor {
                id: format!(
                    "monitor_{}",
                    hash(format!("{fingerprint}:{interface}").as_bytes())
                ),
                name,
                fingerprint,
                connector: interface.clone(),
                backend: "windows-dxva2".into(),
                path: interface,
                edid_available: true,
                permissions: "unknown".into(),
                limitations: vec!["hardware_not_verified".into()],
            };
            out.push(Physical {
                handle: p.hPhysicalMonitor,
                monitor: m,
            });
        }
    }
    Ok(out)
}
pub fn monitors() -> Result<Vec<Monitor>> {
    Ok(enumerate()?.iter().map(|p| p.monitor.clone()).collect())
}
pub fn direct(m: &Monitor, op: &str, vcp: u8, value: u8) -> Result<Value> {
    let all = enumerate()?;
    let matches: Vec<_> = all
        .iter()
        .filter(|p| p.monitor.fingerprint == m.fingerprint && p.monitor.connector == m.connector)
        .collect();
    if matches.is_empty() {
        return Err(Error::new("monitor_missing"));
    }
    if matches.len() != 1 {
        return Err(Error::new("ambiguous_monitor"));
    }
    let h = matches[0].handle;
    unsafe {
        match op {
            "write" => {
                if SetVCPFeature(h, vcp, value as u32) == 0 {
                    return Err(Error::new("ddc_unavailable"));
                }
                Ok(json!({"outcome":"write_api_success","verified":false,"value":value}))
            }
            "read" => {
                let mut current = 0;
                let mut max = 0;
                if GetVCPFeatureAndVCPFeatureReply(h, vcp, ptr::null_mut(), &mut current, &mut max)
                    == 0
                {
                    return Err(Error::new("ddc_unavailable"));
                }
                Ok(json!({"value":current,"maximum":max}))
            }
            "capabilities" => {
                let mut len = 0;
                if GetCapabilitiesStringLength(h, &mut len) == 0 || len > 65536 {
                    return Err(Error::new("ddc_unavailable"));
                }
                let mut buf = vec![0u8; len as usize];
                if CapabilitiesRequestAndCapabilitiesReply(h, buf.as_mut_ptr(), len) == 0 {
                    return Err(Error::new("ddc_unavailable"));
                }
                let raw = String::from_utf8_lossy(&buf)
                    .trim_end_matches('\0')
                    .to_string();
                Ok(json!({"values":capability_inputs(&raw),"raw":raw}))
            }
            _ => Err(Error::new("unsupported")),
        }
    }
}
