#[derive(PartialEq, Copy, Clone)]
pub struct PrefabLevel {
    pub template: &'static str,
    pub width: usize,
    pub height: usize,
}

pub const OVERMAP: PrefabLevel = PrefabLevel { template: OVERMAP_TEMPLATE, width: 69, height: 41 };

const OVERMAP_TEMPLATE: &str =
    "
^^^^^^^^^^^^^^^^^^^^^^^^^^^≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈
^^^^^^^^^^^^^^^^^^^^^^^^^^≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈..≈......≈≈≈≈≈≈≈
^^^^^^^^^^^^^^^....^^^^^^≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈.............≈≈≈≈≈
^^^^^^^^^^^^^^...........≈≈≈≈≈≈≈≈≈........≈≈≈≈≈≈≈≈..............≈≈≈≈≈
^^^^^^^^^^^^^............≈≈≈≈≈≈≈≈...........≈≈≈≈≈..............≈≈≈≈≈≈
^^^^^^^^^^^.............≈≈≈≈≈≈≈≈≈...........≈≈≈≈≈.............≈≈≈≈≈≈≈
^^^^^^^^^...............≈≈≈≈≈≈≈≈≈............≈≈≈≈............≈≈≈≈≈≈≈≈
^^^^^^^^................≈≈≈≈≈≈≈≈≈............≈≈≈≈............≈≈≈≈≈≈≈≈
^^^^^^^..................≈≈≈≈≈≈≈≈≈...........≈≈≈.............≈≈≈≈≈≈≈≈
^^^^.....................≈≈≈≈≈≈≈≈≈≈≈........≈≈≈≈..............≈≈≈≈≈≈≈
^^.........................≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈..............≈≈≈≈≈≈≈
^^..............................≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈................≈≈≈≈≈≈
^.................................≈≈≈≈≈≈≈≈≈≈≈≈≈.................≈≈≈≈≈
^..................................≈≈≈≈≈≈≈≈≈≈≈≈.................≈≈≈≈≈
^....................................≈≈≈≈≈≈≈≈≈..................≈≈≈≈≈
^......................................≈≈≈≈≈≈...................≈≈≈≈≈
^........................................≈≈.....................≈≈≈≈≈
^..............................................................≈≈≈≈≈≈
^..............................................................≈≈≈≈≈≈
^^.............................................................≈≈≈≈≈≈
^^.............................................................≈≈≈≈≈≈
^^.............................................................≈≈≈≈≈≈
^^^............................................................≈≈≈≈≈≈
^^^^............................................................≈≈≈≈≈
^^^^^...........................................................≈≈≈≈≈
^^^^^.^^........................................................≈≈≈≈≈
^^^^..^^^.......................................................≈≈≈≈≈
^^^...^^^..............................≈≈........................≈≈≈≈
^^^2.^^^^.............................≈≈≈≈≈≈≈≈≈...................≈≈≈
^^^^^^^^..............................≈≈≈≈≈≈≈≈≈≈......≈............≈≈
^^^^^^^..............................≈≈≈≈≈≈≈≈≈≈≈≈≈...≈≈..........≈..≈
^^^^^^^..........................≈≈.≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈....≈≈≈≈
^^^^^^^........................≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈.....≈
^^^^^^^^.......................≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈....≈
^^^^^^^^......@...........≈...≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈...≈≈≈...≈≈≈≈≈≈≈≈.≈≈
^^^^^^^^^.................≈≈....≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈....≈≈......≈≈≈≈≈≈≈≈≈
^^^^^^^^^........≈≈≈≈1..≈≈≈≈....≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈..≈≈≈......≈≈≈≈≈≈≈≈≈
^^^^^^^^^^......≈≈≈≈≈..≈≈≈≈≈≈..≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈........≈≈≈≈≈≈≈≈
^^^^^^^^^^.....≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈........≈≈≈≈≈≈≈≈≈
^^^^^^^^^^....≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈....≈≈≈≈≈≈≈≈≈≈
^^^^^^^^^^^^.≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈≈";
