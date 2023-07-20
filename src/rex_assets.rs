use rltk::rex::XpFile;

rltk::embedded_resource!(CAVE_TUNNEL, "../resources/cave_tunnel80x60.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
rltk::embedded_resource!(WFC_DEMO_IMAGE2, "../resources/wfc-demo2.xp");

pub struct RexAssets {
    pub menu: XpFile,
}

impl RexAssets {
    #[allow(clippy::new_without_default)]
    pub fn new() -> RexAssets {
        rltk::link_resource!(CAVE_TUNNEL, "../resources/cave_tunnel80x60.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE1, "../resources/wfc-demo1.xp");
        rltk::link_resource!(WFC_DEMO_IMAGE2, "../resources/wfc-demo2.xp");

        RexAssets { menu: XpFile::from_resource("../resources/cave_tunnel80x60.xp").unwrap() }
    }
}
