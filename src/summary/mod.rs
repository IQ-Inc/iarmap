//! The "module summary" library module

mod module;

use nom::*;

use self::module::module_table;
pub use self::module::Module;

use std::collections::HashMap;
use std::str;

/// Relates an object dir to a map of files.
///
/// For a table
///
/// ```text
///     Module                         ro code  ro data  rw data
///     ------                         -------  -------  -------
/// C:\Projects\A\Obj: [1]
///     Bar.o                               22      44
///     Baz.o                               33      55       22
///     --------------------------------------------------------
///     Total:                              55      99       22
/// ```
///
/// The `name` member is `C:\Projects\A\Obj: [1]`, and the `table` member
/// is a `HashMap` with the object name as the key, and a `Module` as the value.
#[derive(Debug, PartialEq)]
pub struct ObjModuleTable {
    pub name: String,
    pub table: HashMap<String, Module>,
}

/// Parses the many ***** that deliminate sections
named!(stars, take_while!(|c| c == '*' as u8));

/// Parses the module summary header name
named!(module_summary, tag!("MODULE SUMMARY"));

/// Parses the entire module summary header, returning the header name
named!(
    header,
    do_parse!(
        stars >> line_ending >> stars >> space >> summary: module_summary >> line_ending >> stars >>
            (summary)
    )
);

/// Parses the table start. Consumes the table start and returns empty bytes.
named!(
    table_start,
    do_parse!(
        multispace >> tag!("Module") >> multispace >> tag!("ro code") >> space >>
            tag!("ro data") >> space >> tag!("rw data") >> line_ending >>
            take_while!(|x| (x == ' ' as u8) || (x == '-' as u8)) >> (b"")
    )
);

/// Parses the object file directory
named!(obj_header<String>,
    map!(
        do_parse!(bs: not_line_ending >> line_ending >> (bs)),
        |bs| str::from_utf8(bs).unwrap().into()
    )
);

/// Parses a table end of --------------------------
named!(table_end,
    do_parse!(
        multispace >>
        many1!(char!('-')) >>
        line_ending >>
        (&[])
    )
);

/// Parse module tables
named!(tablep< &[u8], ObjModuleTable>,
    do_parse!(
        obj: obj_header >>
        ms: module_table >>
        table_end >>
        take_until_and_consume!("\n") >>
        (ObjModuleTable{ name: obj, table: ms })
    )
);

/// Parse the module summary table from an IAR map file
named!(pub parse_module_summaries< &[u8], Vec<ObjModuleTable> >,
    do_parse!(
        many_till!(anychar, header) >>
        table_start >> line_ending >>
        ts: many0!(do_parse!(t: tablep >> multispace >> (t))) >>
        take_until!("*") >>
        (ts)
    )
);

#[cfg(test)]
mod tests {

    use super::*;
    use nom::IResult;
    use nom::IResult::Done;

    static EMPTY: &[u8] = b"";

    #[test]
    fn test_parse_header() {
        let h = "*******************************************************************************
*** MODULE SUMMARY
***"
            .as_bytes();

        assert_eq!(header(h), Done(EMPTY, &b"MODULE SUMMARY"[..]));
    }

    #[test]
    fn test_parse_table_start() {
        let h = "    Module                         ro code  ro data  rw data
    ------                         -------  -------  -------"
            .as_bytes();

        assert_eq!(table_start(h), Done(EMPTY, EMPTY));
    }

    #[test]
    fn test_parse_obj_header() {
        let h = "C:\\proj\\A\n".as_bytes();
        assert_eq!(obj_header(h), Done(EMPTY, "C:\\proj\\A".into()));
    }

    #[test]
    fn test_module_table_three_row_with_ending() {
        let table = b"                        Foo.o          724      544\n               Bar.o                   532      569      103\n            Baz.o                                88       16\n        --------------------------------------------------------\n    some other garbage";

        let mut expected: HashMap<String, Module> = HashMap::new();
        let rest = b"        --------------------------------------------------------\n    some other garbage";

        expected.insert(
            "Foo.o".into(),
            Module {
                ro_code: Some(724),
                ro_data: Some(544),
                rw_data: None,
            },
        );

        expected.insert(
            "Bar.o".into(),
            Module {
                ro_code: Some(532),
                ro_data: Some(569),
                rw_data: Some(103),
            },
        );

        expected.insert(
            "Baz.o".into(),
            Module {
                ro_code: None,
                ro_data: Some(88),
                rw_data: Some(16),
            },
        );

        let result = module_table(table);
        assert_eq!(result, IResult::Done(&rest[..], expected));
    }

