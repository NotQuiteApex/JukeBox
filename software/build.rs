// build.rs

extern crate winresource;

fn main() {
    if cfg!(target_family = "windows") {
        // add icon
        let mut res = winresource::WindowsResource::new();
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
