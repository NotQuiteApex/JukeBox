// build.rs

use std::process::Command;

extern crate winres;

fn main() {
    if cfg!(target_os = "windows") {
        // Generate applogo ico
        Command::new("magick")
            .args([
                "../assets/applogo.png",
                "-define",
                "icon:auto-resize=16,32,64,128,256",
                "../assets/applogo.ico",
            ])
            .output()
            .expect("Failed to run ImageMagick 7.0 to generate applogo.ico");

        let mut res = winres::WindowsResource::new();

        // add icon
        res.set_icon("../assets/applogo.ico");

        // require admin perms (necessary for CPU temp)
        // res.set_manifest(
        //     r#"
        //         <assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
        //         <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
        //             <security>
        //                 <requestedPrivileges>
        //                     <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        //                 </requestedPrivileges>
        //             </security>
        //         </trustInfo>
        //         </assembly>
        //     "#,
        // );

        // compile
        res.compile().unwrap();
    }
}
