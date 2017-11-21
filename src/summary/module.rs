//! Describes the parsing of a module summary table

use nom::*;

use std::option::Option;
use std::str;
use std::string::String;
use std::collections::HashMap;
use std::ops::Sub;
use std::fmt;

/// A module description. All fields are optional.
///
/// Two modules may be subtracted. Values subtract if-and-only-if each is valid
/// across modules. That is, `Some(v) - None == None`, and `None - Some(v) ==
/// None`.
///
/// ```
/// use iarmap::Module;
///
/// let a = Module{ ro_code: Some(10), ro_data: None, rw_data: Some(20) };
/// let b = Module{ ro_code: Some(5), ro_data: Some(4), rw_data: Some(11) };
///
/// let expected = Module{ ro_code: Some(5), ro_data: None, rw_data: Some(9) };
/// assert_eq!(a - b, expected);
/// ```
///
/// A `Module` will pretty-print with the `{}` formatter as
///
/// ```text
/// ro_code:    526      ro_data:    436         rw_data: ------
/// ```
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct Module {
    pub ro_code: Option<i32>,
    pub ro_data: Option<i32>,
    pub rw_data: Option<i32>,
}

impl Module {
    /// Returns the total size of the Module; concretely, the sum of the three
    /// fields.
    pub fn total(&self) -> i32 {
        self.ro_code.map_or(0, |v| {
            self.ro_data.map_or(
                0,
                |q| self.rw_data.map_or(0, |r| v + q + r),
            )
        })
    }
}

/// Subtract two optional values iff both values exist.
#[inline]
fn optional_diff(left: Option<i32>, right: Option<i32>) -> Option<i32> {
    match (left, right) {
        (Some(l), Some(r)) => Some(l - r),
        _ => None,
    }
}

impl Sub for Module {
    type Output = Module;
    fn sub(self, other: Module) -> Module {
        let ro_code = optional_diff(self.ro_code, other.ro_code);
        let ro_data = optional_diff(self.ro_data, other.ro_data);
        let rw_data = optional_diff(self.rw_data, other.rw_data);
        Module {
            ro_code,
            ro_data,
            rw_data,
        }
    }
}

/// Format an optional number as a string. Describes the pretty-print format.
#[inline]
fn size_to_string(opt: Option<i32>) -> String {
    match opt {
        Some(v) => format!("{num:>width$}", num = v, width = 6),
        None => "------".into(),
    }
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ro_code: String = size_to_string(self.ro_code);
        let ro_data: String = size_to_string(self.ro_data);
        let rw_data: String = size_to_string(self.rw_data);
        write!(
            f,
            "ro_code: {} \t ro_data: {} \t rw_data: {}",
            ro_code,
            ro_data,
            rw_data
        )
    }
}

const MODULE_DATA_SIZE_MAX_BYTES: usize = 7;
const MODULE_DATA_SIZE_DELIMITING_BYTES: usize = 2;
const RO_CODE: usize = MODULE_DATA_SIZE_MAX_BYTES;
const RO_CODE_RO_DATA: usize = 2 * MODULE_DATA_SIZE_MAX_BYTES + MODULE_DATA_SIZE_DELIMITING_BYTES;
const RO_CODE_RO_DATA_RW_DATA: usize = 3 * MODULE_DATA_SIZE_MAX_BYTES +
    2 * MODULE_DATA_SIZE_DELIMITING_BYTES;


/// Utility that accepts bytes and possibly returns a string without spaces
#[inline]
fn bs_to_spaceless_string(bs: &[u8]) -> Option<String> {
    if let Ok(s) = str::from_utf8(bs) {
        Some(s.chars().filter(|c| c != &' ').collect())
    } else {
        None
    }
}

/// Parses the obj file name
fn _namep(input: &[u8], nbytes: usize) -> IResult<&[u8], Option<String>> {
    map!(input, take!(nbytes), bs_to_spaceless_string)
}

/// Parses the obj file name. If a table end is detected,
/// a parser error is thrown to force a backtrack and end the
/// parsing.
fn namep(input: &[u8], nbytes: usize) -> IResult<&[u8], Option<String>> {
    let result = _namep(input, nbytes);
    match result {
        IResult::Done(rest, Some(name)) => {
            let dashes: String = name.chars().filter(|c| c == &'-').collect();
            if name.chars().filter(|c| c != &' ').collect::<String>() == dashes {
                // Force a backtrack if a table end is detected...
                // This is inefficient, but it gets the job done.
                IResult::Error(ErrorKind::Custom(99))
            } else {
                IResult::Done(rest, Some(name))
            }
        }
        _ => {
            println!("Warning: name parser failed: {:?}", result);
            result
        }
    }
}

/// Parses a section size
named!(sizep<&[u8], Option<i32> >,
    map!(
        map_opt!(
            take!(MODULE_DATA_SIZE_MAX_BYTES),
            bs_to_spaceless_string
        ),
        |s| s.parse::<i32>().ok()
    )
);

