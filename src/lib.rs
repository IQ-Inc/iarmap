//! # iarmap
//!
//! The library defines a parser for IAR map files. The library can parse the
//! module summary table of an IAR map files, returning the table(s) as a
//! collection of key-values. For instance, a module summary table with the
//! layout
//!
//! ```text
//!     Module                         ro code  ro data  rw data
//!     ------                         -------  -------  -------
//! C:\Projects\A\Obj: [1]
//!     Bar.o                               22      44
//!     Baz.o                               33      55       22
//!     --------------------------------------------------------
//!     Total:                              55      99       22
//!
//! C:\Projects\B\Obj: [2]
//!     Foo.o                               10               77
//!     Zap.o                               20      50
//!     --------------------------------------------------------
//!     Total:                              30      50       77
//! ```
//!
//! is parsed into a vector of `ObjModuleTable`s. A `Module` contains the
//! three data sizes. `Modules` may be found in an `ObjModuleTable`'s `table`
//! member. The keys are the object file names. The brief example below
//! demonstrates the representation of "Bar.o" from the table above.
//!
//! ```
//! use iarmap::Module;
//! use std::collections::HashMap;
//!
//! let bar = Module{ ro_code: Some(33), ro_data: Some(44), rw_data: None };
//! let mut table: HashMap<String, Module> = HashMap::new();
//! table.insert("Bar.o".into(), bar);
//! ```

#[macro_use]
extern crate nom;

use nom::IResult;

mod summary;
use summary::parse_module_summaries;
pub use summary::{Module, ObjModuleTable};

use std::io::Read;

/// Parse a map file, returning the module summary table, or a string
/// representing an error message.
///
/// The function takes ownership of the reader to locate and consume the module
/// summary table.
pub fn parse_map_file<R: Read>(mut reader: R) -> Result<Vec<ObjModuleTable>, &'static str> {
    let mut buffer: Vec<u8> = Vec::new();

    if let Err(_) = reader.read_to_end(&mut buffer) {
        return Err("Failed to read");
    }

    match parse_module_summaries(&buffer) {
        IResult::Done(_, objsummaries) => Ok(objsummaries),
        _ => {
            return Err("Failed to parse");
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use summary::{Module, ObjModuleTable};
    use super::parse_map_file;

    #[test]
    fn test_parse_map_file() {
        let input = "###############################################################################
#                                                                             #
# THE TYPICAL IAR HEADER                                22/Nov/2016  10:18:24 #
# Copyright (C) YYYY-YYYY Fake company here                                   #
#                                                                             #

*******************************************************************************
*** WHO CARES
***
  .rodata             const    0x080db594     0x28  error.o [1]
  .rodata             const    0x080db5bc     0x28  error.o [1]
  .rodata             const    0x080db5e4     0x28  error.o [1]
  .text               ro code  0x080de6f4     0x24  error.o [1]
  .text               ro code  0x080de718     0x24  error.o [1]
  .text               ro code  0x080de73c     0x24  error.o [1]
  .text               ro code  0x080de760     0x24  error.o [1]
  .text               ro code  0x080de784     0x24  error.o [1]
  .text               ro code  0x080de7a8     0x22  error.o [1]
  .text               ro code  0x080de7ca     0x22  error.o [1]
  .text               ro code  0x080de7ec     0x22  error.o [1]
  .text               ro code  0x080de80e     0x22  error.o [1]
  .text               ro code  0x080de830     0x22  error.o [1]
  .text               ro code  0x080de852     0x22  error.o [1]
  .iar.init_table     const    0x080de874     0x54  - Linker created -
  .rodata             const    0x080de8c8     0x20  error.o [1]
  .rodata             const    0x080de8e8     0x20  error.o [1]
Here's some other content we don't care about...
There's a lot of other content we don't care about,
but we just ramble here to simulate that 'other content.'

*******************************************************************************
*** MODULE SUMMARY
***

    Module                         ro code  ro data  rw data
    ------                         -------  -------  -------
C:\\!prj\\Foo\\Bar\\Baz\\Obj: [1]
    The_Alarm_Logging.o                 88      152       72
    Alarm_Log.o                        390
    More_Alarm_Log.o                   168                20
    --------------------------------------------------------
    Total:                         385 176  616 821  210 147

command line: [2]
    --------------------------------------------------------
    Total:

FileSys.a: [3]
    FAT_CheckDisk.o                  2 924       48       16
    FAT_Dir.o                          536       24
    --------------------------------------------------------
    Total:                              68

    Gaps                                96       90        9
    Linker created                               88  378 432
------------------------------------------------------------
    Grand Total:                   492 776  630 240  591 176


*******************************************************************************
*** ENTRY LIST
***

Entry                      Address    Size  Type      Object
-----                      -------    ----  ----      ------
.iar.dynexit$$Base      0x20004294           --   Gb  - Linker created -
.iar.dynexit$$Limit     0x200044e0           --   Gb  - Linker created -
?main                   0x080e499d          Code  Gb  cmain.o [7]"
            .as_bytes();

        let mut table1: HashMap<String, Module> = HashMap::new();
        table1.insert(
            "The_Alarm_Logging.o".into(),
            Module {
                ro_code: Some(88),
                ro_data: Some(152),
                rw_data: Some(72),
            },
        );
        table1.insert(
            "Alarm_Log.o".into(),
            Module {
                ro_code: Some(390),
                ro_data: None,
                rw_data: None,
            },
        );
        table1.insert(
            "More_Alarm_Log.o".into(),
            Module {
                ro_code: Some(168),
                ro_data: None,
                rw_data: Some(20),
            },
        );

        let table2: HashMap<String, Module> = HashMap::new();

        let mut table3: HashMap<String, Module> = HashMap::new();
        table3.insert(
            "FAT_CheckDisk.o".into(),
            Module {
                ro_code: Some(2_924),
                ro_data: Some(48),
                rw_data: Some(16),
            },
        );
        table3.insert(
            "FAT_Dir.o".into(),
            Module {
                ro_code: Some(536),
                ro_data: Some(24),
                rw_data: None,
            },
        );

        let m1 = ObjModuleTable {
            name: "C:\\!prj\\Foo\\Bar\\Baz\\Obj: [1]".into(),
            table: table1,
        };
        let m2 = ObjModuleTable {
            name: "command line: [2]".into(),
            table: table2,
        };
        let m3 = ObjModuleTable {
            name: "FileSys.a: [3]".into(),
            table: table3,
        };

        let expected = vec![m1, m2, m3];

        let result = parse_map_file(input);
        assert_eq!(result.is_ok(), true);

        let actual = result.unwrap();
        assert_eq!(actual, expected);
    }
}