    #[test]
    fn test_empty_table() {
        let input = "command line: [2]
    --------------------------------------------------------
    Total:\n"
            .as_bytes();
        let expected_map: HashMap<String, Module> = HashMap::new();
        let result = tablep(input);
        assert_eq!(result, IResult::Done(EMPTY, ObjModuleTable{ name: "command line: [2]".into(), table: expected_map }));
    }

    #[test]
    fn test_complete_parse_table() {
        let table = "myarchive.a: [6]
    libFoos.o                        6 258      300
    libBars.o                          638      112
    libBaz.o                                    768      768
    libHey.o                           122
    --------------------------------------------------------
    Total:                           5 700      244      240\n"
            .as_bytes();

        let mut exected_map: HashMap<String, Module> = HashMap::new();

        exected_map.insert(
            "libFoos.o".into(),
            Module {
                ro_code: Some(6258),
                ro_data: Some(300),
                rw_data: None,
            },
        );

        exected_map.insert(
            "libBars.o".into(),
            Module {
                ro_code: Some(638),
                ro_data: Some(112),
                rw_data: None,
            },
        );

        exected_map.insert(
            "libBaz.o".into(),
            Module {
                ro_code: None,
                ro_data: Some(768),
                rw_data: Some(768),
            },
        );

        exected_map.insert(
            "libHey.o".into(),
            Module {
                ro_code: Some(122),
                ro_data: None,
                rw_data: None,
            },
        );

        let expected_obj_name: String = "myarchive.a: [6]".into();

        let result = tablep(table);
        if let &Done(remaining, _) = &result {
            println!("{}", str::from_utf8(remaining).unwrap());
        }
        assert_eq!(result,
            IResult::Done(EMPTY, ObjModuleTable{ name: expected_obj_name, table: exected_map }));
    }

    #[test]
    #[allow(unused_variables)]
    fn test_parse_until_header() {
        let input = "
Here's some other content we don't care about...
There's a lot of other content we don't care about,
but we just ramble here to simulate that 'other content.'

*******************************************************************************
*** MODULE SUMMARY
***"
            .as_bytes();

        let result: IResult<_, _> = many_till!(input, anychar, header);
        assert_eq!(result.is_done(), true);
        assert_eq!(result.remaining_input().unwrap(), EMPTY);
    }

    #[test]
    fn test_parse_table_to_end() {
        let input = "FileSys.a: [3]
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
.iar.dynexit$$Limit     0x200044e0           --   Gb  - Linker created -"
            .as_bytes();

        named!(p< &[u8], ObjModuleTable>,
            do_parse!(
                ts: tablep >>
                take_until!("*") >>
                (ts)
            )
        );

        let result = p(input);
        let rest = "*******************************************************************************
*** ENTRY LIST
***

Entry                      Address    Size  Type      Object
-----                      -------    ----  ----      ------
.iar.dynexit$$Base      0x20004294           --   Gb  - Linker created -
.iar.dynexit$$Limit     0x200044e0           --   Gb  - Linker created -"
            .as_bytes();

        let mut files: HashMap<String, Module> = HashMap::new();
        files.insert(
            "FAT_CheckDisk.o".into(),
            Module {
                ro_code: Some(2_924),
                ro_data: Some(48),
                rw_data: Some(16),
            },
        );
        files.insert(
            "FAT_Dir.o".into(),
            Module {
                ro_code: Some(536),
                ro_data: Some(24),
                rw_data: None,
            },
        );

        assert_eq!(result.is_done(), true);
        assert_eq!(&result.remaining_input().unwrap(), &rest);

        if let IResult::Done(_, ObjModuleTable { name, table: map }) = result {
            assert_eq!(name, String::from("FileSys.a: [3]"));
            assert_eq!(map, files);
        } else {
            unreachable!();
        }

    }
}