/// Parses ro_code only
named!(size_ro_code<&[u8], Module>,
    map!(
        sizep,
        |s| { Module{ ro_code: s, ro_data: None, rw_data: None}}
    )
);

/// Parses ro_code and ro_data
named!(size_ro_code_ro_data<&[u8], Module>,
    map!(
        do_parse!(
            ro_code: sizep >> take!(MODULE_DATA_SIZE_DELIMITING_BYTES) >>
            ro_data: sizep >>
            (ro_code, ro_data)
        ),
        |s| { Module{ ro_code: s.0, ro_data: s.1, rw_data: None}}
    )
);

/// Parses ro_code, ro_data, and rw_data
named!(size_ro_code_ro_data_rw_data<&[u8], Module>,
    map!(
        do_parse!(
            ro_code: sizep >> take!(MODULE_DATA_SIZE_DELIMITING_BYTES) >>
            ro_data: sizep >> take!(MODULE_DATA_SIZE_DELIMITING_BYTES) >>
            rw_data: sizep >>
            (ro_code, ro_data, rw_data)
        ),
        |s| { Module{ ro_code: s.0, ro_data: s.1, rw_data: s.2}}
    )
);

/// Accepts the remaining input, as well as bytes representing the modules
/// sizes. Chooses an appropriate parses based on the length of the sizes slice,
/// and parses a Module value. Returns the remaining input and the module.
fn _size_rowp<'a>(input: &'a [u8], sizes: &[u8]) -> IResult<&'a [u8], Module> {
    let len = sizes.len();

    let parser: fn(&[u8]) -> IResult<&[u8], Module> = if len == RO_CODE {
        size_ro_code
    } else if len == RO_CODE_RO_DATA {
        size_ro_code_ro_data
    } else if len == RO_CODE_RO_DATA_RW_DATA {
        size_ro_code_ro_data_rw_data
    } else {
        return IResult::Error(ErrorKind::Custom(0));
    };

    match parser(&sizes) {
        IResult::Done(_, module) => IResult::Done(input, module),
        _ => IResult::Error(ErrorKind::Custom(0)),
    }
}

/// Parses a row of module sizes
named!(size_rowp<&[u8], Module>,
    do_parse!(
        bs: not_line_ending >>
        m: apply!(_size_rowp, bs) >>
        (m)
    )
);

/// Parses the module name and sizes
fn modulep(input: &[u8], nbytes: usize) -> IResult<&[u8], (String, Module)> {
    do_parse!(input,
        name: apply!(namep, nbytes) >>
        module: size_rowp >>
        ((name.expect("No obj file name parsed from modulep"), module))
    )
}

/// Parses a complete module summary table row
fn table_rowp(input: &[u8], nbytes: usize) -> IResult<&[u8], (String, Module)> {
    do_parse!(input,
        m: apply!(modulep, nbytes) >>
        line_ending >>
        (m)
    )
}

/// Parses all module summary table rows, and inserts the results into a HashMap
pub fn module_table(input: &[u8], nbytes: usize) -> IResult<&[u8], HashMap<String, Module>> {
    fold_many0!(input, apply!(table_rowp, nbytes), HashMap::new(),
        |mut hm: HashMap<_,_>, sm: (String, Module)| {
            hm.insert(sm.0, sm.1);
            hm
        }
    )
}

#[cfg(test)]
mod tests {

    use super::*;
    use nom::IResult;

    static EMPTY: &[u8] = b"";
    static EOL: &[u8] = b"\n";

    #[test]
    fn test_bs_to_spaceless_string() {
        let bs = " 123            4  56 ".as_bytes();
        assert_eq!(bs_to_spaceless_string(bs), Some("123456".into()))
    }

    #[test]
    fn test_namep() {
        let bs = "    MVC_State_Observer_Interface.o ".as_bytes();
        let (rest, name) = namep(bs, 35).unwrap();
        assert_eq!(name, Some("MVC_State_Observer_Interface.o".into()));
        assert_eq!(rest, EMPTY);
    }

    #[test]
    fn test_sizep() {
        let bs = "  1 360".as_bytes();
        let (rest, size) = sizep(bs).unwrap();
        assert_eq!(size, Some(1360));
        assert_eq!(rest, EMPTY);
    }

    #[test]
    fn test_module_all_three_values() {
        let bs = "    BigFoosBarsBaz.o                   532      569      103\n".as_bytes();
        let (rest, namedmod) = modulep(bs, 35).unwrap();
        assert_eq!(rest, EOL);
        assert_eq!(namedmod.0, "BigFoosBarsBaz.o");
        assert_eq!(namedmod.1, Module{ ro_code: Some(532), ro_data: Some(569), rw_data: Some(103)});
    }

