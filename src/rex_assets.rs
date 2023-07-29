use rltk::rex::XpFile;

rltk::embedded_resource!(TITLEIMAGE_105_56_BYTES, "../resources/title_image.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(TITLEIMAGE_105_56_BYTES, "../resources/title_image.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_POPULATED, "../resources/wfc-populated.xp");

        RexAssets { menu: XpFile::from_resource("../resources/title_image.xp").unwrap() }
    }
}
