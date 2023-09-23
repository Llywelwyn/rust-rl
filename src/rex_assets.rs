use bracket_lib::prelude::*;

embedded_resource!(TITLEIMAGE_105_56_BYTES, "../resources/title_image.xp");
embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
embedded_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        link_resource!(TITLEIMAGE_105_56_BYTES, "../resources/title_image.xp");
        link_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
        link_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

        RexAssets { menu: XpFile::from_resource("../resources/title_image.xp").unwrap() }
    }
}
