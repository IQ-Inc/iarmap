//! The analytics module implements module summary analytics

use colored::*;

use iarmap::Module;
use iarmap::ObjModuleTable;

use std::collections::HashSet;
use std::collections::HashMap;

use std::fmt;

/// ColoredDiffModule wraps a Module for showing with colored formatting.
/// The wrapper is used when showing the delta between two modules.
struct ColoredDiffModule {
    m: Module
}

impl fmt::Display for ColoredDiffModule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {

        let painter = |size: Option<i32>| -> ColoredString {
            match size {
                None => Module::size_to_string(size).normal(),
                Some(v) => {
                    if v < 0 {
                        Module::size_to_string(size).red()
                    } else if v > 0 {
                        Module::size_to_string(size).green()
                    } else {
                        Module::size_to_string(size).normal()
                    }
                }
            }
        };

        let ro_code: ColoredString = painter(self.m.ro_code);
        let ro_data: ColoredString = painter(self.m.ro_data);
        let rw_data: ColoredString = painter(self.m.rw_data);
        write!(
            f,
            "ro_code: {} \t ro_data: {} \t rw_data: {}",
            ro_code,
            ro_data,
            rw_data
        )
    }
}

/// Run analytics on the left and right module summary tables
pub fn analyze(left: Vec<ObjModuleTable>, right: Vec<ObjModuleTable>) {

    show_module_differences(&left, &right);

    // Turn the vectors of maps into a single, large map
    let (left, right): (HashMap<String, Module>, HashMap<String, Module>) =
        twice(left, right, |v| {
            let mut map = HashMap::new();
            for obj in v {
                for (key, val) in obj.table.clone() {
                    map.insert(key, val);
                }
            }
            map
        });

    compare_objects(&left, &right);
}

/// Show the differences in module archive names
fn show_module_differences(left: &Vec<ObjModuleTable>, right: &Vec<ObjModuleTable>) {

    let (left, right): (HashSet<_>, HashSet<_>) = twice(left, right, |v| {
        v.iter().map(|obj| obj.name.clone()).collect()
    });

    if left != right {
        println!("Modules unique to left...");
        let mut diff = left.difference(&right).collect::<Vec<_>>();
        diff.sort();
        for unique in diff {
            println!("\tL- {}", unique.magenta());
        }

        println!("Modules unique to right...");
        let mut diff = right.difference(&left).collect::<Vec<_>>();
        diff.sort();
        for unique in diff {
            println!("\tR- {}", unique.yellow());
        }
    } else {
        println!("No module differences");
    }
}

/// Compare objects across two map files
fn compare_objects(left: &HashMap<String, Module>, right: &HashMap<String, Module>) {

    let (lkeys, rkeys): (HashSet<&String>, HashSet<&String>) =
        twice(left, right, |m| m.keys().collect());

    if lkeys != rkeys {
        println!("Objects unique to left...");
        let mut diff = lkeys.difference(&rkeys).collect::<Vec<_>>();
        diff.sort();
        for unique in diff {
            println!("\tL- {}", unique.magenta());
            println!("\t   {}", left.get(*unique).unwrap());
        }

        println!("Objects unique to right...");
        let mut diff = rkeys.difference(&lkeys).collect::<Vec<_>>();
        diff.sort();
        for unique in diff {
            println!("\tR- {}", unique.yellow());
            println!("\t   {}", right.get(*unique).unwrap());
        }
    } else {
        println!("No unique objects between left and right");
    }

    let mut no_difference = true;
    let mut intersect: Vec<_> = lkeys.intersection(&rkeys).collect();
    intersect.sort();
    for obj in intersect {
        let l = left.get(*obj);
        let r = right.get(*obj);

        if let (Some(l), Some(r)) = (l, r) {
            if l == r {
                continue;
            }

            no_difference = false;
            println!("Difference in {}...", obj.cyan());
            println!("\tL- {}", l);
            println!("\tR- {}", r);
            println!("\tD- {}", ColoredDiffModule{ m: *l - *r });
        }
    }

    if no_difference {
        println!("Objects beween left and right were the same");
    }
}

//
// Helpers
//

/// Do action f twice to the left and right inputs
fn twice<T, F, R>(left: T, right: T, f: F) -> (R, R)
where
    F: Fn(T) -> R,
{
    let left: R = f(left);
    let right: R = f(right);
    (left, right)
}