    #[test]
    fn test_module_missing_last_two() {
        let bs = "    BigFoosBarsBaz.o                   532\n".as_bytes();
        let expected: (String, Module) = (
            "BigFoosBarsBaz.o".into(),
            Module {
                ro_code: Some(532),
                ro_data: None,
                rw_data: None,
            },
        );
        let actual = modulep(bs, 35);
        assert_eq!(actual, IResult::Done(EOL, expected));
    }

    #[test]
    fn test_module_missing_first_two() {
        let bs = "    BigFoosBarsBaz.o                                     103\n".as_bytes();
        let expected: (String, Module) = (
            "BigFoosBarsBaz.o".into(),
            Module {
                ro_code: None,
                ro_data: None,
                rw_data: Some(103),
            },
        );
        let actual = modulep(bs, 35);
        assert_eq!(actual, IResult::Done(EOL, expected));
    }

    #[test]
    fn test_module_missing_middle() {
        let bs = "    BigFoosBarsBaz.o                   532               103\n".as_bytes();
        let expected: (String, Module) = (
            "BigFoosBarsBaz.o".into(),
            Module {
                ro_code: Some(532),
                ro_data: None,
                rw_data: Some(103),
            },
        );
        let actual = modulep(bs, 35);
        assert_eq!(actual, IResult::Done(EOL, expected));
    }

    #[test]
    fn test_module_missing_last() {
        let bs = "    BigFoosBarsBaz.o                   166       32\n".as_bytes();
        let expected: (String, Module) = (
            "BigFoosBarsBaz.o".into(),
            Module {
                ro_code: Some(166),
                ro_data: Some(32),
                rw_data: None,
            },
        );
        let actual = modulep(bs, 35);
        assert_eq!(actual, IResult::Done(EOL, expected));
    }

    #[test]
    fn test_module_missing_first() {
        let bs = "    UI_AccessoryFooBar.o                      7 348      128\n".as_bytes();
        let expected: (String, Module) = (
            "UI_AccessoryFooBar.o".into(),
            Module {
                ro_code: None,
                ro_data: Some(7348),
                rw_data: Some(128),
            },
        );
        let actual = modulep(bs, 35);
        assert_eq!(actual, IResult::Done(EOL, expected));
    }

    #[test]
    fn test_module_only_middle() {
        let bs = "    UI_wbstring_ENG.o                        13 172\n".as_bytes();
        let expected: (String, Module) = (
            "UI_wbstring_ENG.o".into(),
            Module {
                ro_code: None,
                ro_data: Some(13172),
                rw_data: None,
            },
        );
        let actual = modulep(bs, 35);
        assert_eq!(actual, IResult::Done(EOL, expected));
    }

    #[test]
    fn test_module_table_one_row() {
        let table = b"    UI_AbstractFactoryThing.o          724      544\n";

        let mut expected: HashMap<String, Module> = HashMap::new();
        expected.insert(
            "UI_AbstractFactoryThing.o".into(),
            Module {
                ro_code: Some(724),
                ro_data: Some(544),
                rw_data: None,
            },
        );

        let result = module_table(table, 35);
        assert_eq!(result, IResult::Done(EMPTY, expected));

    }

    #[test]
    fn test_module_table_two_row() {
        let table = b"    UI_AbstractFactoryThing.o          724      544\n    BigFoosBarsBaz.o                   532      569      103\n";

        let mut expected: HashMap<String, Module> = HashMap::new();

        expected.insert(
            "UI_AbstractFactoryThing.o".into(),
            Module {
                ro_code: Some(724),
                ro_data: Some(544),
                rw_data: None,
            },
        );

        expected.insert(
            "BigFoosBarsBaz.o".into(),
            Module {
                ro_code: Some(532),
                ro_data: Some(569),
                rw_data: Some(103),
            },
        );

        let result = module_table(table, 35);
        assert_eq!(result, IResult::Done(EMPTY, expected));
    }

    #[test]
    fn test_module_table_three_row() {
        let table = b"    UI_AbstractFactoryThing.o          724      544\n    BigFoosBarsBaz.o                   532      569      103\n    UI_FoosBars.o                                88       16\n";

        let mut expected: HashMap<String, Module> = HashMap::new();

        expected.insert(
            "UI_AbstractFactoryThing.o".into(),
            Module {
                ro_code: Some(724),
                ro_data: Some(544),
                rw_data: None,
            },
        );

        expected.insert(
            "BigFoosBarsBaz.o".into(),
            Module {
                ro_code: Some(532),
                ro_data: Some(569),
                rw_data: Some(103),
            },
        );

        expected.insert(
            "UI_FoosBars.o".into(),
            Module {
                ro_code: None,
                ro_data: Some(88),
                rw_data: Some(16),
            },
        );

        let result = module_table(table, 35);
        assert_eq!(result, IResult::Done(EMPTY, expected));
    }

    #[test]
    fn test_module_table_empty_row() {
        let table = b"";

        let empty: HashMap<String, Module> = HashMap::new();

        let result = module_table(table, 35);
        assert_eq!(result, IResult::Done(EMPTY, empty));
    }

}
